//! <!-- markdown-link-check-disable -->
//! # Couple
//!
//! Run `cargo doc --package xpmrl-proposals --open` to view this pallet's documentation.
//!
//! General proposal entrypoint, a module that manages all versions of proposal information
//!
//! - [`xpmrl_proposals::Config`](./trait.Config.html)
//! - [`Call`](./enum.Call.html)
//! - [`Module`](./struct.Module.html)
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

use frame_support::{
    dispatch::{DispatchError, Weight},
    ensure,
    traits::{Get, Time},
};
use sp_runtime::traits::{CheckedAdd, CheckedSub, One, Zero};
use sp_std::vec::Vec;
use xpmrl_traits::{pool::LiquidityPool, ProposalStatus as Status};

#[frame_support::pallet]
pub mod pallet {
    use codec::FullCodec;
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::Time};
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::*;
    use sp_std::{fmt::Debug, vec::Vec};
    use xpmrl_traits::{pool::LiquidityPool, ProposalStatus as Status};
    use xpmrl_utils::with_transaction_result;

    pub(crate) type MomentOf<T> = <<T as Config>::Time as Time>::Moment;
    pub(crate) type BalanceOf<T> = <<T as Config>::LiquidityPool as LiquidityPool<
        <T as frame_system::Config>::AccountId,
        <T as Config>::ProposalId,
        <<T as Config>::Time as Time>::Moment,
        <T as Config>::CategoryId,
    >>::Balance;
    pub(crate) type CurrencyIdOf<T> = <<T as Config>::LiquidityPool as LiquidityPool<
        <T as frame_system::Config>::AccountId,
        <T as Config>::ProposalId,
        <<T as Config>::Time as Time>::Moment,
        <T as Config>::CategoryId,
    >>::CurrencyId;

    /// This is the pallet's configuration trait
    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// Get the timestamp of the current time
        type Time: Time;
        type ProposalId: FullCodec
            + Eq
            + PartialEq
            + Copy
            + MaybeSerializeDeserialize
            + Debug
            + AtLeast32BitUnsigned;
        type CategoryId: FullCodec
            + Eq
            + PartialEq
            + Copy
            + MaybeSerializeDeserialize
            + Debug
            + AtLeast32BitUnsigned;
        type VersionId: FullCodec
            + Eq
            + PartialEq
            + Copy
            + MaybeSerializeDeserialize
            + Debug
            + AtLeast32BitUnsigned;
        /// LiquidityPool trait, used to manipulate the couple module downward
        type LiquidityPool: LiquidityPool<
            Self::AccountId,
            Self::ProposalId,
            MomentOf<Self>,
            Self::CategoryId,
        >;

        /// Decimals of fee
        #[pallet::constant]
        type EarnTradingFeeDecimals: Get<u8>;

        #[pallet::constant]
        type CurrentLiquidateVersionId: Get<Self::VersionId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// Proposal id length, currently if a new proposal is created, it is the id of the new
    /// proposal
    #[pallet::storage]
    #[pallet::getter(fn current_proposal_id)]
    pub type CurrentProposalId<T: Config> = StorageValue<_, T::ProposalId>;

    /// Version id, forwarded to different processing modules through different versions
    ///
    /// This storage is not currently used
    #[pallet::storage]
    #[pallet::getter(fn proposal_liquidate_version_id)]
    pub type ProposalLiquidateVersionId<T: Config> =
        StorageMap<_, Blake2_128Concat, T::ProposalId, T::VersionId, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn proposal_status)]
    pub type ProposalStatus<T: Config> =
        StorageMap<_, Blake2_128Concat, T::ProposalId, Status, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn proposal_owner)]
    pub type ProposalOwner<T: Config> =
        StorageMap<_, Blake2_128Concat, T::ProposalId, T::AccountId, OptionQuery>;

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
    pub type ProposaMinimumIntervalTime<T: Config> = StorageValue<_, MomentOf<T>>;

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
            ProposaMinimumIntervalTime::<T>::set(Some(self.minimum_interval_time.into()));
        }
    }

    #[pallet::event]
    #[pallet::metadata(T::AccountI = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        NewProposal(T::AccountId, T::ProposalId, CurrencyIdOf<T>),
        ProposalStatusChanged(T::ProposalId, Status),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The proposal id has reached the upper limit
        ProposalIdOverflow,
        ProposalIdNotExist,
        /// When setting the state, the new state cannot be the same as the old state
        StatusMustDiff,
        CategoryIdNotZero,
        TokenIdNotZero,
        CloseTimeMustLargeThanNow,
        /// Illegal assets were used to create a proposal
        CurrencyIdNotAllowed,
        NumberMustMoreThanZero,
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
        /// Create a new proposal
        ///
        /// The dispatch origin for this call must be `Signed` by the transactor.
        #[pallet::weight(1_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn new_proposal(
            origin: OriginFor<T>,
            title: Vec<u8>,
            optional: [Vec<u8>; 2],
            close_time: MomentOf<T>,
            category_id: T::CategoryId,
            currency_id: CurrencyIdOf<T>,
            number: BalanceOf<T>,
            earn_fee: u32,
            detail: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(category_id != Zero::zero(), Error::<T>::CategoryIdNotZero);
            ensure!(currency_id != Zero::zero(), Error::<T>::TokenIdNotZero);
            let minimum_interval_time =
                ProposaMinimumIntervalTime::<T>::get().unwrap_or_else(Zero::zero);
            ensure!(
                close_time - T::Time::now() > minimum_interval_time,
                Error::<T>::CloseTimeMustLargeThanNow
            );
            ensure!(
                !ProposalUsedCurrencyId::<T>::contains_key(currency_id),
                Error::<T>::CurrencyIdNotAllowed
            );
            ensure!(number > Zero::zero(), Error::<T>::NumberMustMoreThanZero);
            let proposal_id = with_transaction_result(|| {
                let proposal_id = Self::inner_new_proposal_v1(
                    &who,
                    title,
                    close_time,
                    category_id,
                    currency_id,
                    optional,
                    number,
                    earn_fee,
                    detail,
                )?;
                ProposalStatus::<T>::insert(proposal_id, Status::OriginalPrediction);
                ProposalOwner::<T>::insert(proposal_id, who.clone());
                Ok(proposal_id)
            })?;
            Self::deposit_event(Event::NewProposal(who, proposal_id, currency_id));
            Ok(().into())
        }

        /// Set new state for proposal
        ///
        /// The dispatch origin for this call is `root`.
        #[pallet::weight(1_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn set_status(
            origin: OriginFor<T>,
            proposal_id: T::ProposalId,
            new_status: Status,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_root(origin)?;
            let status = with_transaction_result(|| Self::set_new_status(proposal_id, new_status))?;
            Self::deposit_event(Event::ProposalStatusChanged(proposal_id, status));
            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn begin_block(_: T::BlockNumber) -> Result<Weight, DispatchError> {
        let now = T::Time::now();
        let expiration_time =
            ProposalAutomaticExpirationTime::<T>::get().unwrap_or_else(Zero::zero);
        let max_id = CurrentProposalId::<T>::get().unwrap_or_else(Zero::zero);
        let mut index: <T as Config>::ProposalId = Zero::zero();
        loop {
            if index >= max_id {
                break;
            }
            let (start, end) = T::LiquidityPool::time(index)?;
            let diff = now.checked_sub(&start).unwrap_or_else(Zero::zero);
            let state = ProposalStatus::<T>::get(index).unwrap_or(Status::OriginalPrediction);
            if diff > expiration_time && state == Status::OriginalPrediction {
                Self::set_new_status(index, Status::End)?;
            } else if now > end {
                if state == Status::OriginalPrediction {
                    Self::set_new_status(index, Status::End)?;
                } else if state == Status::FormalPrediction {
                    Self::set_new_status(index, Status::WaitingForResults)?;
                }
            }
            index = index
                .checked_add(&One::one())
                .ok_or(Error::<T>::ProposalIdOverflow)?;
        }
        Ok(0)
    }

    pub fn set_new_status(
        proposal_id: T::ProposalId,
        new_status: Status,
    ) -> Result<Status, DispatchError> {
        if new_status == Status::End {
            T::LiquidityPool::finally_locked(proposal_id)?;
        }
        ProposalStatus::<T>::try_mutate(proposal_id, |status| -> Result<Status, DispatchError> {
            let old_status = status.ok_or(Error::<T>::ProposalIdNotExist)?;
            ensure!(old_status != new_status, Error::<T>::StatusMustDiff);
            *status = Some(new_status);
            Ok(new_status)
        })
    }

    pub fn get_next_proposal_id() -> Result<T::ProposalId, DispatchError> {
        CurrentProposalId::<T>::try_mutate(|value| -> Result<T::ProposalId, DispatchError> {
            let current_id = value.unwrap_or_else(Zero::zero);
            *value = Some(
                current_id
                    .checked_add(&One::one())
                    .ok_or(Error::<T>::ProposalIdOverflow)?,
            );
            Ok(current_id)
        })
    }

    pub fn inner_new_proposal_v1(
        who: &T::AccountId,
        title: Vec<u8>,
        close_time: MomentOf<T>,
        category_id: T::CategoryId,
        currency_id: CurrencyIdOf<T>,
        optional: [Vec<u8>; 2],
        number: BalanceOf<T>,
        earn_fee: u32,
        detail: Vec<u8>,
    ) -> Result<T::ProposalId, DispatchError> {
        let proposal_id = Self::get_next_proposal_id()?;
        let v1: T::VersionId = T::CurrentLiquidateVersionId::get();
        ProposalLiquidateVersionId::<T>::insert(proposal_id, v1);
        let (yes_id, no_id, lp_id) = T::LiquidityPool::new_liquidity_pool(
            &who,
            proposal_id,
            title,
            close_time,
            category_id,
            currency_id,
            optional,
            number,
            earn_fee,
            detail,
        )?;
        ProposalUsedCurrencyId::<T>::insert(yes_id, true);
        ProposalUsedCurrencyId::<T>::insert(no_id, true);
        ProposalUsedCurrencyId::<T>::insert(lp_id, true);
        Ok(proposal_id)
    }
}
