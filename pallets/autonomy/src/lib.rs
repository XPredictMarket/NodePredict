//! <!-- markdown-link-check-disable -->
//! # Autonomy
//!
//! Run `cargo doc --package xpmrl-autonomy --open` to view this pallet's documentation.
//!
//! A module allows ordinary users to govern the results of proposals
//!
//! - [`xpmrl_autonomy::Config`](./pallet/trait.Config.html)
//! - [`Call`](./pallet/enum.Call.html)
//! - [`Pallet`](./pallet/struct.Pallet.html)
//!
//! ## Overview
//!
//! This module allows users to pledge governance tokens to become
//! governance nodes, and can upload or merge proposal results
//!
//! Only the data provided by the officially signed node is valid.
//!

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub use pallet::*;

use frame_support::{
    dispatch::{DispatchError, Weight},
    ensure,
    traits::{Get, Time},
};
use frame_system::offchain::SignedPayload;
use sp_core::crypto::KeyTypeId;
use sp_runtime::{
    traits::{CheckedAdd, One, Zero},
    transaction_validity::{
        InvalidTransaction, TransactionSource, TransactionValidity, ValidTransaction,
    },
};
use xpmrl_traits::{couple::LiquidityCouple, pool::LiquidityPool, tokens::Tokens, ProposalStatus};
use xpmrl_utils::with_transaction_result;

/// Defines application identifier for crypto keys of this module.
///
/// Every module that deals with signatures needs to declare its unique identifier for
/// its crypto keys.
/// When an offchain worker is signing transactions it's going to request keys from type
/// `KeyTypeId` via the keystore to sign the transaction.
/// The keys can be inserted manually via RPC (see `author_insertKey`).
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"xpml");
/// The type to sign and send transactions.
const UNSIGNED_TXS_PRIORITY: u64 = 100;

/// Based on the above `KeyTypeId` we need to generate a pallet-specific crypto type wrapper.
/// We can utilize the supported crypto kinds (`sr25519`, `ed25519` and `ecdsa`) and augment
/// them with the pallet-specific identifier.
pub mod crypto {
    use crate::KEY_TYPE;
    use sp_core::sr25519::Signature as Sr25519Signature;
    use sp_runtime::app_crypto::{app_crypto, sr25519};
    use sp_runtime::{traits::Verify, MultiSignature, MultiSigner};

    app_crypto!(sr25519, KEY_TYPE);

    pub struct OcwAuthId;

    // implemented for runtime
    impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for OcwAuthId {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }

    // implemented for mock runtime in test
    impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
        for OcwAuthId
    {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }
}

