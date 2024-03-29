//! <!-- markdown-link-check-disable -->
//! # Couple
//!
//! Run `cargo doc --package xpmrl-couple --open` to view this pallet's documentation.
//!
//! A module dedicated to processing two option proposals
//!
//! - [`xpmrl_couple::Config`](./pallet/trait.Config.html)
//! - [`Call`](./pallet/enum.Call.html)
//! - [`Pallet`](./pallet/struct.Pallet.html)
//!
//! ## Overview
//!
//! This module allows users to participate in the sale and liquidity of proposals, and users
//! can earn the corresponding settlement currency through this module
//!
//! The transaction fees generated by buying and selling will be given to the liquidity provider,
//! who can provide liquidity and participate in the total market pool
//!
//! 10% of the transaction fee is proposed by the provider, and 90% is given to the liquidity
//! provider, and the final transaction fee is allocated according to the proportion of liquidity.
//!
//! For the specific rules of buying and selling, please refer to our white paper
//!

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unused_unit)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Import macros about storage-related operations
pub(crate) mod macros;
pub(crate) mod tools;

use frame_support::traits::Get;
use frame_system::RawOrigin;
use sp_runtime::DispatchError;
use xpmrl_traits::{couple::LiquidityCouple, pool::LiquiditySubPool};

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::Time};
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::Zero;
    use sp_std::vec::Vec;
    use xpmrl_traits::{
        autonomy::Autonomy, pool::LiquidityPool, ruler::RulerAccounts, system::ProposalSystem,
        tokens::Tokens, ProposalStatus,
    };
    use xpmrl_utils::with_transaction_result;

    pub(crate) type TokensOf<T> =
        <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::Tokens;
    pub(crate) type CurrencyIdOf<T> =
        <TokensOf<T> as Tokens<<T as frame_system::Config>::AccountId>>::CurrencyId;
    pub(crate) type BalanceOf<T> =
        <TokensOf<T> as Tokens<<T as frame_system::Config>::AccountId>>::Balance;
    pub(crate) type CategoryIdOf<T> =
        <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::CategoryId;
    pub(crate) type ProposalIdOf<T> =
        <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::ProposalId;
    pub(crate) type VersionIdOf<T> =
        <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::VersionId;
    pub(crate) type TimeOf<T> = <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::Time;
    pub(crate) type MomentOf<T> = <TimeOf<T> as Time>::Moment;

    macro_rules! ensure_optional_id_belong_proposal {
        ($id: ident, $proposal_id: ident) => {
            let (asset_id_1, asset_id_2) =
                PoolPairs::<T>::get($proposal_id).ok_or(Error::<T>::ProposalIdNotExist)?;
            ensure!(
                $id == asset_id_1 || $id == asset_id_2,
                Error::<T>::CurrencyIdNotFound
            );
        };
    }

    /// Basic attributes of the proposal
    #[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, Default)]
    pub struct Proposal<CategoryId> {
        pub title: Vec<u8>,
        /// The category of the proposal, such as sports, competition
        pub category_id: CategoryId,
        /// The specific description of the proposal
        pub detail: Vec<u8>,
    }

    /// This is the pallet's configuration trait
    ///
    /// Inherited from the proposal pallet, it can use the related functions of the proposal
    /// pallet, which is equivalent to deriving the function of the proposal pallet.
    #[pallet::config]
    pub trait Config:
        frame_system::Config + ProposalSystem<<Self as frame_system::Config>::AccountId>
    {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Pool: LiquidityPool<Self>;
        type Ruler: RulerAccounts<Self>;
        type Autonomy: Autonomy<Self>;

        /// Decimals of fee
        #[pallet::constant]
        type EarnTradingFeeDecimals: Get<u8>;

        #[pallet::constant]
        type CurrentLiquidateVersionId: Get<VersionIdOf<Self>>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// store the basic attributes of all proposals.
    #[pallet::storage]
    #[pallet::getter(fn proposals)]
    pub type Proposals<T: Config> =
        StorageMap<_, Blake2_128Concat, ProposalIdOf<T>, Proposal<T::CategoryId>, OptionQuery>;

    /// It stores the option tokens of the proposal
    #[pallet::storage]
    #[pallet::getter(fn pool_pairs)]
    pub type PoolPairs<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        ProposalIdOf<T>,
        (CurrencyIdOf<T>, CurrencyIdOf<T>),
        OptionQuery,
    >;

    /// It stores the settlement token of the proposal
    #[pallet::storage]
    #[pallet::getter(fn proposal_currency_id)]
    pub type ProposalCurrencyId<T: Config> =
        StorageMap<_, Blake2_128Concat, ProposalIdOf<T>, CurrencyIdOf<T>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn proposal_total_volume)]
    pub type ProposalTotalVolume<T: Config> =
        StorageMap<_, Blake2_128Concat, ProposalIdOf<T>, BalanceOf<T>, OptionQuery>;

    /// It stores the liquidity token of the proposal
    #[pallet::storage]
    #[pallet::getter(fn proposal_liquidate_currency_id)]
    pub type ProposalLiquidateCurrencyId<T: Config> =
        StorageMap<_, Blake2_128Concat, ProposalIdOf<T>, CurrencyIdOf<T>, OptionQuery>;

    /// It stores the fee rate of the proposal
    #[pallet::storage]
    #[pallet::getter(fn proposal_total_earn_trading_fee)]
    pub type ProposalTotalEarnTradingFee<T: Config> =
        StorageMap<_, Blake2_128Concat, ProposalIdOf<T>, u32, OptionQuery>;

    /// It stores the results of the proposal
    #[pallet::storage]
    #[pallet::getter(fn proposal_result)]
    pub type ProposalResult<T: Config> =
        StorageMap<_, Blake2_128Concat, ProposalIdOf<T>, CurrencyIdOf<T>, OptionQuery>;

    /// It stores the participating accounts of the proposal and how many settlement tokens it has
    /// deposited into the proposal
    #[pallet::storage]
    #[pallet::getter(fn proposal_account_info)]
    pub type ProposalAccountInfo<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        ProposalIdOf<T>,
        Twox64Concat,
        T::AccountId,
        BalanceOf<T>,
        OptionQuery,
    >;

    /// It stores the amount of all settlement currencies deposited in the proposal
    #[pallet::storage]
    #[pallet::getter(fn proposal_total_market)]
    pub type ProposalTotalMarket<T: Config> =
        StorageMap<_, Blake2_128Concat, ProposalIdOf<T>, BalanceOf<T>, OptionQuery>;

    /// It stores the number of option currencies for the proposal
    #[pallet::storage]
    #[pallet::getter(fn proposal_total_optional_market)]
    pub type ProposalTotalOptionalMarket<T: Config> =
        StorageMap<_, Blake2_128Concat, ProposalIdOf<T>, (BalanceOf<T>, BalanceOf<T>), OptionQuery>;

    /// It stores the final option currency information of the proposal, because after the proposal
    /// is over, as the user clears and liquidity is withdrawn, the total pool number will also
    /// change, which will affect the user’s revenue ratio, so it needs to be fixed after the
    /// proposal ends. Number of assets.
    #[pallet::storage]
    #[pallet::getter(fn proposal_finally_optional_market)]
    pub type ProposalFinallyTotalOptionalMarket<T: Config> =
        StorageMap<_, Blake2_128Concat, ProposalIdOf<T>, (BalanceOf<T>, BalanceOf<T>), OptionQuery>;

    /// It stores the total fee of the proposal
    #[pallet::storage]
    #[pallet::getter(fn proposal_total_market_fee)]
    pub type ProposalTotalMarketFee<T: Config> =
        StorageMap<_, Blake2_128Concat, ProposalIdOf<T>, BalanceOf<T>, OptionQuery>;

    /// It stores all the final fees of the proposal
    ///
    /// Same as `ProposalFinallyTotalOptionalMarket`
    #[pallet::storage]
    #[pallet::getter(fn proposal_finally_market_fee)]
    pub type ProposalFinallyMarketFee<T: Config> =
        StorageMap<_, Blake2_128Concat, ProposalIdOf<T>, BalanceOf<T>, OptionQuery>;

    /// It stores all the liquidity of the proposal
    #[pallet::storage]
    #[pallet::getter(fn proposal_total_market_liquid)]
    pub type ProposalTotalMarketLiquid<T: Config> =
        StorageMap<_, Blake2_128Concat, ProposalIdOf<T>, BalanceOf<T>, OptionQuery>;

    /// It stores all the final liquidity of the proposal
    ///
    /// Same as `ProposalFinallyTotalOptionalMarket`
    #[pallet::storage]
    #[pallet::getter(fn proposal_finally_market_liquid)]
    pub type ProposalFinallyMarketLiquid<T: Config> =
        StorageMap<_, Blake2_128Concat, ProposalIdOf<T>, BalanceOf<T>, OptionQuery>;

    /// It stores how much commission the provider of the proposal has withdrawn
    #[pallet::storage]
    #[pallet::getter(fn proposal_owner_already_withdrawn_fee)]
    pub type ProposalOwnerAlreadyWithdrawnFee<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        ProposalIdOf<T>,
        Twox64Concat,
        T::AccountId,
        BalanceOf<T>,
        OptionQuery,
    >;

    /// The percentage of the commission that the creator of the proposal can get.
    #[pallet::storage]
    #[pallet::getter(fn proposal_liquidity_provider_fee_rate)]
    pub type ProposalLiquidityProviderFeeRate<T: Config> = StorageValue<_, u32, OptionQuery>;

    /// After the prediction is successful, the withdrawal fee rate charged at the time of
    /// liquidation
    #[pallet::storage]
    #[pallet::getter(fn proposal_withdrawal_fee_rate)]
    pub type ProposalWithdrawalFeeRate<T: Config> = StorageValue<_, u32, OptionQuery>;

    /// After the proposal is over, when the user is clearing, the total settlement currency reward
    /// that the node that participates in providing the result can obtain
    #[pallet::storage]
    #[pallet::getter(fn proposal_total_autonomy_reward)]
    pub type ProposalTotalAutonomyReward<T: Config> =
        StorageMap<_, Blake2_128Concat, ProposalIdOf<T>, BalanceOf<T>, OptionQuery>;

    /// After the proposal is over, when the user is cleared, the nodes participating in providing
    /// the result can obtain the current settlement currency reward, because as the reward is
    /// added or withdrawn, the total pool will also be changed accordingly.
    #[pallet::storage]
    #[pallet::getter(fn proposal_current_autonomy_reward)]
    pub type ProposalCurrentAutonomyReward<T: Config> =
        StorageMap<_, Blake2_128Concat, ProposalIdOf<T>, BalanceOf<T>, OptionQuery>;

    /// With the withdrawal of users, this value will be updated accordingly. This value is the
    /// value of the current total pool, so the amount withdrawn by the user is:
    /// `current rewards` = `total pool` - `current value` * `percentage`
    /// `percentage` = 1 / `total number of correct votes`
    #[pallet::storage]
    #[pallet::getter(fn proposal_account_reward_start)]
    pub type ProposalAccountRewardStart<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        ProposalIdOf<T>,
        Twox64Concat,
        T::AccountId,
        BalanceOf<T>,
        OptionQuery,
    >;

    #[pallet::genesis_config]
    pub struct GenesisConfig {
        pub liquidity_provider_fee_rate: u32,
        pub withdrawal_fee_rate: u32,
    }

    #[cfg(feature = "std")]
    impl Default for GenesisConfig {
        fn default() -> Self {
            Self {
                liquidity_provider_fee_rate: 9000,
                withdrawal_fee_rate: 50,
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            ProposalLiquidityProviderFeeRate::<T>::set(Some(self.liquidity_provider_fee_rate));
            ProposalWithdrawalFeeRate::<T>::set(Some(self.withdrawal_fee_rate));
        }
    }

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        AddLiquidity(T::AccountId, ProposalIdOf<T>, CurrencyIdOf<T>, BalanceOf<T>),
        RemoveLiquidity(T::AccountId, ProposalIdOf<T>, CurrencyIdOf<T>, BalanceOf<T>),
        Buy(T::AccountId, ProposalIdOf<T>, CurrencyIdOf<T>, BalanceOf<T>),
        Sell(T::AccountId, ProposalIdOf<T>, CurrencyIdOf<T>, BalanceOf<T>),
        /// A liquidation event occurs after the liquidation, and the amount of liquidation will be
        /// included in the event
        Retrieval(T::AccountId, ProposalIdOf<T>, CurrencyIdOf<T>, BalanceOf<T>),
        SetResult(ProposalIdOf<T>, CurrencyIdOf<T>),
        NewProposal(T::AccountId, ProposalIdOf<T>, CurrencyIdOf<T>),
        WithdrawalReward(T::AccountId, ProposalIdOf<T>, BalanceOf<T>),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// When buying, selling or clearing, if the currency id you enter is not the id of the
        /// option currency, this error will be thrown
        CurrencyIdNotFound,
        /// The status of the current proposal is incorrect, and the current operation is not
        /// supported.
        ProposalAbnormalState,
        /// A non-existent proposal was executed
        ProposalIdNotExist,
        /// During liquidation, this error will be thrown if the proposal does not have a result
        /// set
        ProposalNotResult,
        /// The quantity overflowed during calculation
        BalanceOverflow,
        /// The equation has no real solution
        NoRealNumber,
        InsufficientBalance,
        CategoryIdNotZero,
        TokenIdNotZero,
        NumberMustMoreThanZero,
        CloseTimeMustLargeThanNow,
        CurrencyIdNotAllowed,
        /// The proposal id has reached the upper limit
        ProposalIdOverflow,
        /// What the user uploaded is not the correct result
        UploadedNotResult,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new proposal
        ///
        /// The dispatch origin for this call must be `Signed` by the transactor.
        #[pallet::weight(1_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn new_proposal(
            origin: OriginFor<T>,
            title: Vec<u8>,
            optional: [Vec<u8>; 2],
            close_time: MomentOf<T>,
            category_id: CategoryIdOf<T>,
            currency_id: CurrencyIdOf<T>,
            number: BalanceOf<T>,
            earn_fee: u32,
            detail: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(category_id > Zero::zero(), Error::<T>::CategoryIdNotZero);
            ensure!(currency_id > Zero::zero(), Error::<T>::TokenIdNotZero);
            ensure!(number > Zero::zero(), Error::<T>::NumberMustMoreThanZero);
            let now = <TimeOf<T> as Time>::now();
            let minimum_interval_time = T::Pool::get_proposal_minimum_interval_time();
            ensure!(
                close_time > now + minimum_interval_time,
                Error::<T>::CloseTimeMustLargeThanNow
            );
            ensure!(
                !T::Pool::is_currency_id_used(currency_id),
                Error::<T>::CurrencyIdNotAllowed
            );
            let proposal_id = with_transaction_result(|| {
                let proposal_id = T::Pool::get_next_proposal_id()?;
                Self::init_pool(
                    &who,
                    proposal_id,
                    title,
                    close_time,
                    category_id,
                    earn_fee,
                    detail,
                )?;
                Self::new_currency(&who, proposal_id, currency_id, number, optional)
            })?;
            Self::deposit_event(Event::NewProposal(who, proposal_id, currency_id));
            Ok(().into())
        }

        /// Provide liquidity to proposals
        ///
        /// The dispatch origin for this call must be `Signed` by the transactor.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn add_liquidity(
            origin: OriginFor<T>,
            proposal_id: ProposalIdOf<T>,
            number: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let status = T::Pool::get_proposal_state(proposal_id)?;
            ensure!(
                status == ProposalStatus::FormalPrediction,
                Error::<T>::ProposalAbnormalState
            );
            let currency_id =
                ProposalCurrencyId::<T>::get(proposal_id).ok_or(Error::<T>::ProposalIdNotExist)?;
            let (asset_id_1, asset_id_2) =
                PoolPairs::<T>::get(proposal_id).ok_or(Error::<T>::ProposalIdNotExist)?;
            let liquidate_currency_id = ProposalLiquidateCurrencyId::<T>::get(proposal_id)
                .ok_or(Error::<T>::ProposalIdNotExist)?;
            with_transaction_result(|| {
                Self::inner_add_liquidity(
                    &who,
                    proposal_id,
                    currency_id,
                    asset_id_1,
                    asset_id_2,
                    liquidate_currency_id,
                    number,
                )
            })?;
            Self::deposit_event(Event::AddLiquidity(who, proposal_id, currency_id, number));
            Ok(().into())
        }

        /// Get back your own assets through liquidity
        ///
        /// The dispatch origin for this call must be `Signed` by the transactor.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn remove_liquidity(
            origin: OriginFor<T>,
            proposal_id: ProposalIdOf<T>,
            number: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let status = T::Pool::get_proposal_state(proposal_id)?;
            ensure!(
                status == ProposalStatus::End,
                Error::<T>::ProposalAbnormalState
            );
            let currency_id =
                ProposalCurrencyId::<T>::get(proposal_id).ok_or(Error::<T>::ProposalIdNotExist)?;
            let liquidate_currency_id = ProposalLiquidateCurrencyId::<T>::get(proposal_id)
                .ok_or(Error::<T>::ProposalIdNotExist)?;
            let (asset_id_1, asset_id_2) =
                PoolPairs::<T>::get(proposal_id).ok_or(Error::<T>::ProposalIdNotExist)?;
            let (finally_o1, finally_o2) =
                ProposalFinallyTotalOptionalMarket::<T>::get(proposal_id)
                    .ok_or(Error::<T>::ProposalIdNotExist)?;
            with_transaction_result(|| {
                Self::inner_remove_liquidity(
                    &who,
                    proposal_id,
                    currency_id,
                    liquidate_currency_id,
                    asset_id_1,
                    asset_id_2,
                    number,
                    finally_o1,
                    finally_o2,
                )
            })?;
            Self::deposit_event(Event::RemoveLiquidity(
                who,
                proposal_id,
                currency_id,
                number,
            ));
            Ok(().into())
        }

        /// Buy option currency
        ///
        /// The dispatch origin for this call must be `Signed` by the transactor.
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn buy(
            origin: OriginFor<T>,
            proposal_id: ProposalIdOf<T>,
            optional_currency_id: CurrencyIdOf<T>,
            number: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let status = T::Pool::get_proposal_state(proposal_id)?;
            ensure!(
                status == ProposalStatus::FormalPrediction,
                Error::<T>::ProposalAbnormalState
            );
            let currency_id =
                ProposalCurrencyId::<T>::get(proposal_id).ok_or(Error::<T>::ProposalIdNotExist)?;
            ensure_optional_id_belong_proposal!(optional_currency_id, proposal_id);
            let other_currency = Self::get_other_optional_id(proposal_id, optional_currency_id)?;
            let actual_number = with_transaction_result(|| {
                Self::inner_buy(
                    &who,
                    proposal_id,
                    currency_id,
                    optional_currency_id,
                    number,
                    other_currency,
                )
            })?;
            Self::deposit_event(Event::Buy(
                who,
                proposal_id,
                optional_currency_id,
                actual_number,
            ));
            Ok(().into())
        }

        /// Sell option currency
        ///
        /// The dispatch origin for this call must be `Signed` by the transactor.
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn sell(
            origin: OriginFor<T>,
            proposal_id: ProposalIdOf<T>,
            optional_currency_id: CurrencyIdOf<T>,
            number: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let status = T::Pool::get_proposal_state(proposal_id)?;
            ensure!(
                status == ProposalStatus::FormalPrediction,
                Error::<T>::ProposalAbnormalState
            );
            let currency_id =
                ProposalCurrencyId::<T>::get(proposal_id).ok_or(Error::<T>::ProposalIdNotExist)?;
            ensure_optional_id_belong_proposal!(optional_currency_id, proposal_id);
            let other_currency = Self::get_other_optional_id(proposal_id, optional_currency_id)?;
            let actual_number = with_transaction_result(|| {
                Self::inner_sell(
                    &who,
                    proposal_id,
                    currency_id,
                    optional_currency_id,
                    number,
                    other_currency,
                )
            })?;
            Self::deposit_event(Event::Sell(
                who,
                proposal_id,
                optional_currency_id,
                actual_number,
            ));
            Ok(().into())
        }

        /// Settlement option currency, get settlement currency
        ///
        /// The dispatch origin for this call must be `Signed` by the transactor.
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn retrieval(
            origin: OriginFor<T>,
            proposal_id: ProposalIdOf<T>,
            optional_currency_id: CurrencyIdOf<T>,
            number: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let status = T::Pool::get_proposal_state(proposal_id)?;
            ensure!(
                status == ProposalStatus::End,
                Error::<T>::ProposalAbnormalState
            );
            ensure_optional_id_belong_proposal!(optional_currency_id, proposal_id);
            let result_id =
                ProposalResult::<T>::get(proposal_id).ok_or(Error::<T>::ProposalNotResult)?;
            let balance =
                <TokensOf<T> as Tokens<T::AccountId>>::balance(optional_currency_id, &who);
            ensure!(balance >= Zero::zero(), Error::<T>::InsufficientBalance);
            let number = if number >= balance { balance } else { number };
            let number = with_transaction_result(|| -> Result<BalanceOf<T>, DispatchError> {
                Self::inner_retrieval(&who, proposal_id, result_id, optional_currency_id, number)
            })?;
            Self::deposit_event(Event::Retrieval(who, proposal_id, result_id, number));
            Ok(().into())
        }

        /// Set result for proposal
        ///
        /// The dispatch origin for this call is `root`.
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn set_result(
            origin: OriginFor<T>,
            proposal_id: ProposalIdOf<T>,
            currency_id: CurrencyIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_root(origin)?;
            let status = T::Pool::get_proposal_state(proposal_id)?;
            ensure!(
                status == ProposalStatus::WaitingForResults,
                Error::<T>::ProposalAbnormalState
            );
            ensure_optional_id_belong_proposal!(currency_id, proposal_id);
            with_transaction_result(|| {
                T::Pool::set_proposal_state(proposal_id, ProposalStatus::End)?;
                ProposalResult::<T>::insert(proposal_id, currency_id);
                Self::finally_locked(proposal_id)
            })?;
            Self::deposit_event(Event::SetResult(proposal_id, currency_id));
            Ok(().into())
        }

        /// Set result for proposal when the state is over 
        ///
        /// The dispatch origin for this call is `root`.
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn set_result_when_end(
            origin: OriginFor<T>,
            proposal_id: ProposalIdOf<T>,
            currency_id: CurrencyIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_root(origin)?;
            let status = T::Pool::get_proposal_state(proposal_id)?;
            ensure!(
                status == ProposalStatus::End,
                Error::<T>::ProposalAbnormalState
            );
            ensure_optional_id_belong_proposal!(currency_id, proposal_id);
            with_transaction_result(|| {
                ProposalResult::<T>::insert(proposal_id, currency_id);
                Self::finally_locked(proposal_id)
            })?;
            Self::deposit_event(Event::SetResult(proposal_id, currency_id));
            Ok(().into())
        }
    }
}

