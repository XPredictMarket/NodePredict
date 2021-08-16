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

#[cfg(feature = "std")]
use frame_support::traits::GenesisBuild;

use frame_support::{
    dispatch::{DispatchError, Weight},
    ensure,
    traits::{Get, Time},
};
use sp_runtime::traits::{
    AccountIdConversion, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, One, Zero,
};
use xpmrl_traits::{
    pool::{LiquidityPool, LiquiditySubPool},
    tokens::Tokens,
    ProposalStatus as Status,
};

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::Time};
    use frame_system::pallet_prelude::*;
    use sp_runtime::{traits::Zero, ModuleId};
    use xpmrl_traits::{
        pool::{LiquidityPool, LiquiditySubPool},
        system::ProposalSystem,
        tokens::Tokens,
        ProposalStatus as Status,
    };
    use xpmrl_utils::with_transaction_result;

    pub(crate) type TimeOf<T> = <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::Time;
    pub(crate) type MomentOf<T> = <TimeOf<T> as Time>::Moment;

    pub(crate) type TokensOf<T> =
        <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::Tokens;
    pub(crate) type CurrencyIdOf<T> =
        <TokensOf<T> as Tokens<<T as frame_system::Config>::AccountId>>::CurrencyId;
    pub(crate) type ProposalIdOf<T> =
        <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::ProposalId;
    pub(crate) type VersionIdOf<T> =
        <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::VersionId;
    pub(crate) type BalanceOf<T> =
        <TokensOf<T> as Tokens<<T as frame_system::Config>::AccountId>>::Balance;

    /// This is the pallet's configuration trait
    #[pallet::config]
    pub trait Config: frame_system::Config + ProposalSystem<Self::AccountId> {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type SubPool: LiquiditySubPool<Self>;

        #[pallet::constant]
        type GovernanceCurrencyId: Get<CurrencyIdOf<Self>>;

        #[pallet::constant]
        type RewardId: Get<ModuleId>;
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

    /// storage proposal closed time, It's also the end time. After the end, you can upload the
    /// results
    #[pallet::storage]
    #[pallet::getter(fn proposal_close_time)]
    pub type ProposalCloseTime<T: Config> =
        StorageMap<_, Blake2_128Concat, ProposalIdOf<T>, MomentOf<T>, OptionQuery>;

    /// It stores the creation time of the proposal, which is mainly used to determine whether the
    /// proposal has expired and needs to be closed. If there is no heat after the proposal is put
    /// forward, it will be closed after a period of time.
    #[pallet::storage]
    #[pallet::getter(fn proposal_create_time)]
    pub type ProposalCreateTime<T: Config> =
        StorageMap<_, Blake2_128Concat, ProposalIdOf<T>, MomentOf<T>, OptionQuery>;

    /// Time when the proposal enters the announcement
    #[pallet::storage]
    #[pallet::getter(fn proposal_announcement_time)]
    pub type ProposalAnnouncementTime<T: Config> =
        StorageMap<_, Blake2_128Concat, ProposalIdOf<T>, MomentOf<T>>;

    /// After this period, the proposal will enter the formal proposal state based on voting
    #[pallet::storage]
    #[pallet::getter(fn proposal_automatic_expiration_time)]
    pub type ProposalAutomaticExpirationTime<T: Config> = StorageValue<_, MomentOf<T>>;

    /// The minimum difference between the end time of the proposal and the current time. The unit
    /// is milliseconds
    #[pallet::storage]
    #[pallet::getter(fn proposal_minimum_interval_time)]
    pub type ProposalMinimumIntervalTime<T: Config> = StorageValue<_, MomentOf<T>>;

    /// Stores the number of governance tokens pledged by users on a proposal
    ///
    /// `bool` means approval or disapproval
    /// - `true` means vote approval
    /// - `false` means vote disapproval
    #[pallet::storage]
    #[pallet::getter(fn proposal_vote_stake)]
    pub type ProposalVoteStake<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        ProposalIdOf<T>,
        Twox64Concat,
        T::AccountId,
        (BalanceOf<T>, bool),
        OptionQuery,
    >;

    /// Statistics of votes on a proposal
    ///
    /// `bool` means approval or disapproval
    /// - `true` means vote approval
    /// - `false` means vote disapproval
    #[pallet::storage]
    #[pallet::getter(fn proposal_count_vote)]
    pub type ProposalCountVote<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        ProposalIdOf<T>,
        Twox64Concat,
        bool,
        BalanceOf<T>,
        OptionQuery,
    >;

    /// The number of votes for the proposal must be greater than the minimum number of votes, so
    /// that the proposal can be converted to a formal proposal
    #[pallet::storage]
    #[pallet::getter(fn minimum_vote)]
    pub type MinimumVote<T: Config> = StorageValue<_, BalanceOf<T>, OptionQuery>;

    /// The default reward value is used to set the default value when creating a proposal. If no
    /// reward is needed later, it can be set to 0
    #[pallet::storage]
    #[pallet::getter(fn default_reward)]
    pub type DefaultReward<T: Config> = StorageValue<_, BalanceOf<T>, OptionQuery>;

    /// The total amount of rewards given to voters when each proposal becomes a formal proposal
    #[pallet::storage]
    #[pallet::getter(fn proposal_reward)]
    pub type ProposalReward<T: Config> =
        StorageMap<_, Blake2_128Concat, ProposalIdOf<T>, BalanceOf<T>, OptionQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub expiration_time: u32,
        pub minimum_interval_time: u32,
        pub minimum_vote: BalanceOf<T>,
        pub default_reward: BalanceOf<T>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                /// one week
                expiration_time: 7 * 24 * 60 * 60 * 1000,
                minimum_interval_time: 10 * 60 * 1000,
                minimum_vote: Zero::zero(),
                default_reward: Zero::zero(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            ProposalAutomaticExpirationTime::<T>::set(Some(self.expiration_time.into()));
            ProposalMinimumIntervalTime::<T>::set(Some(self.minimum_interval_time.into()));
            MinimumVote::<T>::set(Some(self.minimum_vote));
            DefaultReward::<T>::set(Some(self.default_reward));
        }
    }

    #[pallet::event]
    #[pallet::metadata(T::AccountI = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ProposalStatusChanged(ProposalIdOf<T>, Status),
        StakeTo(T::AccountId, ProposalIdOf<T>, BalanceOf<T>),
        UnStakeFrom(T::AccountId, ProposalIdOf<T>, BalanceOf<T>),
        DepositReward(T::AccountId, T::AccountId, BalanceOf<T>),
        ReclaimReward(T::AccountId, T::AccountId, BalanceOf<T>),
        WithdrawalReward(T::AccountId, ProposalIdOf<T>, BalanceOf<T>),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The proposal id has reached the upper limit
        ProposalIdOverflow,
        ProposalIdNotExist,
        /// The status of the current proposal is incorrect, and the current operation is not
        /// supported.
        ProposalAbnormalState,
        /// When setting the state, the new state cannot be the same as the old state
        StatusMustDiff,
        /// Users are not allowed to vote on the same proposal repeatedly
        NonRrepeatableStake,
        VoteOverflow,
        /// Proposal owners are not allowed to submit their own proposals
        OwnerNotAllowedVote,
        ProposalAbnormalVote,
        AccountNotStake,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// When the block is encapsulated, execute the following hook function
        ///
        /// At this time, it is used to automatically expire the proposal
        fn on_initialize(n: T::BlockNumber) -> Weight {
            Self::begin_block(n).unwrap_or_else(|e| {
                sp_runtime::print(e);
                0
            })
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
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
            let state = <Self as LiquidityPool<T>>::get_proposal_state(proposal_id)?;
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

        /// If users are interested in a proposal and need to vote, they need to pledge a certain
        /// amount of governance tokens to vote
        ///
        /// The dispatch origin for this call must be `Signed` by the transactor.
        #[pallet::weight(1_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn stake_to(
            origin: OriginFor<T>,
            proposal_id: ProposalIdOf<T>,
            number: BalanceOf<T>,
            opinion: bool,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let state = <Self as LiquidityPool<T>>::get_proposal_state(proposal_id)?;
            ensure!(
                state == Status::OriginalPrediction,
                Error::<T>::ProposalAbnormalState
            );
            let owner = <Self as LiquidityPool<T>>::proposal_owner(proposal_id)?;
            ensure!(who != owner, Error::<T>::OwnerNotAllowedVote);
            let number = with_transaction_result(|| -> Result<BalanceOf<T>, DispatchError> {
                Self::inner_stake_to(&who, proposal_id, number, opinion)
            })?;
            Self::deposit_event(Event::<T>::StakeTo(who, proposal_id, number));
            Ok(().into())
        }

        /// Withdraw the stake, after the proposal status changes, the staked coins can be
        /// withdrawn
        ///
        /// The dispatch origin for this call must be `Signed` by the transactor.
        #[pallet::weight(1_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn unstake_from(
            origin: OriginFor<T>,
            proposal_id: ProposalIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let state = <Self as LiquidityPool<T>>::get_proposal_state(proposal_id)?;
            ensure!(
                state != Status::OriginalPrediction,
                Error::<T>::ProposalAbnormalState
            );
            let number = with_transaction_result(|| -> Result<BalanceOf<T>, DispatchError> {
                let currency_id = T::GovernanceCurrencyId::get();
                let number = ProposalVoteStake::<T>::try_mutate_exists(
                    proposal_id,
                    &who,
                    |optional| -> Result<BalanceOf<T>, DispatchError> {
                        match optional.clone() {
                            Some(val) => {
                                *optional = None;
                                Ok(val.0)
                            }
                            None => Ok(Zero::zero()),
                        }
                    },
                )?;
                <TokensOf<T> as Tokens<T::AccountId>>::unreserve(currency_id, &who, number)?;
                Ok(number)
            })?;
            Self::deposit_event(Event::<T>::UnStakeFrom(who, proposal_id, number));
            Ok(().into())
        }

        /// If the proposal becomes a formal proposal, the users who voted for it can withdraw the
        /// corresponding reward
        ///
        /// The dispatch origin for this call must be `Signed` by the transactor.
        #[pallet::weight(1_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn withdrawal_reward(
            origin: OriginFor<T>,
            proposal_id: ProposalIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let state = <Self as LiquidityPool<T>>::get_proposal_state(proposal_id)?;
            ensure!(
                state != Status::OriginalPrediction,
                Error::<T>::ProposalAbnormalState
            );
            let minimum_vote = MinimumVote::<T>::get().unwrap_or_else(Zero::zero);
            let approval =
                ProposalCountVote::<T>::get(proposal_id, true).unwrap_or_else(Zero::zero);
            let disapproval =
                ProposalCountVote::<T>::get(proposal_id, false).unwrap_or_else(Zero::zero);
            ensure!(
                approval > disapproval && approval > minimum_vote,
                Error::<T>::ProposalAbnormalVote
            );
            let number = with_transaction_result(|| -> Result<BalanceOf<T>, DispatchError> {
                Self::inner_withdrawal_reward(&who, proposal_id, approval)
            })?;
            Self::deposit_event(Event::<T>::WithdrawalReward(who, proposal_id, number));
            Ok(().into())
        }

        /// Deposit the reward amount, this is usually called by the project party, this amount
        /// will be used to issue the corresponding reward
        ///
        /// The dispatch origin for this call must be `Signed` by the transactor.
        #[pallet::weight(1_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn deposit_reward(
            origin: OriginFor<T>,
            number: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let currency_id = T::GovernanceCurrencyId::get();
            let reward_account = Self::module_account();
            let number = with_transaction_result(|| -> Result<BalanceOf<T>, DispatchError> {
                <TokensOf<T> as Tokens<T::AccountId>>::transfer(
                    currency_id,
                    &who,
                    &reward_account,
                    number,
                )
            })?;
            Self::deposit_event(Event::<T>::DepositReward(who, reward_account, number));
            Ok(().into())
        }

        /// Reclaim unused rewards
        ///
        /// The dispatch origin for this call is `root`.
        #[pallet::weight(1_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn reclaim_reward(
            origin: OriginFor<T>,
            to: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_root(origin)?;
            let currency_id = T::GovernanceCurrencyId::get();
            let reward_account = Self::module_account();
            let number = with_transaction_result(|| -> Result<BalanceOf<T>, DispatchError> {
                let number =
                    <TokensOf<T> as Tokens<T::AccountId>>::balance(currency_id, &reward_account);
                <TokensOf<T> as Tokens<T::AccountId>>::transfer(
                    currency_id,
                    &reward_account,
                    &to,
                    number,
                )
            })?;
            Self::deposit_event(Event::<T>::ReclaimReward(reward_account, to, number));
            Ok(().into())
        }

        #[pallet::weight(1_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn set_proposal_minimum_interval_time(
            origin: OriginFor<T>,
            time: MomentOf<T>,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_root(origin)?;
            ProposalMinimumIntervalTime::<T>::set(Some(time));
            Ok(().into())
        }

        #[pallet::weight(1_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn set_default_reward(
            origin: OriginFor<T>,
            value: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_root(origin)?;
            DefaultReward::<T>::set(Some(value));
            Ok(().into())
        }
    }
}