#[cfg(feature = "std")]
use frame_support::traits::GenesisBuild;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::Time};
    use frame_system::{offchain::*, pallet_prelude::*};
    use sp_runtime::traits::*;
    use xpmrl_traits::{
        couple::LiquidityCouple, pool::LiquidityPool, system::ProposalSystem, tokens::Tokens,
        ProposalStatus,
    };
    use xpmrl_utils::with_transaction_result;

    pub(crate) type TokensOf<T> =
        <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::Tokens;
    pub(crate) type CurrencyIdOf<T> =
        <TokensOf<T> as Tokens<<T as frame_system::Config>::AccountId>>::CurrencyId;
    pub(crate) type BalanceOf<T> =
        <TokensOf<T> as Tokens<<T as frame_system::Config>::AccountId>>::Balance;
    pub(crate) type ProposalIdOf<T> =
        <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::ProposalId;
    pub(crate) type TimeOf<T> = <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::Time;
    pub(crate) type MomentOf<T> = <TimeOf<T> as Time>::Moment;

    /// The payload struct of unsigned transaction with signed payload
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
    pub struct Payload<Public, ProposalId, ResultId> {
        /// The id of the proposal that needs to upload the result
        pub proposal_id: ProposalId,
        /// The asset id of the proposal result
        ///
        /// The proposal option is a token, so here only the id of the corresponding token needs to be uploaded
        pub result: ResultId,
        pub public: Public,
    }

    /// implament trait for payload
    /// make sure the payload can be signed and verify
    impl<T: Config> SignedPayload<T> for Payload<T::Public, T::ProposalId, CurrencyIdOf<T>> {
        fn public(&self) -> T::Public {
            self.public.clone()
        }
    }

    /// This is the pallet's configuration trait
    #[pallet::config]
    pub trait Config:
        frame_system::Config
        + SigningTypes
        + ProposalSystem<<Self as frame_system::Config>::AccountId>
    {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// The overarching dispatch call type.
        type Call: From<Call<Self>>;
        /// The identifier type for an offchain worker.
        type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
        type Pool: LiquidityPool<Self>;
        type CouplePool: LiquidityCouple<Self>;

        /// The asset id of the governance token
        ///
        /// This ensures that we only accept the assets of this id as governance assets
        #[pallet::constant]
        type StakeCurrencyId: Get<CurrencyIdOf<Self>>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// stored those accounts that staked tokens and their corresponding amounts
    #[pallet::storage]
    #[pallet::getter(fn staked_account)]
    pub type StakedAccount<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, OptionQuery>;

    /// actual governance account
    ///
    /// officially labeled account, but this account must have staked governance tokens
    #[pallet::storage]
    #[pallet::getter(fn autonomy_account)]
    pub type AutonomyAccount<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, (), OptionQuery>;

    /// used to temporarily store the results of the proposal uploaded by the governance account
    #[pallet::storage]
    #[pallet::getter(fn temporary_results)]
    pub type TemporaryResults<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::ProposalId,
        Twox64Concat,
        T::AccountId,
        CurrencyIdOf<T>,
        OptionQuery,
    >;

    /// Store the proposal id in the publicity and the time when the publicity started
    #[pallet::storage]
    #[pallet::getter(fn proposal_announcement)]
    pub type ProposalAnnouncement<T: Config> =
        StorageMap<_, Blake2_128Concat, T::ProposalId, MomentOf<T>, OptionQuery>;

    /// used to temporarily store the statistics of the proposal results uploaded from the governance account
    #[pallet::storage]
    #[pallet::getter(fn statistical_results)]
    pub type StatisticalResults<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, T::ProposalId, Twox64Concat, CurrencyIdOf<T>, u64>;

    /// the minimum number of governance tokens that need to be staked to become a governance node
    #[pallet::storage]
    #[pallet::getter(fn minimal_stake_number)]
    pub type MinimalStakeNumber<T: Config> = StorageValue<_, BalanceOf<T>, OptionQuery>;

    /// the minimum number of governance tokens that need to be staked to become a governance node
    #[pallet::storage]
    #[pallet::getter(fn publicity_interval)]
    pub type PublicityInterval<T: Config> = StorageValue<_, MomentOf<T>, OptionQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub minimal_number: BalanceOf<T>,
        pub interval: u32,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                minimal_number: Zero::zero(),
                interval: Zero::zero(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            MinimalStakeNumber::<T>::set(Some(self.minimal_number));
            PublicityInterval::<T>::set(Some(self.interval.into()));
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Account stake successfully.
        Stake(T::AccountId, BalanceOf<T>),
        /// Account unstake successfully.
        UnStake(T::AccountId, BalanceOf<T>),
        /// Punish evil nodes.
        Slash(T::AccountId, BalanceOf<T>),
        /// Tag an account.
        Tagging(T::AccountId),
        /// Untag an account.
        Untagging(T::AccountId),
        /// Account uploaded result.
        UploadResult(T::AccountId, T::ProposalId, CurrencyIdOf<T>),
        /// Set the minimum stake amount
        SetMinimalNumber(BalanceOf<T>),
        /// Set the publicity interval
        SetPublicityInterval(MomentOf<T>),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The account has already been staked and cannot be staked repeatedly
        AccountAlreadyStaked,
        /// Account not staked
        AccountNotStaked,
        /// The account has already uploaded the results, the same proposal cannot be uploaded
        /// again
        AccountHasAlreadyUploaded,
        /// Value has been overflow
        Overflow,
        /// Proposal is not at the wait for result, unable to upload results
        ProposalAbnormalState,
        /// Incorrect proposal options
        ProposalOptionNotCorrect,
        ProposalIdOverflow,
        /// The final count of all the options of the proposal is equal, and the final result
        /// cannot be obtained
        ResultIsEqual,
        /// The account has been tagged and cannot be tagged again
        AccountHasTagged,
        /// The account has not been tagged yet
        AccountNotTagged,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// Offchain Worker entry point.
        ///
        /// The node can upload the result in its own way, or it can write another program to
        /// upload the result by itself.
        ///
        /// This callback function does not have to be implemented.
        fn offchain_worker(_block_number: T::BlockNumber) {
            debug::info!("Entering off-chain worker");
        }

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
        /// Ordinary accounts can become pre-selected governance nodes by staking governance
        /// tokens.
        ///
        /// The dispatch origin for this call must be `Signed` by the transactor.
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn stake(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let number = with_transaction_result(|| Self::inner_stake(&who))?;
            Self::deposit_event(Event::<T>::Stake(who, number));
            Ok(().into())
        }

        /// If the account is not contested as a governance node, he can withdraw the pledged
        /// governance tokens by himself.
        ///
        /// The dispatch origin for this call must be `Signed` by the transactor.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn unstake(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let number = with_transaction_result(|| Self::inner_unstake(&who))?;
            Self::deposit_event(Event::<T>::UnStake(who, number));
            Ok(().into())
        }

        /// Slash the reserved of a given account.
        ///
        /// If a node commits evil and uploads some false results, the official or the community
        /// will directly punish all his pledges and cancel his label.
        ///
        /// The dispatch origin for this call is `root`.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn slash(origin: OriginFor<T>, who: T::AccountId) -> DispatchResultWithPostInfo {
            let _ = ensure_root(origin)?;
            let number = with_transaction_result(|| Self::inner_slash(&who))?;
            Self::deposit_event(Event::<T>::Slash(who, number));
            Ok(().into())
        }

        /// Upload proposal results
        ///
        /// The governance node uploads the final result of the proposal through an unsigned
        /// transaction with a signed payload
        ///
        /// This transaction does not need to be signed, but the payload must be signed
        #[pallet::weight(0)]
        pub fn upload_result(
            origin: OriginFor<T>,
            payload: Payload<T::Public, T::ProposalId, CurrencyIdOf<T>>,
            _signature: T::Signature,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_none(origin)?;
            let Payload {
                public,
                proposal_id,
                result,
            } = payload;
            Self::ensure_proposal_status(proposal_id, ProposalStatus::WaitingForResults)?;
            let who = public.into_account();
            with_transaction_result(|| Self::inner_upload_result(&who, proposal_id, result))?;
            Self::deposit_event(Event::<T>::UploadResult(who, proposal_id, result));
            Ok(().into())
        }

        /// Tag accounts that have pledged governance tokens
        ///
        /// Only the account that has been tagged can upload the results
        ///
        /// The dispatch origin for this call is `root`.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn tagging(origin: OriginFor<T>, account: T::AccountId) -> DispatchResultWithPostInfo {
            let _ = ensure_root(origin)?;
            ensure!(
                StakedAccount::<T>::contains_key(&account),
                Error::<T>::AccountNotStaked
            );
            AutonomyAccount::<T>::try_mutate(&account, |option| -> Result<(), DispatchError> {
                if let Some(_) = option {
                    Err(Error::<T>::AccountHasTagged)?
                } else {
                    *option = Some(());
                }
                Ok(())
            })?;
            Self::deposit_event(Event::<T>::Tagging(account));
            Ok(().into())
        }

        /// Delete the label, in some cases it is necessary to cancel the label of some accounts
        ///
        /// The dispatch origin for this call is `root`.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn untagging(
            origin: OriginFor<T>,
            account: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_root(origin)?;
            AutonomyAccount::<T>::try_mutate_exists(
                &account,
                |option| -> Result<(), DispatchError> {
                    if let Some(_) = option {
                        *option = None;
                    } else {
                        Err(Error::<T>::AccountNotTagged)?
                    }
                    Ok(())
                },
            )?;
            Self::deposit_event(Event::<T>::Untagging(account));
            Ok(().into())
        }

        /// Set the minimum stake amount
        ///
        /// The dispatch origin for this call is `root`.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_minimal_number(
            origin: OriginFor<T>,
            number: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_root(origin)?;
            MinimalStakeNumber::<T>::set(Some(number));
            Self::deposit_event(Event::<T>::SetMinimalNumber(number));
            Ok(().into())
        }

        /// Set the publicity interval
        ///
        /// The dispatch origin for this call is `root`.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_publicity_interval(
            origin: OriginFor<T>,
            interval: MomentOf<T>,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_root(origin)?;
            PublicityInterval::<T>::set(Some(interval.into()));
            Self::deposit_event(Event::<T>::SetPublicityInterval(interval));
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
    pub fn begin_block(_: T::BlockNumber) -> Result<Weight, DispatchError> {
        let max_id = T::Pool::max_proposal_id();
        let mut index: ProposalIdOf<T> = Zero::zero();
        let now = <TimeOf<T> as Time>::now();
        let interval = PublicityInterval::<T>::get().unwrap_or_else(Zero::zero);
        loop {
            if index >= max_id {
                break;
            }

            if let Ok(_) = with_transaction_result(|| Self::change_state(index, interval, now)) {}

            index = index
                .checked_add(&One::one())
                .ok_or(Error::<T>::ProposalIdOverflow)?;
        }
        Ok(0)
    }

    fn change_state(
        index: ProposalIdOf<T>,
        interval: MomentOf<T>,
        now: MomentOf<T>,
    ) -> Result<(), DispatchError> {
        let state = T::Pool::get_proposal_state(index)?;
        let time = T::Pool::proposal_announcement_time(index)?;
        if let Some(val) = ProposalAnnouncement::<T>::get(index) {
            ensure!(
                state == ProposalStatus::ResultAnnouncement,
                Error::<T>::ProposalAbnormalState
            );
            if now - val > interval {
                Self::inner_merge_result(index)?;
            }
        } else {
            ensure!(
                state == ProposalStatus::WaitingForResults,
                Error::<T>::ProposalAbnormalState
            );
            if now - time > interval {
                let (id1, id2) = T::CouplePool::proposal_pair(index)?;
                let sum1 = StatisticalResults::<T>::get(index, id1).unwrap_or_else(Zero::zero);
                let sum2 = StatisticalResults::<T>::get(index, id2).unwrap_or_else(Zero::zero);
                ensure!(sum1 != sum2, Error::<T>::ResultIsEqual);
                ProposalAnnouncement::<T>::insert(index, now);
                T::Pool::set_proposal_state(index, ProposalStatus::ResultAnnouncement)?;
            }
        }
        Ok(())
    }

    fn ensure_proposal_status(
        proposal_id: T::ProposalId,
        state: ProposalStatus,
    ) -> Result<(), DispatchError> {
        let old_state = T::Pool::get_proposal_state(proposal_id)?;
        ensure!(old_state == state, Error::<T>::ProposalAbnormalState);
        Ok(())
    }

    fn unstake_and_untagged(
        who: &T::AccountId,
    ) -> Result<(CurrencyIdOf<T>, BalanceOf<T>), DispatchError> {
        let currency_id = T::StakeCurrencyId::get();
        let number = StakedAccount::<T>::try_mutate_exists(
            &who,
            |option_num| -> Result<BalanceOf<T>, DispatchError> {
                let num = option_num.ok_or(Error::<T>::AccountNotStaked)?;
                *option_num = None;
                Ok(num)
            },
        )?;
        AutonomyAccount::<T>::try_mutate_exists(&who, |option| -> Result<(), DispatchError> {
            if let Some(_) = option {
                *option = None;
            }
            Ok(())
        })?;
        Ok((currency_id, number))
    }

    fn ensure_proposal_optional_id(
        proposal_id: T::ProposalId,
        result: CurrencyIdOf<T>,
    ) -> Result<(), DispatchError> {
        let (id1, id2) = T::CouplePool::proposal_pair(proposal_id)?;
        ensure!(
            result == id1 || result == id2,
            Error::<T>::ProposalOptionNotCorrect
        );
        Ok(())
    }

    fn inner_stake(who: &T::AccountId) -> Result<BalanceOf<T>, DispatchError> {
        let currency_id = T::StakeCurrencyId::get();
        let number = MinimalStakeNumber::<T>::get().unwrap_or_else(Zero::zero);
        StakedAccount::<T>::try_mutate(&who, |option_num| -> Result<(), DispatchError> {
            match option_num {
                Some(_) => Err(Error::<T>::AccountAlreadyStaked)?,
                None => {
                    *option_num = Some(number);
                    Ok(())
                }
            }
        })?;
        T::Tokens::reserve(currency_id, &who, number)
    }

    fn inner_unstake(who: &T::AccountId) -> Result<BalanceOf<T>, DispatchError> {
        let (currency_id, number) = Self::unstake_and_untagged(&who)?;
        T::Tokens::unreserve(currency_id, &who, number)
    }

    fn inner_slash(who: &T::AccountId) -> Result<BalanceOf<T>, DispatchError> {
        let (currency_id, number) = Self::unstake_and_untagged(&who)?;
        T::Tokens::slash_reserved(currency_id, &who, number)
    }

    fn inner_upload_result(
        who: &T::AccountId,
        proposal_id: T::ProposalId,
        result: CurrencyIdOf<T>,
    ) -> Result<(), DispatchError> {
        ensure!(
            AutonomyAccount::<T>::contains_key(&who),
            Error::<T>::AccountNotStaked
        );
        Self::ensure_proposal_optional_id(proposal_id, result)?;
        TemporaryResults::<T>::try_mutate(
            proposal_id,
            &who,
            |option_id| -> Result<(), DispatchError> {
                match option_id {
                    Some(_) => Err(Error::<T>::AccountHasAlreadyUploaded)?,
                    None => {
                        *option_id = Some(result);
                        Ok(())
                    }
                }
            },
        )?;
        StatisticalResults::<T>::try_mutate(
            proposal_id,
            result,
            |option_sum| -> Result<(), DispatchError> {
                let mut sum = option_sum.unwrap_or_else(Zero::zero);
                sum = sum.checked_add(One::one()).ok_or(Error::<T>::Overflow)?;
                *option_sum = Some(sum);
                Ok(())
            },
        )
    }

    fn inner_merge_result(proposal_id: T::ProposalId) -> Result<CurrencyIdOf<T>, DispatchError> {
        let (id1, id2) = T::CouplePool::proposal_pair(proposal_id)?;
        let sum1 = StatisticalResults::<T>::get(proposal_id, id1).unwrap_or_else(Zero::zero);
        let sum2 = StatisticalResults::<T>::get(proposal_id, id2).unwrap_or_else(Zero::zero);
        ensure!(sum1 != sum2, Error::<T>::ResultIsEqual);
        let result = if sum1 > sum2 { id1 } else { id2 };
        T::CouplePool::set_proposal_result(proposal_id, result)?;
        Ok(result)
    }
}

/// impl `ValidateUnsigned` trait with valudate unsigned transaction
impl<T: Config> frame_support::unsigned::ValidateUnsigned for Module<T> {
    type Call = Call<T>;

    /// Validate unsigned call to this module.
    ///
    /// By default unsigned transactions are disallowed, but implementing the validator
    /// here we make sure that some particular calls (the ones produced by offchain worker)
    /// are being whitelisted and marked as valid.
    fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
        let valid_tx = |provide| {
            ValidTransaction::with_tag_prefix("autonomy")
                .priority(UNSIGNED_TXS_PRIORITY)
                .and_provides([&provide])
                .longevity(3)
                .propagate(true)
                .build()
        };

        match call {
            Call::upload_result(ref payload, ref signature) => {
                if !SignedPayload::<T>::verify::<T::AuthorityId>(payload, signature.clone()) {
                    return InvalidTransaction::BadProof.into();
                }
                valid_tx(b"upload_result".to_vec())
            }
            _ => InvalidTransaction::Call.into(),
        }
    }
}
