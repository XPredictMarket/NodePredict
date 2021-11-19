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
#![allow(clippy::unused_unit)]
#![allow(clippy::type_complexity)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub(crate) mod macros;

pub use pallet::*;

use frame_support::{
    dispatch::{DispatchError, Weight},
    ensure,
    traits::{Get, Time},
};
use frame_system::offchain::SignedPayload;
use sp_core::crypto::KeyTypeId;
use sp_runtime::{
    traits::{AccountIdConversion, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, One, Zero},
    transaction_validity::{
        InvalidTransaction, TransactionSource, TransactionValidity, ValidTransaction,
    },
};
use xpmrl_traits::{
    autonomy::Autonomy, couple::LiquidityCouple, pool::LiquidityPool, tokens::Tokens,
    ProposalStatus,
};
use xpmrl_utils::{with_transaction_result, storage_try_mutate};
use sp_std::collections::btree_map::BTreeMap;

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

    /// A custom type of authentication ID is used to verify that the payload is correctly signed.
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
    use sp_std::collections::btree_map::BTreeMap;

    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::Time};
    use frame_system::{offchain::*, pallet_prelude::*};
    use sp_runtime::{traits::*, ModuleId};
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
    pub struct Payload<Public, ProposalId, ResultId, Balance> {
        /// The id of the proposal that needs to upload the result
        pub proposal_id: ProposalId,
        /// The asset id of the proposal result
        ///
        /// The proposal option is a token, so here only the id of the corresponding token needs to be uploaded
        pub result: ResultId,
        /// Account for uploading results
        pub public: Public,
        /// Upload votes
        pub vote_num: Balance,
    }

    /// implament trait for payload
    /// make sure the payload can be signed and verify
    impl<T: Config> SignedPayload<T> for Payload<T::Public, T::ProposalId, CurrencyIdOf<T>, BalanceOf<T>> {
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
        /// A trait used to obtain public information about the proposal
        type Pool: LiquidityPool<Self>;
        /// A trait that operates on a proposal with two options
        type CouplePool: LiquidityCouple<Self>;

        /// The asset id of the governance token
        ///
        /// This ensures that we only accept the assets of this id as governance assets
        #[pallet::constant]
        type StakeCurrencyId: Get<CurrencyIdOf<Self>>;

        #[pallet::constant]
        type AutonomyId: Get<ModuleId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn staked_node)]
    pub type StakedNode<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, (BalanceOf<T>,bool),OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn staked_node_lock_total_num)]
    pub type StakedNodeLockTotalNum<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn staked_node_lock_num)]
    pub type StakedNodeLockNum<T: Config> =
        StorageDoubleMap<
        _, 
        Blake2_128Concat,
        T::ProposalId, 
        Twox64Concat,
        T::AccountId,  
        BalanceOf<T>, 
        OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn node_review_voting_status)]
    pub type NodeReviewVotingStatus<T: Config> = StorageDoubleMap<
        _, 
        Blake2_128Concat,
        T::ProposalId, 
        Twox64Concat,
        T::AccountId, 
        BTreeMap<bool, BalanceOf<T>>,
        OptionQuery>;
    
    #[pallet::storage]
    #[pallet::getter(fn review_voting_status)]
    pub type ReviewVotingStatus <T: Config> = StorageDoubleMap<
        _, 
        Blake2_128Concat, 
        T::ProposalId,
        Twox64Concat,
        bool,
        BalanceOf<T>, 
        OptionQuery>;    

    #[pallet::storage]
    #[pallet::getter(fn consent_flag)]
    pub type ConsentFlag<T: Config> =
        StorageMap<_, Blake2_128Concat, T::ProposalId, (), OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn opposition_flag)]
    pub type OppositionFlag <T: Config> =
        StorageMap<_, Blake2_128Concat, T::ProposalId, (), OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn review_equal_flag)]
    pub type ReviewEqualFlag<T: Config> =
        StorageMap<_, Blake2_128Concat, T::ProposalId, (), OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn review_flag)]
    pub type ReviewFlag<T: Config> =
        StorageMap<_, Blake2_128Concat, T::ProposalId, (),  OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn snap_shot)]
    pub type SnapShot<T: Config> = StorageDoubleMap<
        _, 
        Blake2_128Concat,
        T::AccountId, 
        Twox64Concat,
        u64,
        (T::BlockNumber, BalanceOf<T>),
        OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn snap_shot_num)]
    pub type SnapShotNum<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u64, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn report_asset_pool )]
    pub type ReportAssetPool<T: Config> =
        StorageMap<_, Blake2_128Concat, ProposalIdOf<T>, BalanceOf<T>,OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn node_result_voting_status)]
    pub type NodeResultVotingStatus<T: Config> = StorageDoubleMap<
        _, 
        Blake2_128Concat,
        T::ProposalId, 
        Twox64Concat,
        T::AccountId, 
        (CurrencyIdOf<T>, BalanceOf<T>),
        OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn result_voting_status)]
    pub type ResultVotingStatus <T: Config> = StorageDoubleMap<
        _, 
        Blake2_128Concat, 
        T::ProposalId,
        Twox64Concat,
        CurrencyIdOf<T>,  
        BalanceOf<T>, 
        OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn account_report_number)]
    pub type AccountReportNumber<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        ProposalIdOf<T>,
        Twox64Concat,
        T::AccountId,
        BalanceOf<T>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn report_success_flag )]
    pub type ReportSuccessFlag<T: Config> =
        StorageMap<_, Blake2_128Concat, ProposalIdOf<T>, (),  OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn slash_finish_flag )]
    pub type SlashFinishFlag<T: Config> =
        StorageMap<_, Blake2_128Concat, ProposalIdOf<T>, (), OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn account_slash_number )]
    pub type AccountSlashNumber<T: Config> = StorageDoubleMap<
        _, 
        Blake2_128Concat, 
        ProposalIdOf<T>,
        Twox64Concat,
        T::AccountId,
        BalanceOf<T>,
        OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn report_voting_status )]
    pub type ReportVotingStatus<T: Config> =
        StorageMap<_, Blake2_128Concat, ProposalIdOf<T>, BalanceOf<T>,OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn minimal_review_number)]
    pub type MinimalReviewNumber<T: Config> = StorageValue<_, BalanceOf<T>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn minimal_stake_number)]
    pub type MinimalStakeNumber<T: Config> = StorageValue<_, BalanceOf<T>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn minimal_report_number)]
    pub type MinimalReportNumber<T: Config> = StorageValue<_, BalanceOf<T>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn lock_ratio)]
    pub type LockRatio<T: Config> = StorageValue<_, BalanceOf<T>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn review_cycle)]
    pub type ReviewCycle<T: Config> = StorageValue<_, MomentOf<T>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn result_upload_cycle)]
    pub type ResultUploadCycle<T: Config> = StorageValue<_, MomentOf<T>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn publicity_period)]
    pub type PublicityPeriod<T: Config> = StorageValue<_, MomentOf<T>, OptionQuery>;
    
    #[pallet::storage]
    #[pallet::getter(fn review_delay)]
    pub type ReviewDelay<T: Config> = StorageMap<_, Blake2_128Concat, ProposalIdOf<T>, MomentOf<T>, OptionQuery>;
    
    #[pallet::storage]
    #[pallet::getter(fn upload_delay)]
    pub type UploadDelay<T: Config> = StorageMap<_, Blake2_128Concat, ProposalIdOf<T>, MomentOf<T>, OptionQuery>;
    
    #[pallet::storage]
    #[pallet::getter(fn report_delay)]
    pub type ReportDelay<T: Config> = StorageMap<_, Blake2_128Concat, ProposalIdOf<T>, MomentOf<T>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn result_announcement_time )]
    pub type ResultAnnouncementTime<T: Config> =
        StorageMap<_, Blake2_128Concat, ProposalIdOf<T>, MomentOf<T>, OptionQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub minimal_stake_number: BalanceOf<T>,
        pub minimal_review_number: BalanceOf<T>,
        pub minimal_report_number: BalanceOf<T>,
        pub review_cycle: u32,
        pub result_upload_cycle: u32,
        pub publicity_period: u32,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                minimal_stake_number: Zero::zero(),
                minimal_review_number: Zero::zero(),
                minimal_report_number: Zero::zero(),
                review_cycle: Zero::zero(),
                result_upload_cycle: Zero::zero(),
                publicity_period: Zero::zero(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            MinimalStakeNumber::<T>::set(Some(self.minimal_stake_number));
            MinimalReviewNumber::<T>::set(Some(self.minimal_review_number));
            MinimalReportNumber::<T>::set(Some(self.minimal_report_number));
            ReviewCycle::<T>::set(Some(self.review_cycle.into()));
            ResultUploadCycle::<T>::set(Some(self.result_upload_cycle.into()));
            PublicityPeriod::<T>::set(Some(self.publicity_period.into()));
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Account stake successfully.
        Stake(T::AccountId, BalanceOf<T>),
        /// Account unstake successfully.
        UnStake(T::AccountId, BalanceOf<T>),
        /// Review vote
        Review(T::AccountId, T::ProposalId, bool, BalanceOf<T>),
        /// Punish evil nodes.
        Slash(T::AccountId, T::ProposalId, BalanceOf<T>),
        /// Slash finish
        SlashFinsh(T::ProposalId),
        /// StakedNode uploaded result.
        UploadResult(T::AccountId, T::ProposalId, CurrencyIdOf<T>, BalanceOf<T>),
        /// Report proposal result
        Report(T::AccountId, ProposalIdOf<T>, BalanceOf<T>),
        /// Take out the tokens pledged by the report
        TakeOut(T::AccountId, ProposalIdOf<T>, BalanceOf<T>),
        /// The staked node unlocks the number of votes
        Unlock(T::AccountId, ProposalIdOf<T>, BalanceOf<T>),
        /// Set the minimum review amount
        SetMinimalReviewNumber(BalanceOf<T>),
        /// Set the minimum stake amount
        SetMinimalStakeNumber(BalanceOf<T>),
        /// Set the minimum report amount
        SetMinimalReportNumber(BalanceOf<T>),
        /// Set the lock ratio
        SetLockRatio(BalanceOf<T>),
        /// Set the review cycle
        SetReviewCycle(MomentOf<T>),
        /// Set the upload cycle
        SetUploadCycle(MomentOf<T>),
        /// Set the publicity period
        SetPublicityPeriod(MomentOf<T>),

    }

    #[pallet::error]
    pub enum Error<T> {
        /// Account not staked
        AccountNotStaked,
        /// The account has already uploaded the results, the same proposal cannot be uploaded
        /// again
        AccountHasAlreadyUploaded,
        /// The node did not upload results
        AccountNotUpload,
        /// The account did not upload the result
        AccountDidNotUploadResult,
        /// Value has been overflow
        Overflow,
        /// Proposal is not at the wait for result, unable to upload results
        ProposalAbnormalState,
        /// Incorrect proposal options
        ProposalOptionNotCorrect,
        /// Proposal ID overflow
        ProposalIdOverflow,
        /// The final count of all the options of the proposal is equal, and the final result
        /// cannot be obtained
        ResultIsEqual,
        /// Insufficient balance to report
        ReportInsufficientBalance,
        /// Attitude should be the same
        AttitudeNeedSame,
        /// The number of report votes is enough, and there is no need to continue to stake
        ProposalNotNeedSecond,
        /// Not a staked node
        NotAStakingNode,
        /// Report that the input pledge amount is 0
        ReportStakedNumberZero,
        /// Review input pledge amount is 0
        ReviewStakedNumberZero,
        /// The node has participated in a review vote
        NodeHasAlreadyReview,
        /// The user has already reported once
        AccountHasAlreadyReport,
        /// The minimum number of reports is not set
        MinimalReportNumberNotSet,
        /// Report unsuccessful
        ReportNotSuccess,
        /// Account has been punished
        AccountHasBeenSlashed,
        /// Slash has been completed
        SlashHasBeenCompleted,
        /// Snapshot re-entry
        SnapshotReEntry,
        /// The number of snapshots is not entered
        SnapshotNumNotEntry,
        /// The snapshot is not recorded
        SnapshotNotEntry,
        /// Not enough votes available
        InsufficientNumberOfVotes,
        /// Locked rate is not set
        LockRatioNotSet,
        /// Slash account errors
        SlashAccountError,
        /// Slash num errors
        SlashNumError,
        /// The upload result was reported, and the number of locked tickets 
        /// could not be withdrawn
        UploadResultWasReported,
        /// The number of locks is 0
        NoLockedQuantity,
        /// The proposal has not entered the publicity period
        ProposalHasNotEnteredThePublicityPeriod,
        /// Input ratio is too large
        InputRatioIsTooLarge
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
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(5, 2))]
        pub fn stake(origin: OriginFor<T>, stake_number: BalanceOf<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let number = with_transaction_result(|| Self::inner_stake(&who, stake_number))?;
            Self::deposit_event(Event::<T>::Stake(who, number));
            Ok(().into())
        }

        /// If the account is not contested as a governance node, he can withdraw the pledged
        /// governance tokens by himself.
        ///
        /// The dispatch origin for this call must be `Signed` by the transactor.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(6, 3))]
        pub fn unstake(origin: OriginFor<T>, unstake_number: BalanceOf<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let number = with_transaction_result(|| Self::inner_unstake(&who, unstake_number))?;
            Self::deposit_event(Event::<T>::UnStake(who, number));
            Ok(().into())
        }

        /// Review proposals in the original forecasting stage
        ///
        /// The governance node decides whether the proposal is approved by voting agree or against
        ///
        /// The dispatch origin for this call must be `Signed` by the transactor.       
        #[pallet::weight(10_000 + T::DbWeight::get().writes(5, 2))]
        pub fn review(origin: OriginFor<T>, vote_number: BalanceOf<T>, proposal_id: ProposalIdOf<T>, vote_type: bool) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::ensure_proposal_status(proposal_id, ProposalStatus::OriginalPrediction)?;
            let (_, node_flag) = StakedNode::<T>::get(&who).ok_or(Error::<T>::AccountNotStaked)?;
            ensure!(node_flag, Error::<T>::NotAStakingNode);
            ensure!(NodeReviewVotingStatus::<T>::get(proposal_id, &who).is_none(), Error::<T>::NodeHasAlreadyReview);
            ensure!(
                vote_number != Zero::zero() ,
                Error::<T>::ReviewStakedNumberZero
            );
            with_transaction_result(|| Self::inner_review(&who, proposal_id, vote_number, vote_type))?;
            Self::deposit_event(Event::<T>::Review(who, proposal_id, vote_type, vote_number));
            Ok(().into())
        }

        /// Slash the reserved of a given account.
        ///
        /// If a node commits evil and uploads some false results, the official or the community
        /// will directly punish him for the amount of votes he pledged in this proposal.
        ///
        /// The dispatch origin for this call is `root`.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(9, 2))]
        pub fn slash(
            origin: OriginFor<T>, 
            who: T::AccountId,
            proposal_id: ProposalIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_root(origin)?;
            Self::ensure_proposal_status(proposal_id, ProposalStatus::End)?;
            ensure!(
                ReportSuccessFlag::<T>::get(proposal_id) == Some(()),
                Error::<T>::ReportNotSuccess,
            );
            ensure!(
                AccountSlashNumber::<T>::get(proposal_id, &who) == None,
                Error::<T>::AccountHasBeenSlashed,
            );
            ensure!(
                SlashFinishFlag::<T>::get(proposal_id) == None,
                Error::<T>::SlashHasBeenCompleted,
            );
            let number = with_transaction_result(|| Self::inner_slash(&who, proposal_id))?;
            Self::deposit_event(Event::<T>::Slash(who, proposal_id, number));
            Ok(().into())
        }

        /// Set slash completion flag
        ///
        /// When all malicious nodes are slashed, this flag is set to completed
        ///
        /// The dispatch origin for this call is `root`.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(9, 1))]
        pub fn slash_finish(
            origin: OriginFor<T>, 
            proposal_id: ProposalIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_root(origin)?;
            with_transaction_result(|| Self::inner_slash_finish(proposal_id))?;
            Self::deposit_event(Event::<T>::SlashFinsh(proposal_id));
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
            payload: Payload<T::Public, T::ProposalId, CurrencyIdOf<T>, BalanceOf<T>>,
            _signature: T::Signature,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_none(origin)?;
            let Payload {
                public,
                proposal_id,
                result,
                vote_num,
            } = payload;
            Self::ensure_proposal_status(proposal_id, ProposalStatus::WaitingForResults)?;
            let who = public.into_account();
            with_transaction_result(|| Self::inner_upload_result(&who, proposal_id, result, vote_num))?;
            Self::deposit_event(Event::<T>::UploadResult(who, proposal_id, result, vote_num));
            Ok(().into())
        }

        /// Users can report proposals with incorrect results. If successful, 
        /// all nodes that cast incorrect results will be punished.
        ///
        /// The dispatch origin for this call must be `Signed` by the transactor.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(4, 2))]
        pub fn report(
            origin: OriginFor<T>,
            proposal_id: ProposalIdOf<T>,
            vote_num: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let state = T::Pool::get_proposal_state(proposal_id)?;
            ensure!(
                state == ProposalStatus::ResultAnnouncement,
                Error::<T>::ProposalAbnormalState,
            );
            with_transaction_result(|| Self::inner_report(&who, proposal_id, vote_num))?;
            Self::deposit_event(Event::<T>::Report(who, proposal_id, vote_num));
            Ok(().into())
        }

        /// After the report is completed, users who report successfully can take 
        /// out the staked tokens
        ///
        /// The dispatch origin for this call must be `Signed` by the transactor.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(5, 1))]
        pub fn take_out(
            origin: OriginFor<T>,
            proposal_id: ProposalIdOf<T>
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::ensure_proposal_status(proposal_id, ProposalStatus::End)?;
            let number = with_transaction_result( || Self::inner_take_out(&who, proposal_id))?;
            Self::deposit_event(Event::<T>::TakeOut(who, proposal_id, number));
            Ok(().into())
        }

        /// After the proposal is over, the governance node can unlock the number of 
        /// votes that were locked when uploading the results
        ///
        /// The dispatch origin for this call must be `Signed` by the transactor.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(3, 2))]
        pub fn unlock(
            origin: OriginFor<T>,
            proposal_id: ProposalIdOf<T>
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::ensure_proposal_status(proposal_id, ProposalStatus::End)?;
            let number = with_transaction_result( || Self::inner_unlock(&who, proposal_id))?;
            Self::deposit_event(Event::<T>::Unlock(who, proposal_id, number));
            Ok(().into())
        }

        /// Set the minimum number of reviews
        ///
        /// The dispatch origin for this call is `root`.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_minimal_review_number(
            origin: OriginFor<T>,
            number: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_root(origin)?;
            MinimalReviewNumber::<T>::set(Some(number));
            Self::deposit_event(Event::<T>::SetMinimalReviewNumber(number));
            Ok(().into())
        }

        /// Set the minimum number of stake
        ///
        /// The dispatch origin for this call is `root`.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_minimal_stake_number(
            origin: OriginFor<T>,
            number: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_root(origin)?;
            MinimalStakeNumber::<T>::set(Some(number));
            Self::deposit_event(Event::<T>::SetMinimalStakeNumber(number));
            Ok(().into())
        }

        /// Set the minimum number of report
        ///
        /// The dispatch origin for this call is `root`.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_minimal_report_number(
            origin: OriginFor<T>,
            number: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_root(origin)?;
            MinimalReportNumber::<T>::set(Some(number));
            Self::deposit_event(Event::<T>::SetMinimalReportNumber(number));
            Ok(().into())
        }

        /// Set the minimum number of lockratio
        ///
        /// The dispatch origin for this call is `root`.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_lock_ratio(
            origin: OriginFor<T>,
            number: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_root(origin)?;
            ensure!(number < 100u32.into(),Error::<T>::InputRatioIsTooLarge);
            LockRatio::<T>::set(Some(number));
            Self::deposit_event(Event::<T>::SetLockRatio(number));
            Ok(().into())
        }

        /// Set the publicity interval
        ///
        /// The dispatch origin for this call is `root`.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_review_cycle(
            origin: OriginFor<T>,
            interval: MomentOf<T>,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_root(origin)?;
            ReviewCycle::<T>::set(Some(interval));
            Self::deposit_event(Event::<T>::SetReviewCycle(interval));
            Ok(().into())
        }

        /// Set the upload cycle
        ///
        /// The dispatch origin for this call is `root`.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_upload_cycle(
            origin: OriginFor<T>,
            interval: MomentOf<T>,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_root(origin)?;
            ResultUploadCycle::<T>::set(Some(interval));
            Self::deposit_event(Event::<T>::SetUploadCycle(interval));
            Ok(().into())
        }

        /// Set the publicity period
        ///
        /// The dispatch origin for this call is `root`.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_publicity_period(
            origin: OriginFor<T>,
            interval: MomentOf<T>,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_root(origin)?;
            PublicityPeriod::<T>::set(Some(interval));
            Self::deposit_event(Event::<T>::SetPublicityPeriod(interval));
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
        T::AutonomyId::get().into_account()
    }

    fn begin_block(_: T::BlockNumber) -> Result<Weight, DispatchError> {
        let max_id = T::Pool::max_proposal_id();
        let mut index: ProposalIdOf<T> = Zero::zero();
        let now = <TimeOf<T> as Time>::now();
        loop {
            
            if index >= max_id {
                break;
            }
            let result = with_transaction_result(|| {
                
                Self::change_state(index, now)
            });
            if result.is_ok() {}
            index = index
                .checked_add(&One::one())
                .ok_or(Error::<T>::ProposalIdOverflow)?;
        }
        Ok(0)
    }

    fn change_state(
        index: ProposalIdOf<T>,
        now: MomentOf<T>,
    ) -> Result<(), DispatchError> {
        let state = T::Pool::get_proposal_state(index)?;
        let create_time = T::Pool::proposal_create_time(index)?;
        let close_time = T::Pool::proposal_close_time(index)?;
        let review_time = ReviewCycle::<T>::get().unwrap_or_else(Zero::zero);
        let upload_time = ResultUploadCycle::<T>::get().unwrap_or_else(Zero::zero);
        let report_time = PublicityPeriod::<T>::get().unwrap_or_else(Zero::zero);
        match state{
            ProposalStatus::OriginalPrediction => {
                let diff = now.checked_sub(&create_time).unwrap_or_else(Zero::zero);
                let delay_num = ReviewDelay::<T>::get(index).unwrap_or_else(Zero::zero);
                let delay = delay_num.checked_mul(&review_time).ok_or(Error::<T>::Overflow)?;
                let delay = delay.checked_add(&review_time).ok_or(Error::<T>::Overflow)?;
                if diff >= delay{
                    if ReviewEqualFlag::<T>::get(index).is_some(){
                            let new_v = delay_num.checked_add(&One::one()).ok_or(Error::<T>::Overflow)?;
                            ReviewDelay::<T>::insert(index, new_v);
                    }
                    else{
                        let v1 = ReviewVotingStatus::<T>::get(index, true).unwrap_or_else(Zero::zero);
                        let v2 = ReviewVotingStatus::<T>::get(index, false).unwrap_or_else(Zero::zero);
                        if v1 > v2{
                            T::Pool::set_proposal_state(index, ProposalStatus::FormalPrediction)?;
                        }
                        else{
                            T::Pool::set_proposal_state(index, ProposalStatus::End)?;
                        }
                    }
                }
            }
            ProposalStatus::WaitingForResults => {
                let (p1, p2) = T::CouplePool::proposal_pair(index)?;
                let p1_balance = ResultVotingStatus::<T>::get(index, p1).unwrap_or_else(Zero::zero);
                let p2_balance = ResultVotingStatus::<T>::get(index, p2).unwrap_or_else(Zero::zero);
                let diff = now.checked_sub(&close_time).unwrap_or_else(Zero::zero);
                let delay_num = UploadDelay::<T>::get(index).unwrap_or_else(Zero::zero);
                let delay = delay_num.checked_mul(&upload_time).ok_or(Error::<T>::Overflow)?;
                let delay = delay.checked_add(&upload_time).ok_or(Error::<T>::Overflow)?;
                if diff >= delay{
                    if (p1_balance == p2_balance) && (p1_balance != Zero::zero()){
                        let new_v = delay_num.checked_add(&One::one()).ok_or(Error::<T>::Overflow)?;
                        UploadDelay::<T>::insert(index, new_v);
                    }
                    else if p1_balance > p2_balance{
                        T::CouplePool::set_proposal_result(index, p1)?;
                        T::Pool::set_proposal_state(index, ProposalStatus::ResultAnnouncement)?;
                        ResultAnnouncementTime::<T>::insert(index, now);
                    }
                    else if p1_balance < p2_balance{
                        T::CouplePool::set_proposal_result(index, p2)?;
                        T::Pool::set_proposal_state(index, ProposalStatus::ResultAnnouncement)?;
                        ResultAnnouncementTime::<T>::insert(index, now);
                    }
                }
            }
            ProposalStatus::ResultAnnouncement => {
                let announcement_time = ResultAnnouncementTime::<T>::get(index)
                    .ok_or(Error::<T>::ProposalHasNotEnteredThePublicityPeriod)?;
                let diff = now.checked_sub(&announcement_time).unwrap_or_else(Zero::zero);
                if diff >= report_time{
                        T::Pool::set_proposal_state(index, ProposalStatus::End)?;
                }
                
            }
            _ => {

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

    fn inner_review(
        who: &T::AccountId, 
        proposal_id: ProposalIdOf<T>, 
        vote_number: BalanceOf<T>, 
        vote_type: bool) 
        -> Result<(), DispatchError> {
        let usable_num = Self::inner_get_snapshot_usable_num(who)?;
        ensure!(vote_number <= usable_num, Error::<T>::InsufficientNumberOfVotes);
        let minimal_number = MinimalReviewNumber::<T>::get().unwrap_or_else(Zero::zero);
        NodeReviewVotingStatus::<T>::try_mutate(
            proposal_id,
            &who,
            |option| -> Result<(), DispatchError> {
                match option {
                    Some(_) => {
                        Err(Error::<T>::NodeHasAlreadyReview.into())
                    },
                    None => {
                        let mut map = BTreeMap::<bool, BalanceOf<T>>::new();
                        map.insert(vote_type, vote_number);
                        *option = Some(map);
                        Ok(())
                    }
                }
            }
        )?;
        ReviewVotingStatus::<T>::try_mutate(
            proposal_id,
            vote_type,
            |option| -> Result<(), DispatchError> {
                let opposite_v = ReviewVotingStatus::<T>::get(proposal_id, !vote_type).unwrap_or_else(Zero::zero);
                let new_v = match option {
                    Some(v) => {
                        v.checked_add(&vote_number).ok_or(Error::<T>::Overflow)?
                    },
                    None => {
                        vote_number
                    }
                };
                if  new_v >= minimal_number  && 
                    vote_type  && 
                    ConsentFlag::<T>::get(proposal_id) == None
                {
                    let v = Some(());
                    flag_try_mutate!(ConsentFlag, proposal_id, v )?;
                }
                else if new_v >= minimal_number  && 
                    !vote_type && 
                    OppositionFlag::<T>::get(proposal_id) == None
                {
                    let v = Some(());
                    flag_try_mutate!(OppositionFlag, proposal_id, v )?;
                }
                if new_v >= minimal_number  && new_v >= opposite_v
                {
                    if vote_type && ReviewFlag::<T>::get(proposal_id) == None{
                        let v = Some(());
                        flag_try_mutate!(ReviewFlag, proposal_id, v )?;
                    }
                    else if !vote_type && ReviewFlag::<T>::get(proposal_id) == Some(()){
                        let v:Option<T> = None;
                        flag_try_mutate!(ReviewFlag, proposal_id, v )?;
                    }
                    if new_v == opposite_v{
                        let v = Some(());
                        flag_try_mutate!(ReviewEqualFlag, proposal_id, v )?;
                    }
                    else{
                        let v:Option<T> = None;
                        flag_try_mutate!(ReviewEqualFlag, proposal_id, v )?;
                    }
                }
                *option = Some(new_v);
                Ok(())
            }
        )?;
        Ok(())
    }

    fn inner_stake(who: &T::AccountId, stake_number: BalanceOf<T>) -> Result<BalanceOf<T>, DispatchError> {
        let currency_id = T::StakeCurrencyId::get();
        let block_number = frame_system::Pallet::<T>::block_number();
        let minimal_number = MinimalStakeNumber::<T>::get().unwrap_or_else(Zero::zero);
        let snap_shot_num = Self::inner_snapshot_num_update(who)?;
        let last_snap_shot_num = snap_shot_num - 1;
        let (_, balance) = SnapShot::<T>::get(&who, last_snap_shot_num).unwrap_or(
            (Zero::zero(), Zero::zero()));
        SnapShot::<T>::try_mutate(
            &who, 
            snap_shot_num,
            |option_num| -> Result<(), DispatchError> {
                match option_num {
                    Some(_) => {
                        Err(Error::<T>::SnapshotReEntry.into())
                    },
                    None => {
                        let new_balance = balance.checked_add(&stake_number).ok_or(Error::<T>::Overflow)?;
                        *option_num = Some((block_number, new_balance));
                        Ok(())
                    }
                }
            }
        )?;
        StakedNode::<T>::try_mutate(
            &who, 
            |option_num| -> Result<(), DispatchError> {
                let (new_balance, new_node_flag) = match option_num {
                    Some(tuple) => {
                        let (old_balance, _) = tuple;
                        let new_balance = old_balance.checked_add(&stake_number).ok_or(Error::<T>::Overflow)?;
                        (new_balance, Self::inner_update_stake_node(new_balance, minimal_number))
                    },
                    None => {
                        (stake_number, Self::inner_update_stake_node(stake_number, minimal_number))
                    }
                };
                *option_num = Some((new_balance, new_node_flag));
                Ok(())
            }
        )?;
        <TokensOf<T> as Tokens<T::AccountId>>::reserve(currency_id, who, stake_number)
    }

    fn inner_unstake(
        who: &T::AccountId,
        unstake_number: BalanceOf<T>
    ) -> Result<BalanceOf<T>, DispatchError> {
        let currency_id = T::StakeCurrencyId::get();
        let block_number = frame_system::Pallet::<T>::block_number();
        let minimal_number = MinimalStakeNumber::<T>::get().unwrap_or_else(Zero::zero);
        let snap_shot_num = Self::inner_snapshot_num_update(who)?;
        let last_snap_shot_num = snap_shot_num - 1;
        let (_, balance) = SnapShot::<T>::get(&who, last_snap_shot_num).unwrap_or(
            (Zero::zero(), Zero::zero()) );
        SnapShot::<T>::try_mutate(
            &who, 
            snap_shot_num,
            |option_num| -> Result<(), DispatchError> {
                match option_num {
                    Some(_) => {
                        Err(Error::<T>::SnapshotReEntry.into())
                    },
                    None => {
                        let new_balance = balance.checked_sub(&unstake_number).unwrap_or_else(Zero::zero);
                        *option_num = Some((block_number, new_balance));
                        Ok(())
                    }
                }
            }
        )?;
        let actual_number = StakedNode::<T>::try_mutate_exists(
            &who,
            |option_num| -> Result<BalanceOf<T>, DispatchError> {
                match option_num {
                    Some(tuple) => {
                        let (old_balance, _) = tuple;
                        let new_balance = old_balance.checked_sub(&unstake_number).unwrap_or_else(Zero::zero);
                        let actual_number = old_balance.checked_sub(&new_balance).ok_or(Error::<T>::Overflow)?;
                        *option_num = Some((new_balance, Self::inner_update_stake_node(new_balance, minimal_number)));
                        Ok(<TokensOf<T> as Tokens<T::AccountId>>::unreserve(currency_id, who, actual_number)?)
                    }
                    None => {
                        Err(Error::<T>::AccountNotStaked.into())
                    }
                }
                
            }

        )?;
  
        Ok(actual_number)
    }

    fn inner_update_stake_node(current_number: BalanceOf<T>, minimal_number: BalanceOf<T>) -> bool{
        current_number >= minimal_number
    }

    fn inner_get_snapshot_usable_num(who: &T::AccountId) -> Result<BalanceOf<T>, DispatchError> {
        let snapshot_num = SnapShotNum::<T>::get(&who).ok_or(Error::<T>::SnapshotNumNotEntry)?;
        let (_, balance) =  SnapShot::<T>::get(&who, snapshot_num).ok_or(Error::<T>::SnapshotNotEntry)?;
        let lock_balance = StakedNodeLockTotalNum::<T>::get(&who).unwrap_or_else(Zero::zero);
        let usable_balance = balance.checked_sub(&lock_balance).ok_or(Error::<T>::Overflow)?;
        Ok(usable_balance)
    }

    fn inner_snapshot_num_update(who: &T::AccountId) -> Result<u64, DispatchError> {
        SnapShotNum::<T>::try_mutate(
            &who,
            |optional| -> Result<u64, DispatchError> {
                let mut sum = optional.unwrap_or_else(Zero::zero);
                sum = sum.checked_add(One::one()).ok_or(Error::<T>::Overflow)?;
                *optional = Some(sum);
                Ok(sum)
            }
        )
    }

    fn inner_slash(who: &T::AccountId, proposal_id: ProposalIdOf<T>) -> Result<BalanceOf<T>, DispatchError> {
        let currency_stake_id = T::StakeCurrencyId::get();
        let (currency_id, vote_number) = NodeResultVotingStatus::<T>::get(proposal_id, who).ok_or(Error::<T>::AccountDidNotUploadResult)?;
        let result = T::CouplePool::get_proposal_result(proposal_id)?;
        ensure!(result == currency_id, Error::<T>::SlashAccountError);
        let lock_ratio = LockRatio::<T>::get().ok_or(Error::<T>::LockRatioNotSet)?;
        let slash_number = vote_number.checked_mul(&lock_ratio).unwrap_or_else(Zero::zero);
        let slash_number = slash_number.checked_div(&100u32.into()).unwrap_or_else(Zero::zero);
        let unstake_number = with_transaction_result(|| Self::inner_unstake(who, slash_number))?;
        let autonomy_account = Self::module_account();
        let _ = <TokensOf<T> as Tokens<T::AccountId>>::transfer(currency_stake_id, who, &autonomy_account, unstake_number)?;
        ReportAssetPool::<T>::try_mutate_exists(
            proposal_id,
            |option_num| -> Result<(), DispatchError> {
                let num = match option_num {
                    Some(old_balance) => {
                        old_balance.checked_add(&slash_number).ok_or(Error::<T>::Overflow)?
                    }
                    None => {
                        slash_number
                    }
                };
                *option_num = Some(num);
                Ok(())
            }
        )?;
        AccountSlashNumber::<T>::try_mutate_exists(
            proposal_id,
            &who,
            |option_num| -> Result<(), DispatchError> {
                let num = match option_num {
                    Some(old_balance) => {
                        old_balance.checked_add(&slash_number).ok_or(Error::<T>::Overflow)?
                    }
                    None => {
                        slash_number
                    }
                };
                *option_num = Some(num);
                Ok(())
            }
        )?;
        Ok(slash_number)
    }

    fn inner_slash_finish(proposal_id: ProposalIdOf<T>) -> Result<(), DispatchError> {
        ensure!(
            ReportSuccessFlag::<T>::get(proposal_id) == Some(()),
            Error::<T>::ReportNotSuccess,
        );
        ensure!(
            SlashFinishFlag::<T>::get(proposal_id) == None,
            Error::<T>::SlashHasBeenCompleted,
        );
        let (id1, id2) = T::CouplePool::proposal_pair(proposal_id)?;
        let pool_num = ReportAssetPool::<T>::get(proposal_id).unwrap_or_else(Zero::zero);
        let vote_num1 = ResultVotingStatus::<T>::get(proposal_id, id1).unwrap_or_else(Zero::zero);
        let vote_num2 = ResultVotingStatus::<T>::get(proposal_id, id2).unwrap_or_else(Zero::zero);
        ensure!(vote_num1 != vote_num2, Error::<T>::ResultIsEqual);
        let result_id = T::CouplePool::get_proposal_result(proposal_id)?;
        let result_num = ResultVotingStatus::<T>::get(proposal_id, result_id).unwrap_or_else(Zero::zero);
        let lock_ratio = LockRatio::<T>::get().ok_or(Error::<T>::LockRatioNotSet)?;
        let result_num = result_num.checked_mul(&lock_ratio).ok_or(Error::<T>::Overflow)?;
        let result_num = result_num.checked_div(&100u32.into()).ok_or(Error::<T>::Overflow)?;
        ensure!(result_num == pool_num, Error::<T>::SlashNumError);
        SlashFinishFlag::<T>::try_mutate(
            proposal_id,
            |optional| -> Result<(), DispatchError> {
                match optional {
                    Some(_) => {
                    },
                    None => *optional = Some(()),
                };
                Ok(())
            },
        )?;
        Self::inner_swap_result(proposal_id)
    }

    fn inner_upload_result(
        who: &T::AccountId,
        proposal_id: T::ProposalId,
        result: CurrencyIdOf<T>,
        vote_num: BalanceOf<T>,
    ) -> Result<(), DispatchError> {
        let (_, node_flag) = StakedNode::<T>::get(&who).ok_or(Error::<T>::AccountNotStaked)?;
        ensure!(
            node_flag,
            Error::<T>::NotAStakingNode
        );
        Self::ensure_proposal_optional_id(proposal_id, result)?;
        let usable_balance = Self::inner_get_snapshot_usable_num(who)?;
        ensure!(usable_balance >= vote_num, Error::<T>::InsufficientNumberOfVotes);
        let lock_ratio = LockRatio::<T>::get().ok_or(Error::<T>::LockRatioNotSet)?;
        let lock_num = vote_num.checked_mul(&lock_ratio).ok_or(Error::<T>::Overflow)?;
        let lock_num = lock_num.checked_div(&100u32.into()).ok_or(Error::<T>::Overflow)?;
        StakedNodeLockTotalNum::<T>::try_mutate(
            &who,
            |optional| -> Result<(), DispatchError> {
                let new_lock_balance = match optional{
                    Some(lock_balance) => {
                        lock_balance.checked_add(&lock_num).ok_or(Error::<T>::Overflow)?
                    }
                    None => {
                        lock_num
                    }
                };
                *optional = Some(new_lock_balance);
                Ok(())
            }
        )?;
        StakedNodeLockNum::<T>::try_mutate(
            proposal_id,
            &who,
            |optional| -> Result<(), DispatchError> {
                match optional{
                    Some(_) => {
                        Err(Error::<T>::AccountHasAlreadyUploaded.into())
                    }
                    None => {
                        *optional = Some(lock_num);
                        Ok(())
                    }
                }
            }
        )?;
        NodeResultVotingStatus::<T>::try_mutate(
            proposal_id,
            &who,
            |option_id| -> Result<(), DispatchError> {
                match option_id {
                    Some(_) => Err(Error::<T>::AccountHasAlreadyUploaded.into()),
                    None => {
                        *option_id = Some((result, vote_num));
                        Ok(())
                    }
                }
            },
        )?;
        ResultVotingStatus::<T>::try_mutate(
            proposal_id,
            result,
            |option_sum| -> Result<(), DispatchError> {
                let mut sum = option_sum.unwrap_or_else(Zero::zero);
                sum = sum.checked_add(&vote_num).ok_or(Error::<T>::Overflow)?;
                *option_sum = Some(sum);
                Ok(())
            },
        )?;
        Ok(())
    }

    fn inner_take_out(
        who: &T::AccountId,
        proposal_id: T::ProposalId
    ) -> Result<BalanceOf<T>, DispatchError> {
        let currency_id = T::StakeCurrencyId::get();
        let autonomy_account = Self::module_account();
        let report_number =  AccountReportNumber::<T>::try_mutate_exists(
            proposal_id,
            &who,
            |optional| -> Result<BalanceOf<T>, DispatchError> {
                match optional {
                    Some(balance) => {
                        let v = *balance;
                        *optional = None;
                        Ok(v)
                    },
                    None => Err(Error::<T>::ReportInsufficientBalance.into()),
                }
            },
        )?;
        let reward_num = match ReportSuccessFlag::<T>::get(proposal_id){
            Some(_) => {
                let base: BalanceOf<T> = 100u32.into();
                let report_pool_num = ReportAssetPool::<T>::get(proposal_id).unwrap_or_else(Zero::zero);
                let total_report_num = ReportVotingStatus::<T>::get(proposal_id).unwrap_or_else(Zero::zero);
                let report_number = report_number.checked_mul(&base).ok_or(Error::<T>::Overflow)?;
                let reward_ratio = report_number.checked_div(&total_report_num).ok_or(Error::<T>::Overflow)?;
                let reward_num = report_pool_num.checked_mul(&reward_ratio).ok_or(Error::<T>::Overflow)?;
                reward_num.checked_div(&base).ok_or(Error::<T>::Overflow)?
            }
            None => {
                Zero::zero()
            }
        };
        <TokensOf<T> as Tokens<T::AccountId>>::unreserve(currency_id, who, report_number)?;
        <TokensOf<T> as Tokens<T::AccountId>>::transfer(currency_id, &autonomy_account, who, reward_num)
    }

    fn inner_unlock(
        who: &T::AccountId,
        proposal_id: T::ProposalId
    ) -> Result<BalanceOf<T>, DispatchError> {
        ensure!( ReportSuccessFlag::<T>::get(proposal_id) == None, Error::<T>::UploadResultWasReported);
        let lock_num = StakedNodeLockNum::<T>::try_mutate(
            proposal_id,
            who, 
            |option_num| -> Result<BalanceOf<T>, DispatchError> {
                match option_num {
                    Some(lock_num) => {
                        let v = *lock_num;
                        *option_num = Some(Zero::zero());
                        Ok(v)
                    },
                    None => {
                        Err(Error::<T>::NoLockedQuantity.into())
                    }
                }
            }
        )?;
        StakedNodeLockTotalNum::<T>::try_mutate(
            who,
            |option_num| -> Result<(), DispatchError> {
                match option_num{
                    Some(old_balance) => {
                        let new_balance = old_balance.checked_sub(&lock_num).ok_or(Error::<T>::Overflow)?;
                        *option_num = Some(new_balance);
                        Ok(())
                    }
                    None =>{
                        Err(Error::<T>::NoLockedQuantity.into())
                    }
                }
            }
        )?;
        Ok(lock_num)
    }

    fn inner_swap_result(
        proposal_id: T::ProposalId,
    ) -> Result<(), DispatchError> {
        let (id1, id2) = T::CouplePool::proposal_pair(proposal_id)?;
        let result = T::CouplePool::get_proposal_result(proposal_id)?;
        T::CouplePool::set_proposal_result(proposal_id, [id1, id2][(result == id1) as usize])?;
        T::Pool::set_proposal_state(proposal_id, ProposalStatus::End)?;
        Ok(())
    }

    fn inner_report(
        who: &T::AccountId,
        proposal_id: ProposalIdOf<T>,
        report_num: BalanceOf<T>,
    ) -> Result<(), DispatchError> {
        let currency_id = T::StakeCurrencyId::get();
        ensure!(report_num != Zero::zero(), Error::<T>::ReportStakedNumberZero);
        let minimal_report_number = MinimalReportNumber::<T>::get().ok_or(Error::<T>::MinimalReportNumberNotSet)?;
        AccountReportNumber::<T>::try_mutate(
            proposal_id,
            &who,
            |optional| -> Result<(), DispatchError> {
                match optional {
                    Some(_) => Err(Error::<T>::AccountHasAlreadyReport.into()),
                    None => {
                        *optional = Some(report_num);
                        Ok(())
                    }
                }
            },
        )?;
        ReportVotingStatus::<T>::try_mutate(
            proposal_id,
            |optional| -> Result<(), DispatchError> {
                match optional {
                    Some(old_balance) =>{
                        let new_balance = old_balance.checked_add(&report_num).ok_or(Error::<T>::Overflow)?;
                        if new_balance >= minimal_report_number{
                            ReportSuccessFlag::<T>::try_mutate(
                                proposal_id,
                                |optional| -> Result<(), DispatchError> {
                                    match optional {
                                        Some(_) => {
                                        }
                                        None =>{
                                            *optional = Some(());
                                        }
                                    }
                                    Ok(())
                                },
                            )?;
                        }
                        *optional = Some(new_balance);
                    } ,
                    None => {
                        *optional = Some(report_num);
                    }
                }
                Ok(())
            },
        )?;
        <TokensOf<T> as Tokens<T::AccountId>>::reserve(currency_id, who, report_num)?;
        Ok(())
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

impl<T: Config> Autonomy<T> for Pallet<T> {
    fn temporary_results(
        proposal_id: ProposalIdOf<T>,
        who: &T::AccountId,
    ) -> Result<CurrencyIdOf<T>, DispatchError> {
        match NodeResultVotingStatus::<T>::get(proposal_id, &who) {
            Some(tuple) => {
                let (currency_id, _) = tuple;
                Ok(currency_id)
            },
                
            None => Err(Error::<T>::AccountNotUpload.into()),
        }
    }

    fn statistical_results(
        proposal_id: ProposalIdOf<T>,
        currency_id: CurrencyIdOf<T>,
    ) -> BalanceOf<T> {
        ResultVotingStatus::<T>::get(proposal_id, currency_id).unwrap_or_else(Zero::zero)
    }
}