#[cfg(feature = "std")]
impl<T: Config> GenesisConfig<T> {
    /// Direct implementation of `GenesisBuild::build_storage`.
    ///
    /// Kept in order not to break dependency.
    pub fn build_storage(&self) -> Result<sp_runtime::Storage, String> {
        <Self as GenesisBuild<T>>::build_storage(self)
    }

    /// Direct implementation of `GenesisBuild::assimilate_storage`.
    ///
    /// Kept in order not to break dependency.
    pub fn assimilate_storage(&self, storage: &mut sp_runtime::Storage) -> Result<(), String> {
        <Self as GenesisBuild<T>>::assimilate_storage(self, storage)
    }
}

impl<T: Config> Pallet<T> {
    fn module_account() -> T::AccountId {
        T::RewardId::get().into_account()
    }

    fn begin_block(_: T::BlockNumber) -> Result<Weight, DispatchError> {
        let now = <TimeOf<T> as Time>::now();
        let expiration_time = <Self as LiquidityPool<T>>::proposal_automatic_expiration_time();
        let max_id = <Self as LiquidityPool<T>>::max_proposal_id();
        let mut index: ProposalIdOf<T> = Zero::zero();
        let minimum_vote = MinimumVote::<T>::get().unwrap_or_else(Zero::zero);
        loop {
            if index >= max_id {
                break;
            }
            let start =
                ProposalCreateTime::<T>::get(index).ok_or(Error::<T>::ProposalIdNotExist)?;
            let end = ProposalCloseTime::<T>::get(index).ok_or(Error::<T>::ProposalIdNotExist)?;
            let diff = now.checked_sub(&start).unwrap_or_else(Zero::zero);
            let state = <Self as LiquidityPool<T>>::get_proposal_state(index)
                .unwrap_or(Status::OriginalPrediction);
            if diff > expiration_time && state == Status::OriginalPrediction {
                let approval = ProposalCountVote::<T>::get(index, true).unwrap_or_else(Zero::zero);
                let disapproval =
                    ProposalCountVote::<T>::get(index, false).unwrap_or_else(Zero::zero);
                if approval > disapproval && approval > minimum_vote {
                    <Self as LiquidityPool<T>>::set_proposal_state(
                        index,
                        Status::FormalPrediction,
                    )?;
                } else {
                    <Self as LiquidityPool<T>>::set_proposal_state(index, Status::End)?;
                }
            } else if now > end {
                if state == Status::OriginalPrediction {
                    <Self as LiquidityPool<T>>::set_proposal_state(index, Status::End)?;
                } else if state == Status::FormalPrediction {
                    <Self as LiquidityPool<T>>::set_proposal_state(
                        index,
                        Status::WaitingForResults,
                    )?;
                    ProposalAnnouncementTime::<T>::insert(index, now);
                }
            }
            index = index
                .checked_add(&One::one())
                .ok_or(Error::<T>::ProposalIdOverflow)?;
        }
        Ok(0)
    }

