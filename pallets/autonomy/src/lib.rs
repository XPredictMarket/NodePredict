#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use frame_support::{dispatch::DispatchError, ensure, traits::Get};
use frame_system::{offchain::SignedPayload, RawOrigin};
use sp_core::crypto::KeyTypeId;
use sp_runtime::{
    traits::{One, Zero},
    transaction_validity::{
        InvalidTransaction, TransactionSource, TransactionValidity, ValidTransaction,
    },
};
use xpmrl_couple::Pallet as CouplePallet;
use xpmrl_traits::tokens::Tokens;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"xpml");
const UNSIGNED_TXS_PRIORITY: u64 = 100;

pub mod crypto {
    use crate::KEY_TYPE;
    use sp_core::sr25519::Signature as Sr25519Signature;
    use sp_runtime::app_crypto::{app_crypto, sr25519};
    use sp_runtime::{traits::Verify, MultiSignature, MultiSigner};

    app_crypto!(sr25519, KEY_TYPE);

    pub struct OcwAuthId;
    impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for OcwAuthId {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }

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
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::{offchain::*, pallet_prelude::*};
    use sp_runtime::traits::*;
    use xpmrl_traits::tokens::Tokens;
    use xpmrl_utils::with_transaction_result;

    pub(crate) type BalanceOf<T> = <<T as xpmrl_couple::Config>::Tokens as Tokens<
        <T as frame_system::Config>::AccountId,
    >>::Balance;

    pub(crate) type CurrencyIdOf<T> = <<T as xpmrl_couple::Config>::Tokens as Tokens<
        <T as frame_system::Config>::AccountId,
    >>::CurrencyId;

    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
    pub struct Payload<Public, ProposalId, ResultId> {
        proposal_id: ProposalId,
        result: ResultId,
        public: Public,
    }

    impl<T: Config> SignedPayload<T> for Payload<T::Public, T::ProposalId, CurrencyIdOf<T>> {
        fn public(&self) -> T::Public {
            self.public.clone()
        }
    }

    #[pallet::config]
    #[pallet::disable_frame_system_supertrait_check]
    pub trait Config: xpmrl_couple::Config + SigningTypes {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Call: From<Call<Self>>;
        type AuthorityId: AppCrypto<Self::Public, Self::Signature>;

        #[pallet::constant]
        type StakeCurrencyId: Get<CurrencyIdOf<Self>>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn staked_account)]
    pub type StakedAccount<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn autonomy_account)]
    pub type AutonomyAccount<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, (), OptionQuery>;

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

    #[pallet::storage]
    #[pallet::getter(fn statistical_results)]
    pub type StatisticalResults<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, T::ProposalId, Twox64Concat, CurrencyIdOf<T>, u64>;

    #[pallet::storage]
    #[pallet::getter(fn minimal_stake_number)]
    pub type MinimalStakeNumber<T: Config> = StorageValue<_, BalanceOf<T>, OptionQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub minimal_number: BalanceOf<T>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                minimal_number: Zero::zero(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            MinimalStakeNumber::<T>::set(Some(self.minimal_number));
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Stake(T::AccountId, BalanceOf<T>),
        UnStake(T::AccountId, BalanceOf<T>),
        Tagging(T::AccountId),
        Untagging(T::AccountId),
        UploadResult(T::AccountId, T::ProposalId, CurrencyIdOf<T>),
        SetMinimalNumber(BalanceOf<T>),
        MergeResult(T::ProposalId, CurrencyIdOf<T>),
    }

    #[pallet::error]
    pub enum Error<T> {
        AccountAlreadyStaked,
        AccountNotStaked,
        AccountHasAlreadyUploaded,
        Overflow,
        ProposalIdNotExist,
        ProposalOptionNotCorrect,
        ResultIsEqual,
        AccountHasTagged,
        AccountNotTagged,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn offchain_worker(_block_number: T::BlockNumber) {
            debug::info!("Entering off-chain worker");
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn stake(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let number = with_transaction_result(|| Self::inner_stake(&who))?;
            Self::deposit_event(Event::<T>::Stake(who, number));
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn unstake(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let number = with_transaction_result(|| Self::inner_unstake(&who))?;
            Self::deposit_event(Event::<T>::UnStake(who, number));
            Ok(().into())
        }

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
            let who = public.into_account();
            with_transaction_result(|| Self::inner_upload_result(&who, proposal_id, result))?;
            Self::deposit_event(Event::<T>::UploadResult(who, proposal_id, result));
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn merge_result(
            origin: OriginFor<T>,
            proposal_id: T::ProposalId,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_root(origin)?;
            let result = with_transaction_result(|| Self::inner_merge_result(proposal_id))?;
            Self::deposit_event(Event::<T>::MergeResult(proposal_id, result));
            Ok(().into())
        }

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
    }
}

#[cfg(feature = "std")]
impl<T: Config> GenesisConfig<T> {
    pub fn build_storage(&self) -> Result<sp_runtime::Storage, String> {
        <Self as GenesisBuild<T>>::build_storage(self)
    }

    pub fn assimilate_storage(&self, storage: &mut sp_runtime::Storage) -> Result<(), String> {
        <Self as GenesisBuild<T>>::assimilate_storage(self, storage)
    }
}

impl<T: Config> Pallet<T> {
    fn proposal_pairs(
        proposal_id: T::ProposalId,
    ) -> Result<(CurrencyIdOf<T>, CurrencyIdOf<T>), DispatchError> {
        match CouplePallet::<T>::pool_pairs(proposal_id) {
            Some(pairs) => Ok(pairs),
            None => Err(Error::<T>::ProposalIdNotExist)?,
        }
    }

    fn ensure_proposal_optional_id(
        proposal_id: T::ProposalId,
        result: CurrencyIdOf<T>,
    ) -> Result<(), DispatchError> {
        let (id1, id2) = Self::proposal_pairs(proposal_id)?;
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
        T::Tokens::unreserve(currency_id, &who, number)
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
        let (id1, id2) = Self::proposal_pairs(proposal_id)?;
        let sum1 = StatisticalResults::<T>::get(proposal_id, id1).unwrap_or_else(Zero::zero);
        let sum2 = StatisticalResults::<T>::get(proposal_id, id2).unwrap_or_else(Zero::zero);
        ensure!(sum1 != sum1, Error::<T>::ResultIsEqual);
        let result = if sum1 > sum2 { id1 } else { id2 };
        CouplePallet::<T>::set_result(RawOrigin::Root.into(), proposal_id, result)
            .map_err(|e| e.error)?;
        Ok(result)
    }
}

impl<T: Config> frame_support::unsigned::ValidateUnsigned for Module<T> {
    type Call = Call<T>;

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
