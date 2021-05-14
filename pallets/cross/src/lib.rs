#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::ensure;
pub use pallet::*;
use sp_runtime::{
    traits::{CheckedSub, Zero},
    DispatchError,
};

#[frame_support::pallet]
pub mod pallet {
    use codec::FullCodec;
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::Time};
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::{AtLeast32BitUnsigned, CheckedAdd};
    use sp_std::{collections::btree_map::BTreeMap, fmt::Debug, vec::Vec};
    use xpmrl_traits::tokens::Tokens;
    use xpmrl_utils::with_transaction_result;

    pub(crate) type MomentOf<T> = <<T as Config>::Time as Time>::Moment;
    pub(crate) type CurrencyIdOf<T> =
        <<T as Config>::Tokens as Tokens<<T as frame_system::Config>::AccountId>>::CurrencyId;
    pub(crate) type BalanceOf<T> =
        <<T as Config>::Tokens as Tokens<<T as frame_system::Config>::AccountId>>::Balance;

    #[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, Default)]
    pub struct CrossInfo<CurrencyId, Balance, ChainId> {
        pub to: Vec<u8>,
        pub currency_id: CurrencyId,
        pub number: Balance,
        pub chain_id: ChainId,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Tokens: Tokens<Self::AccountId>;
        type Time: Time;
        type ChainId: FullCodec
            + Eq
            + PartialEq
            + Copy
            + MaybeSerializeDeserialize
            + Debug
            + AtLeast32BitUnsigned;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn transaction_info)]
    pub type TransactionInfo<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Twox64Concat,
        MomentOf<T>,
        CrossInfo<CurrencyIdOf<T>, BalanceOf<T>, T::ChainId>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn burn_account)]
    pub type BurnAccount<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn allowance_reserve)]
    pub type Allowance<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,                         // owner
        Twox64Concat,                         // hasher
        CurrencyIdOf<T>,                      // currency id
        BTreeMap<T::AccountId, BalanceOf<T>>, // map (spender, number)
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        CrossWithdrawal(
            T::AccountId,
            Vec<u8>,
            CurrencyIdOf<T>,
            BalanceOf<T>,
            MomentOf<T>,
        ),
        NewBurnAddress(T::AccountId),
        UnReserved(
            CurrencyIdOf<T>,
            T::ChainId,
            T::AccountId,
            MomentOf<T>,
            BalanceOf<T>,
        ),
    }

    #[pallet::error]
    pub enum Error<T> {
        MustSetBurnAddress,
        MustCallFromBurnAddress,
        AddressNotCross,
        ApproveSelf,
        BalanceOverflow,
        OriginNotAllowed,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn cross_transaction(
            origin: OriginFor<T>,
            chain_id: T::ChainId,
            address: Vec<u8>,
            currency_id: CurrencyIdOf<T>,
            number: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let now = T::Time::now();
            let burn_address = BurnAccount::<T>::get().ok_or(Error::<T>::MustSetBurnAddress)?;
            ensure!(who != burn_address, Error::<T>::ApproveSelf);
            with_transaction_result(|| {
                T::Tokens::reserve(currency_id, &who, number)?;
                Allowance::<T>::try_mutate(
                    who.clone(),
                    currency_id,
                    |items| -> Result<BalanceOf<T>, DispatchError> {
                        let mut new_items = items.clone().unwrap_or(BTreeMap::new());
                        let number = {
                            if let Some(x) = new_items.get(&burn_address) {
                                x.checked_add(&number).ok_or(Error::<T>::BalanceOverflow)?
                            } else {
                                number
                            }
                        };
                        new_items.insert(burn_address.clone(), number);
                        *items = Some(new_items);
                        Ok(number)
                    },
                )?;
                TransactionInfo::<T>::insert(
                    who.clone(),
                    now,
                    CrossInfo {
                        to: address.clone(),
                        currency_id,
                        number,
                        chain_id,
                    },
                );
                Ok(())
            })?;
            Self::deposit_event(Event::CrossWithdrawal(
                who,
                address,
                currency_id,
                number,
                now,
            ));
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn set_burn_address(
            origin: OriginFor<T>,
            address: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_root(origin)?;
            BurnAccount::<T>::set(Some(address.clone()));
            Self::deposit_event(Event::NewBurnAddress(address));
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn unreserved(
            origin: OriginFor<T>,
            currency_id: CurrencyIdOf<T>,
            chain_id: T::ChainId,
            address: T::AccountId,
            time: MomentOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let burn_address = BurnAccount::<T>::get().ok_or(Error::<T>::MustSetBurnAddress)?;
            ensure!(who == burn_address, Error::<T>::MustCallFromBurnAddress);
            let old = TransactionInfo::<T>::try_get(&address, time)
                .map_err(|_| Error::<T>::AddressNotCross)?;
            ensure!(old.chain_id == chain_id, Error::<T>::AddressNotCross);
            ensure!(old.currency_id == currency_id, Error::<T>::AddressNotCross);
            with_transaction_result(|| {
                Self::update_info(&who, &address, currency_id, old.number, time)?;
                T::Tokens::unreserve(currency_id, &address, old.number)?;
                Ok(())
            })?;
            Self::deposit_event(Event::UnReserved(
                currency_id,
                chain_id,
                address,
                time,
                old.number,
            ));
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn slash_reserved(
            origin: OriginFor<T>,
            currency_id: CurrencyIdOf<T>,
            chain_id: T::ChainId,
            address: T::AccountId,
            time: MomentOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let burn_address = BurnAccount::<T>::get().ok_or(Error::<T>::MustSetBurnAddress)?;
            ensure!(who == burn_address, Error::<T>::MustCallFromBurnAddress);
            let old = TransactionInfo::<T>::try_get(&address, time)
                .map_err(|_| Error::<T>::AddressNotCross)?;
            ensure!(old.chain_id == chain_id, Error::<T>::AddressNotCross);
            ensure!(old.currency_id == currency_id, Error::<T>::AddressNotCross);
            with_transaction_result(|| {
                Self::update_info(&who, &address, currency_id, old.number, time)?;
                T::Tokens::slash_reserved(currency_id, &address, old.number)?;
                Ok(())
            })?;
            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T> {
    fn update_info(
        who: &T::AccountId,
        address: &T::AccountId,
        currency_id: CurrencyIdOf<T>,
        number: BalanceOf<T>,
        time: MomentOf<T>,
    ) -> Result<(), DispatchError> {
        Allowance::<T>::try_mutate_exists(
            &who,
            currency_id,
            |item| -> Result<(), DispatchError> {
                let mut alloweds = item.clone().ok_or(Error::<T>::OriginNotAllowed)?;
                let allow = *(alloweds.get(&who).ok_or(Error::<T>::OriginNotAllowed)?);
                ensure!(allow >= number, Error::<T>::OriginNotAllowed);
                let result_number = allow.checked_sub(&number).unwrap_or(Zero::zero());
                *item = if result_number == Zero::zero() {
                    None
                } else {
                    alloweds.insert(who.clone(), result_number);
                    Some(alloweds)
                };
                Ok(())
            },
        )?;
        TransactionInfo::<T>::try_mutate_exists(
            address.clone(),
            time,
            |item| -> Result<(), DispatchError> {
                *item = None;
                Ok(())
            },
        )?;
        Ok(())
    }
}
