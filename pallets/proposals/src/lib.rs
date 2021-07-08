//! <!-- markdown-link-check-disable -->
//! # Couple
//!
//! Run `cargo doc --package xpmrl-proposals --open` to view this pallet's documentation.
//!
//! General proposal entrypoint, a module that manages all versions of proposal information
//!
//! - [`xpmrl_proposals::Config`](./pallet/trait.Config.html)
//! - [`Call`](./pallet/enum.Call.html)
//! - [`Pallet`](./pallet/struct.Pallet.html)
//!
//! ## Overview
//!
//! All versions of proposals are created through this entry, and specific operations are handled
//! by the corresponding module, which only manages the same information in the proposal.
//!

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use frame_support::{dispatch::DispatchError, ensure, traits::Get};
use sp_runtime::traits::{CheckedAdd, One, Zero};
use xpmrl_traits::{
    pool::{LiquidityPool, LiquiditySubPool},
    ProposalStatus as Status,
};

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::Time};
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;
    use xpmrl_traits::{
        couple::LiquidityCouple, pool::LiquiditySubPool, system::ProposalSystem, tokens::Tokens,
        ProposalStatus as Status,
    };
    use xpmrl_utils::with_transaction_result;

    pub(crate) type TimeOf<T> = <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::Time;
    pub(crate) type MomentOf<T> = <TimeOf<T> as Time>::Moment;

    pub(crate) type TokensOf<T> =
        <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::Tokens;
    pub(crate) type CurrencyIdOf<T> =
        <TokensOf<T> as Tokens<<T as frame_system::Config>::AccountId>>::CurrencyId;
    pub(crate) type BalanceOf<T> =
        <TokensOf<T> as Tokens<<T as frame_system::Config>::AccountId>>::Balance;
    pub(crate) type ProposalIdOf<T> =
        <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::ProposalId;
    pub(crate) type CategoryIdOf<T> =
        <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::CategoryId;
    pub(crate) type VersionIdOf<T> =
        <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::VersionId;

    /// This is the pallet's configuration trait
    #[pallet::config]
    pub trait Config: frame_system::Config + ProposalSystem<Self::AccountId> {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type SubPool: LiquiditySubPool<Self>;
        type CouplePool: LiquidityCouple<Self>;

        /// Decimals of fee
        #[pallet::constant]
        type EarnTradingFeeDecimals: Get<u8>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// Proposal id length, currently if a new proposal is created, it is the id of the new
    /// proposal
    #[pallet::storage]
    #[pallet::getter(fn current_proposal_id)]
    pub type CurrentProposalId<T: Config> = StorageValue<_, ProposalIdOf<T>>;

    /// Version id, forwarded to different processing modules through different versions
    ///
    /// This storage is not currently used
    #[pallet::storage]
    #[pallet::getter(fn proposal_liquidate_version_id)]
    pub type ProposalLiquidateVersionId<T: Config> =
        StorageMap<_, Blake2_128Concat, ProposalIdOf<T>, VersionIdOf<T>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn proposal_status)]
    pub type ProposalStatus<T: Config> =
        StorageMap<_, Blake2_128Concat, ProposalIdOf<T>, Status, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn proposal_owner)]
    pub type ProposalOwner<T: Config> =
        StorageMap<_, Blake2_128Concat, ProposalIdOf<T>, T::AccountId, OptionQuery>;

    /// When creating a proposal, the asset ids that have been used cannot be used as settlement
    /// currency.
    #[pallet::storage]
    #[pallet::getter(fn proposal_used_currency_id)]
    pub type ProposalUsedCurrencyId<T: Config> =
        StorageMap<_, Blake2_128Concat, CurrencyIdOf<T>, bool, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn proposal_automatic_expiration_time)]
    pub type ProposalAutomaticExpirationTime<T: Config> = StorageValue<_, MomentOf<T>>;

    /// The minimum difference between the end time of the proposal and the current time. The unit
    /// is milliseconds
    #[pallet::storage]
    #[pallet::getter(fn proposal_minimum_interval_time)]
    pub type ProposalMinimumIntervalTime<T: Config> = StorageValue<_, MomentOf<T>>;

    /// The percentage of the commission that the creator of the proposal can get.
    #[pallet::storage]
    #[pallet::getter(fn proposal_liquidity_provider_fee_rate)]
    pub type ProposalLiquidityProviderFeeRate<T: Config> = StorageValue<_, u32>;

    #[pallet::genesis_config]
    pub struct GenesisConfig {
        pub expiration_time: u32,
        pub liquidity_provider_fee_rate: u32,
        pub minimum_interval_time: u32,
    }

    #[cfg(feature = "std")]
    impl Default for GenesisConfig {
        fn default() -> Self {
            Self {
                expiration_time: 3 * 24 * 60 * 60 * 1000,
                liquidity_provider_fee_rate: 9000,
                minimum_interval_time: 60 * 1000,
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            ProposalAutomaticExpirationTime::<T>::set(Some(self.expiration_time.into()));
            ProposalLiquidityProviderFeeRate::<T>::set(Some(self.liquidity_provider_fee_rate));
            ProposalMinimumIntervalTime::<T>::set(Some(self.minimum_interval_time.into()));
        }
    }

    #[pallet::event]
    #[pallet::metadata(T::AccountI = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ProposalStatusChanged(ProposalIdOf<T>, Status),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The proposal id has reached the upper limit
        ProposalIdOverflow,
        ProposalIdNotExist,
        /// The status of the current proposal is incorrect, and the current operation is not
        ///  supported.
        ProposalAbnormalState,
        /// When setting the state, the new state cannot be the same as the old state
        StatusMustDiff,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> frame_support::weights::Weight {
            if let Ok(_) = ProposalMinimumIntervalTime::<T>::try_mutate(
                |optional| -> Result<(), DispatchError> {
                    if let None = optional {
                        let minimum_interval_time: u32 = 10 * 60 * 1000;
                        *optional = Some(minimum_interval_time.into());
                    }
                    Ok(())
                },
            ) {}
            0
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new proposal
        ///
        /// The method has been deleted and moved to the couple module
        ///
        /// The dispatch origin for this call must be `Signed` by the transactor.
        #[pallet::weight(0)]
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
            T::CouplePool::new_couple_proposal(
                origin,
                title,
                optional,
                close_time,
                category_id,
                currency_id,
                number,
                earn_fee,
                detail,
            )
        }

        /// Set new state for proposal
        ///
        /// The dispatch origin for this call is `root`.
        #[pallet::weight(1_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn set_status(
            origin: OriginFor<T>,
            proposal_id: ProposalIdOf<T>,
            new_status: Status,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_root(origin)?;
            let state =
                ProposalStatus::<T>::get(proposal_id).ok_or(Error::<T>::ProposalIdNotExist)?;
            ensure!(
                state == Status::OriginalPrediction,
                Error::<T>::ProposalAbnormalState
            );
            if new_status != Status::End {
                ensure!(
                    new_status == Status::FormalPrediction,
                    Error::<T>::ProposalAbnormalState
                );
            }
            let state = with_transaction_result(|| Self::set_new_status(proposal_id, new_status))?;
            Self::deposit_event(Event::ProposalStatusChanged(proposal_id, state));
            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn set_new_status(
        proposal_id: ProposalIdOf<T>,
        new_status: Status,
    ) -> Result<Status, DispatchError> {
        if new_status == Status::End {
            T::SubPool::finally_locked(proposal_id)?;
        }
        ProposalStatus::<T>::try_mutate(proposal_id, |status| -> Result<Status, DispatchError> {
            let old_status = status.ok_or(Error::<T>::ProposalIdNotExist)?;
            ensure!(old_status != new_status, Error::<T>::StatusMustDiff);
            *status = Some(new_status);
            Ok(new_status)
        })
    }

    fn get_next_proposal_id() -> Result<ProposalIdOf<T>, DispatchError> {
        CurrentProposalId::<T>::try_mutate(|value| -> Result<ProposalIdOf<T>, DispatchError> {
            let current_id = value.unwrap_or_else(Zero::zero);
            *value = Some(
                current_id
                    .checked_add(&One::one())
                    .ok_or(Error::<T>::ProposalIdOverflow)?,
            );
            Ok(current_id)
        })
    }
}