    fn inner_stake_to(
        who: &T::AccountId,
        proposal_id: ProposalIdOf<T>,
        number: BalanceOf<T>,
        opinion: bool,
    ) -> Result<BalanceOf<T>, DispatchError> {
        let currency_id = T::GovernanceCurrencyId::get();
        ProposalVoteStake::<T>::try_mutate(
            proposal_id,
            &who,
            |optional| -> Result<(), DispatchError> {
                match optional {
                    Some(_) => Err(Error::<T>::NonRrepeatableStake)?,
                    None => {
                        *optional = Some((number, opinion));
                        Ok(())
                    }
                }
            },
        )?;
        ProposalCountVote::<T>::try_mutate(
            proposal_id,
            opinion,
            |optional| -> Result<(), DispatchError> {
                let old = optional.unwrap_or_else(Zero::zero);
                let new = old.checked_add(&number).ok_or(Error::<T>::VoteOverflow)?;
                *optional = Some(new);
                Ok(())
            },
        )?;
        let number = <TokensOf<T> as Tokens<T::AccountId>>::reserve(currency_id, &who, number)?;
        Ok(number)
    }

    fn inner_withdrawal_reward(
        who: &T::AccountId,
        proposal_id: ProposalIdOf<T>,
        approval: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        let (number, opinion) =
            ProposalVoteStake::<T>::get(proposal_id, &who).ok_or(Error::<T>::AccountNotStake)?;
        let total = ProposalReward::<T>::get(proposal_id).ok_or(Error::<T>::ProposalIdNotExist)?;
        let base: BalanceOf<T> = 100u32.into();
        let currency_id = T::GovernanceCurrencyId::get();
        let reward_account = Self::module_account();
        match opinion {
            true => {
                let number = number * base;
                let number = number.checked_div(&approval).unwrap_or_else(Zero::zero);
                let number = number.checked_mul(&total).unwrap_or_else(Zero::zero);
                let number = number.checked_div(&base).unwrap_or_else(Zero::zero);
                <TokensOf<T> as Tokens<T::AccountId>>::transfer(
                    currency_id,
                    &reward_account,
                    &who,
                    number,
                )
            }
            false => Ok(Zero::zero()),
        }
    }