impl<T: Config> LiquiditySubPool<T> for Pallet<T> {
    fn finally_locked(proposal_id: ProposalIdOf<T>) -> Result<(), DispatchError> {
        Self::finally_locked(proposal_id)
    }
}

impl<T: Config> LiquidityCouple<T> for Pallet<T> {
    fn proposal_pair(
        proposal_id: ProposalIdOf<T>,
    ) -> Result<(CurrencyIdOf<T>, CurrencyIdOf<T>), DispatchError> {
        match PoolPairs::<T>::get(proposal_id) {
            Some(pair) => Ok(pair),
            None => Err(Error::<T>::ProposalIdNotExist.into()),
        }
    }

    fn set_proposal_result(
        proposal_id: ProposalIdOf<T>,
        result: CurrencyIdOf<T>,
    ) -> Result<(), DispatchError> {
        match Self::set_result(RawOrigin::Root.into(), proposal_id, result) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.error),
        }
    }

    fn set_proposal_result_when_end(
        proposal_id: ProposalIdOf<T>,
        result: CurrencyIdOf<T>,
    ) -> Result<(), DispatchError> {
        match Self::set_result_when_end(RawOrigin::Root.into(), proposal_id, result) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.error),
        }
    }

    fn get_proposal_result(proposal_id: ProposalIdOf<T>) -> Result<CurrencyIdOf<T>, DispatchError> {
        match ProposalResult::<T>::get(proposal_id) {
            Some(result) => Ok(result),
            None => Err(Error::<T>::ProposalNotResult.into()),
        }
    }

    fn proposal_liquidate_currency_id(
        proposal_id: ProposalIdOf<T>,
    ) -> Result<CurrencyIdOf<T>, DispatchError> {
        match ProposalLiquidateCurrencyId::<T>::get(proposal_id) {
            Some(id) => Ok(id),
            None => Err(Error::<T>::ProposalIdNotExist.into()),
        }
    }
}