impl<T: Config> LiquidityPool<T> for Pallet<T> {
    fn get_proposa_minimum_interval_time() -> MomentOf<T> {
        ProposalMinimumIntervalTime::<T>::get().unwrap_or_else(Zero::zero)
    }

    fn is_currency_id_used(currency_id: CurrencyIdOf<T>) -> bool {
        ProposalUsedCurrencyId::<T>::contains_key(currency_id)
    }

    fn get_next_proposal_id() -> Result<ProposalIdOf<T>, DispatchError> {
        Self::get_next_proposal_id()
    }

    fn init_proposal(
        proposal_id: ProposalIdOf<T>,
        owner: &T::AccountId,
        state: Status,
        version: VersionIdOf<T>,
    ) {
        ProposalStatus::<T>::insert(proposal_id, state);
        ProposalOwner::<T>::insert(proposal_id, owner.clone());
        ProposalLiquidateVersionId::<T>::insert(proposal_id, version);
    }

    fn append_used_currency(currency_id: CurrencyIdOf<T>) {
        ProposalUsedCurrencyId::<T>::insert(currency_id, true);
    }

    fn max_proposal_id() -> ProposalIdOf<T> {
        CurrentProposalId::<T>::get().unwrap_or_else(Zero::zero)
    }

    fn proposal_automatic_expiration_time() -> MomentOf<T> {
        ProposalAutomaticExpirationTime::<T>::get().unwrap_or_else(Zero::zero)
    }

    fn get_proposal_state(proposal_id: ProposalIdOf<T>) -> Result<Status, DispatchError> {
        ProposalStatus::<T>::get(proposal_id).ok_or(Err(Error::<T>::ProposalIdNotExist)?)
    }

    fn set_proposal_state(
        proposal_id: ProposalIdOf<T>,
        new_state: Status,
    ) -> Result<Status, DispatchError> {
        Self::set_new_status(proposal_id, new_state)
    }

    fn get_earn_trading_fee_decimals() -> u8 {
        T::EarnTradingFeeDecimals::get()
    }

    fn proposal_liquidity_provider_fee_rate() -> u32 {
        ProposalLiquidityProviderFeeRate::<T>::get().unwrap_or_else(Zero::zero)
    }

    fn proposal_owner(proposal_id: ProposalIdOf<T>) -> Result<T::AccountId, DispatchError> {
        ProposalOwner::<T>::get(proposal_id).ok_or(Err(Error::<T>::ProposalIdNotExist)?)
    }
}