    fn set_new_status(
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
    fn get_proposal_minimum_interval_time() -> MomentOf<T> {
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
        create_time: MomentOf<T>,
        close_time: MomentOf<T>,
        version: VersionIdOf<T>,
    ) {
        ProposalStatus::<T>::insert(proposal_id, state);
        ProposalOwner::<T>::insert(proposal_id, owner.clone());
        ProposalLiquidateVersionId::<T>::insert(proposal_id, version);
        ProposalCreateTime::<T>::insert(proposal_id, create_time);
        ProposalCloseTime::<T>::insert(proposal_id, close_time);
        let default_reward = DefaultReward::<T>::get().unwrap_or_else(Zero::zero);
        ProposalReward::<T>::insert(proposal_id, default_reward);
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
        match ProposalStatus::<T>::get(proposal_id) {
            Some(state) => Ok(state),
            None => Err(Error::<T>::ProposalIdNotExist)?,
        }
    }

    fn set_proposal_state(
        proposal_id: ProposalIdOf<T>,
        new_state: Status,
    ) -> Result<Status, DispatchError> {
        Self::set_new_status(proposal_id, new_state)
    }

    fn proposal_owner(proposal_id: ProposalIdOf<T>) -> Result<T::AccountId, DispatchError> {
        match ProposalOwner::<T>::get(proposal_id) {
            Some(owner) => Ok(owner),
            None => Err(Error::<T>::ProposalIdNotExist)?,
        }
    }

    fn proposal_announcement_time(
        proposal_id: ProposalIdOf<T>,
    ) -> Result<MomentOf<T>, DispatchError> {
        match ProposalAnnouncementTime::<T>::get(proposal_id) {
            Some(time) => Ok(time),
            None => Err(Error::<T>::ProposalIdNotExist)?,
        }
    }
}
