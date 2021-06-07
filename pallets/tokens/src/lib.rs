#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use frame_support::{
    dispatch::DispatchError,
    ensure,
    traits::Get,
    traits::{Currency, ExistenceRequirement, ReservableCurrency},
};
use sp_runtime::traits::{AccountIdConversion, CheckedAdd, CheckedSub, One, Zero};
use sp_std::vec::Vec;
use xpmrl_traits::tokens::Tokens;

#[cfg(feature = "std")]
use frame_support::traits::GenesisBuild;

#[frame_support::pallet]
pub mod pallet {
    use codec::FullCodec;
    use frame_support::{
        dispatch::DispatchResultWithPostInfo,
        pallet_prelude::*,
        traits::{Currency, ReservableCurrency},
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::{traits::*, ModuleId};
    use sp_std::{collections::btree_map::BTreeMap, fmt::Debug, vec::Vec};
    use xpmrl_utils::with_transaction_result;

    #[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, Default)]
    pub struct PRC20 {
        pub name: Vec<u8>,
        pub symbol: Vec<u8>,
        pub decimals: u8,
    }

    pub(crate) type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    pub(crate) type CurrencyIdOf<T> = <T as Config>::CurrencyId;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type CurrencyId: FullCodec
            + Eq
            + PartialEq
            + Copy
            + MaybeSerializeDeserialize
            + Debug
            + AtLeast32BitUnsigned;
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        #[pallet::constant]
        type NativeCurrencyId: Get<CurrencyIdOf<Self>>;

        #[pallet::constant]
        type ModuleId: Get<ModuleId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn current_currency_id)]
    pub type CurrentCurrencyId<T: Config> = StorageValue<_, T::CurrencyId>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub tokens: Vec<(Vec<u8>, Vec<u8>, u8)>,
        pub balances: Vec<(T::AccountId, BalanceOf<T>)>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                tokens: Vec::new(),
                balances: Vec::new(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            let f = || -> Result<(), DispatchError> {
                let id = T::NativeCurrencyId::get()
                    .checked_add(&One::one())
                    .ok_or(Error::<T>::CurrencyIdOverflow)?;
                CurrentCurrencyId::<T>::put(id);
                for (name, symbol, decimals) in &self.tokens {
                    let currency_id =
                        Pallet::<T>::inner_new_asset(name.clone(), symbol.clone(), *decimals)?;
                    for (to, number) in &self.balances {
                        Pallet::<T>::inner_mint_to(currency_id, to, *number)?;
                    }
                }
                Ok(())
            };
            if f().is_err() {}
        }
    }

    #[pallet::storage]
    #[pallet::getter(fn currencies)]
    pub type Currencies<T: Config> =
        StorageMap<_, Blake2_128Concat, T::CurrencyId, PRC20, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn total_supply)]
    pub type TotalSupply<T: Config> =
        StorageMap<_, Blake2_128Concat, T::CurrencyId, BalanceOf<T>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn free_balance_of)]
    pub type FreeBalanceOf<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Twox64Concat,
        T::CurrencyId,
        BalanceOf<T>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn reserve_of)]
    pub type ReserveOf<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Twox64Concat,
        T::CurrencyId,
        BalanceOf<T>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn allowance)]
    pub type Allowance<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,                         // owner
        Twox64Concat,                         // hasher
        T::CurrencyId,                        // currency id
        BTreeMap<T::AccountId, BalanceOf<T>>, // map (spender, number)
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        NewAsset(T::CurrencyId),
        Mint(T::CurrencyId, T::AccountId, BalanceOf<T>),
        Burn(T::CurrencyId, T::AccountId, BalanceOf<T>),
        BurnFrom(T::CurrencyId, T::AccountId, T::AccountId, BalanceOf<T>),
        Transfer(T::CurrencyId, T::AccountId, T::AccountId, BalanceOf<T>),
        TransferFrom(
            T::CurrencyId,
            T::AccountId,
            T::AccountId,
            T::AccountId,
            BalanceOf<T>,
        ),
        Approval(T::CurrencyId, T::AccountId, T::AccountId, BalanceOf<T>),
    }

    #[pallet::error]
    pub enum Error<T> {
        CurrencyIdOverflow,
        CurrencyIdNotExist,
        BalanceOverflow,
        ZeroBalance,
        InsufficientBalance,
        TransferFromSelf,
        BurnFromSelf,
        ApproveSelf,
        OriginNotAllowed,
        CannotBurnNativeAsset,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(1_000 + T::DbWeight::get().reads_writes(2, 2))]
        pub fn new_asset(
            origin: OriginFor<T>,
            name: Vec<u8>,
            symbol: Vec<u8>,
            decimals: u8,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_root(origin)?;
            let currency_id = Self::inner_new_asset(name, symbol, decimals)?;
            Self::deposit_event(Event::NewAsset(currency_id));
            Ok(().into())
        }

        #[pallet::weight(1_000 + T::DbWeight::get().reads_writes(2, 1))]
        pub fn mint(
            origin: OriginFor<T>,
            currency_id: T::CurrencyId,
            to: T::AccountId,
            number: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_root(origin)?;
            let actual_number =
                with_transaction_result(|| Self::inner_mint_to(currency_id, &to, number))?;
            Self::deposit_event(Event::Mint(currency_id, to, actual_number));
            Ok(().into())
        }

        #[pallet::weight(1_000 + T::DbWeight::get().reads_writes(2, 1))]
        pub fn burn(
            origin: OriginFor<T>,
            currency_id: T::CurrencyId,
            number: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let actual_number =
                with_transaction_result(|| Self::inner_burn_from(currency_id, &who, number))?;
            Self::deposit_event(Event::Burn(currency_id, who, actual_number));
            Ok(().into())
        }

        #[pallet::weight(1_000 + T::DbWeight::get().reads_writes(6, 2))]
        pub fn burn_from(
            origin: OriginFor<T>,
            currency_id: T::CurrencyId,
            from: T::AccountId,
            number: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(who != from, Error::<T>::BurnFromSelf);
            let alloweds = Allowance::<T>::try_get(&from, currency_id)
                .map_err(|_| Error::<T>::OriginNotAllowed)?;
            let allow = *(alloweds.get(&who).ok_or(Error::<T>::OriginNotAllowed)?);
            ensure!(allow >= number, Error::<T>::OriginNotAllowed);
            let actual_number = with_transaction_result(|| {
                let actual_number = Self::inner_burn_from(currency_id, &from, number)?;
                Self::inner_approve(
                    currency_id,
                    &from,
                    &who,
                    allow.checked_sub(&number).unwrap_or_else(Zero::zero),
                )?;
                Ok(actual_number)
            })?;
            Self::deposit_event(Event::BurnFrom(currency_id, who, from, actual_number));
            Ok(().into())
        }

        #[pallet::weight(1_000 + T::DbWeight::get().reads_writes(4, 2))]
        pub fn transfer(
            origin: OriginFor<T>,
            currency_id: T::CurrencyId,
            to: T::AccountId,
            number: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(who != to, Error::<T>::TransferFromSelf);
            let actual_number = with_transaction_result(|| {
                Self::inner_transfer_from(currency_id, &who, &to, number)
            })?;
            Self::deposit_event(Event::Transfer(currency_id, who, to, actual_number));
            Ok(().into())
        }

        #[pallet::weight(1_000 + T::DbWeight::get().reads_writes(7, 3))]
        pub fn transfer_from(
            origin: OriginFor<T>,
            currency_id: T::CurrencyId,
            from: T::AccountId,
            to: T::AccountId,
            number: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(from != to, Error::<T>::TransferFromSelf);
            ensure!(who != from, Error::<T>::TransferFromSelf);
            let alloweds = Allowance::<T>::try_get(&from, currency_id)
                .map_err(|_| Error::<T>::OriginNotAllowed)?;
            let allow = *(alloweds.get(&who).ok_or(Error::<T>::OriginNotAllowed)?);
            ensure!(allow >= number, Error::<T>::OriginNotAllowed);
            let actual_number = with_transaction_result(|| {
                let actual_number = Self::inner_transfer_from(currency_id, &from, &to, number)?;
                Self::inner_approve(
                    currency_id,
                    &from,
                    &who,
                    allow.checked_sub(&number).unwrap_or_else(Zero::zero),
                )?;
                Ok(actual_number)
            })?;
            Self::deposit_event(Event::TransferFrom(
                currency_id,
                who,
                from,
                to,
                actual_number,
            ));
            Ok(().into())
        }

        #[pallet::weight(1_000 + T::DbWeight::get().reads_writes(2, 1))]
        pub fn approve(
            origin: OriginFor<T>,
            currency_id: T::CurrencyId,
            spender: T::AccountId,
            number: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let acutal_number = with_transaction_result(|| {
                Self::inner_approve(currency_id, &who, &spender, number)
            })?;
            Self::deposit_event(Event::Approval(currency_id, who, spender, acutal_number));
            Ok(().into())
        }

        #[pallet::weight(1_000 + T::DbWeight::get().reads_writes(2, 1))]
        pub fn add_approve(
            origin: OriginFor<T>,
            currency_id: T::CurrencyId,
            spender: T::AccountId,
            number: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let acutal_number = with_transaction_result(|| {
                Self::inner_add_approve(currency_id, &who, &spender, number)
            })?;
            Self::deposit_event(Event::Approval(currency_id, who, spender, acutal_number));
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
    fn get_next_currency_id() -> Result<T::CurrencyId, DispatchError> {
        CurrentCurrencyId::<T>::try_mutate(|value| -> Result<T::CurrencyId, DispatchError> {
            let mut currency_id = value.unwrap_or_else(Zero::zero);
            if currency_id == T::NativeCurrencyId::get() {
                currency_id = currency_id
                    .checked_add(&One::one())
                    .ok_or(Error::<T>::CurrencyIdOverflow)?;
            }
            *value = Some(
                currency_id
                    .checked_add(&One::one())
                    .ok_or(Error::<T>::CurrencyIdOverflow)?,
            );
            Ok(currency_id)
        })
    }

    fn ensure_currency_id(currency_id: T::CurrencyId) -> Result<(), DispatchError> {
        if currency_id != T::NativeCurrencyId::get() {
            ensure!(
                Currencies::<T>::contains_key(currency_id),
                Error::<T>::CurrencyIdNotExist
            );
        }
        Ok(())
    }

    fn inner_decimals(currency_id: T::CurrencyId) -> Result<u8, DispatchError> {
        let xrc = Currencies::<T>::get(currency_id).ok_or(Error::<T>::CurrencyIdNotExist)?;
        Ok(xrc.decimals)
    }

    fn inner_new_asset(
        name: Vec<u8>,
        symbol: Vec<u8>,
        decimals: u8,
    ) -> Result<T::CurrencyId, DispatchError> {
        let currency_id = Self::get_next_currency_id()?;
        let asset = PRC20 {
            name,
            symbol,
            decimals,
        };
        Currencies::<T>::insert(currency_id, asset);
        Ok(currency_id)
    }

    fn inner_mint_to(
        currency_id: T::CurrencyId,
        to: &T::AccountId,
        number: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        Self::ensure_currency_id(currency_id)?;
        if currency_id == T::NativeCurrencyId::get() {
            let old_balance = T::Currency::free_balance(&to);
            let new_balance = old_balance
                .checked_add(&number)
                .ok_or(Error::<T>::BalanceOverflow)?;
            T::Currency::make_free_balance_be(&to, new_balance);
            Ok(number)
        } else {
            let actual_number = FreeBalanceOf::<T>::try_mutate(
                &to,
                currency_id,
                |balance| -> Result<BalanceOf<T>, DispatchError> {
                    let old_balance = balance.unwrap_or_else(Zero::zero);
                    let new_balance = old_balance
                        .checked_add(&number)
                        .ok_or(Error::<T>::BalanceOverflow)?;
                    *balance = Some(new_balance);
                    Ok(new_balance
                        .checked_sub(&old_balance)
                        .unwrap_or_else(Zero::zero))
                },
            )?;
            let _ = TotalSupply::<T>::try_mutate(
                currency_id,
                |total_supply| -> Result<BalanceOf<T>, DispatchError> {
                    let old_total = total_supply.unwrap_or_else(Zero::zero);
                    let new_total = old_total
                        .checked_add(&actual_number)
                        .ok_or(Error::<T>::BalanceOverflow)?;
                    *total_supply = Some(new_total);
                    Ok(new_total.checked_sub(&old_total).unwrap_or_else(Zero::zero))
                },
            )?;
            Ok(actual_number)
        }
    }

    fn inner_burn_from(
        currency_id: T::CurrencyId,
        from: &T::AccountId,
        number: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        Self::ensure_currency_id(currency_id)?;
        if currency_id == T::NativeCurrencyId::get() {
            let old_balance = T::Currency::free_balance(&from);
            let new_balance = old_balance.checked_sub(&number).unwrap_or_else(Zero::zero);
            T::Currency::make_free_balance_be(&from, new_balance);
            Ok(number)
        } else {
            let actual_number = FreeBalanceOf::<T>::try_mutate(
                &from,
                currency_id,
                |balance| -> Result<BalanceOf<T>, DispatchError> {
                    let old_balance = balance.unwrap_or_else(Zero::zero);
                    let new_balance = old_balance.checked_sub(&number).unwrap_or_else(Zero::zero);
                    *balance = Some(new_balance);
                    Ok(old_balance
                        .checked_sub(&new_balance)
                        .unwrap_or_else(Zero::zero))
                },
            )?;
            let _ = TotalSupply::<T>::try_mutate(
                currency_id,
                |total_supply| -> Result<BalanceOf<T>, DispatchError> {
                    let old_total = total_supply.ok_or(Error::<T>::CurrencyIdNotExist)?;
                    let new_total = old_total
                        .checked_sub(&actual_number)
                        .unwrap_or_else(Zero::zero);
                    *total_supply = Some(new_total);
                    Ok(old_total.checked_sub(&new_total).unwrap_or_else(Zero::zero))
                },
            )?;
            Ok(actual_number)
        }
    }

    fn inner_transfer_from(
        currency_id: T::CurrencyId,
        from: &T::AccountId,
        to: &T::AccountId,
        number: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        Self::ensure_currency_id(currency_id)?;
        if currency_id == T::NativeCurrencyId::get() {
            ensure!(
                T::Currency::free_balance(&from) >= number,
                Error::<T>::InsufficientBalance
            );
            T::Currency::transfer(&from, &to, number, ExistenceRequirement::AllowDeath)?;
            Ok(number)
        } else {
            ensure!(
                FreeBalanceOf::<T>::get(&from, currency_id).unwrap_or_else(Zero::zero) >= number,
                Error::<T>::InsufficientBalance
            );
            let actual_number = FreeBalanceOf::<T>::try_mutate(
                &from,
                currency_id,
                |balance| -> Result<BalanceOf<T>, DispatchError> {
                    let old_balance = balance.unwrap_or_else(Zero::zero);
                    let new_balance = old_balance.checked_sub(&number).unwrap_or_else(Zero::zero);
                    *balance = Some(new_balance);
                    Ok(old_balance - new_balance)
                },
            )?;
            FreeBalanceOf::<T>::try_mutate(
                &to,
                currency_id,
                |balance| -> Result<BalanceOf<T>, DispatchError> {
                    let old_balance = balance.unwrap_or_else(Zero::zero);
                    *balance = Some(
                        old_balance
                            .checked_add(&actual_number)
                            .ok_or(Error::<T>::BalanceOverflow)?,
                    );
                    Ok(actual_number)
                },
            )
        }
    }

    fn inner_approve(
        currency_id: T::CurrencyId,
        owner: &T::AccountId,
        spender: &T::AccountId,
        number: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        Self::ensure_currency_id(currency_id)?;
        ensure!(owner != spender, Error::<T>::ApproveSelf);
        Allowance::<T>::try_mutate(
            owner,
            currency_id,
            |items| -> Result<BalanceOf<T>, DispatchError> {
                let mut new_items = items.clone().unwrap_or_default();
                new_items.insert(spender.clone(), number);
                *items = Some(new_items);
                Ok(number)
            },
        )
    }

    fn inner_add_approve(
        currency_id: T::CurrencyId,
        owner: &T::AccountId,
        spender: &T::AccountId,
        number: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        Self::ensure_currency_id(currency_id)?;
        ensure!(owner != spender, Error::<T>::ApproveSelf);
        Allowance::<T>::try_mutate(
            owner,
            currency_id,
            |items| -> Result<BalanceOf<T>, DispatchError> {
                let mut new_items = items.clone().unwrap_or_default();
                let number = {
                    if let Some(x) = new_items.get(&spender) {
                        x.checked_add(&number).ok_or(Error::<T>::BalanceOverflow)?
                    } else {
                        number
                    }
                };
                new_items.insert(spender.clone(), number);
                *items = Some(new_items);
                Ok(number)
            },
        )
    }

    fn inner_reserve(
        currency_id: T::CurrencyId,
        from: &T::AccountId,
        number: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        if currency_id == T::NativeCurrencyId::get() {
            ensure!(
                T::Currency::free_balance(&from) >= number,
                Error::<T>::InsufficientBalance
            );
            T::Currency::reserve(&from, number)?;
            Ok(number)
        } else {
            let actual_number = FreeBalanceOf::<T>::try_mutate(
                &from,
                currency_id,
                |val| -> Result<BalanceOf<T>, DispatchError> {
                    let old_val = val.unwrap_or_else(Zero::zero);
                    ensure!(old_val >= number, Error::<T>::InsufficientBalance);
                    let new_val = old_val.checked_sub(&number).unwrap_or_else(Zero::zero);
                    *val = Some(new_val);
                    Ok(old_val.checked_sub(&number).unwrap_or_else(Zero::zero))
                },
            )?;
            ReserveOf::<T>::try_mutate(&from, currency_id, |val| -> Result<(), DispatchError> {
                let old_val = val.unwrap_or_else(Zero::zero);
                let new_val = old_val
                    .checked_add(&actual_number)
                    .ok_or(Error::<T>::BalanceOverflow)?;
                *val = Some(new_val);
                Ok(())
            })?;
            Ok(actual_number)
        }
    }

    fn inner_slash_reserved(
        currency_id: T::CurrencyId,
        from: &T::AccountId,
        number: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        Self::ensure_currency_id(currency_id)?;
        if currency_id == T::NativeCurrencyId::get() {
            let old_value = T::Currency::reserved_balance(&from);
            let new_value = old_value.checked_sub(&number).unwrap_or_else(Zero::zero);
            let (_, actual_number) = T::Currency::slash_reserved(
                &from,
                old_value.checked_sub(&new_value).unwrap_or_else(Zero::zero),
            );
            Ok(actual_number)
        } else {
            let actual_number = ReserveOf::<T>::try_mutate(
                &from,
                currency_id,
                |val| -> Result<BalanceOf<T>, DispatchError> {
                    let old_val = val.unwrap_or_else(Zero::zero);
                    let new_val = old_val.checked_sub(&number).unwrap_or_else(Zero::zero);
                    *val = Some(new_val);
                    Ok(old_val.checked_sub(&new_val).unwrap_or_else(Zero::zero))
                },
            )?;
            let _ = TotalSupply::<T>::try_mutate(
                currency_id,
                |total_supply| -> Result<BalanceOf<T>, DispatchError> {
                    let old_total = total_supply.ok_or(Error::<T>::CurrencyIdNotExist)?;
                    let new_total = old_total
                        .checked_sub(&actual_number)
                        .unwrap_or_else(Zero::zero);
                    *total_supply = Some(new_total);
                    Ok(old_total.checked_sub(&new_total).unwrap_or_else(Zero::zero))
                },
            )?;
            Ok(actual_number)
        }
    }

    fn inner_unreserve(
        currency_id: T::CurrencyId,
        from: &T::AccountId,
        number: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        if currency_id == T::NativeCurrencyId::get() {
            ensure!(
                T::Currency::reserved_balance(&from) >= number,
                Error::<T>::InsufficientBalance
            );
            T::Currency::unreserve(&from, number);
            Ok(number)
        } else {
            let mut actual_number = ReserveOf::<T>::try_mutate(
                &from,
                currency_id,
                |val| -> Result<BalanceOf<T>, DispatchError> {
                    let old_val = val.unwrap_or_else(Zero::zero);
                    ensure!(old_val >= number, Error::<T>::InsufficientBalance);
                    let new_val = old_val.checked_sub(&number).unwrap_or_else(Zero::zero);
                    *val = Some(new_val);
                    Ok(old_val.checked_sub(&new_val).unwrap_or_else(Zero::zero))
                },
            )?;
            actual_number = FreeBalanceOf::<T>::try_mutate(
                &from,
                currency_id,
                |val| -> Result<BalanceOf<T>, DispatchError> {
                    let old_val = val.unwrap_or_else(Zero::zero);
                    let new_val = old_val
                        .checked_add(&actual_number)
                        .ok_or(Error::<T>::BalanceOverflow)?;
                    *val = Some(new_val);
                    Ok(new_val.checked_sub(&old_val).unwrap_or_else(Zero::zero))
                },
            )?;
            Ok(actual_number)
        }
    }

    fn inner_balance_of(currency_id: T::CurrencyId, who: &T::AccountId) -> BalanceOf<T> {
        if currency_id == T::NativeCurrencyId::get() {
            T::Currency::free_balance(&who)
        } else {
            FreeBalanceOf::<T>::get(&who, currency_id).unwrap_or_else(Zero::zero)
        }
    }
}

impl<T: Config> Tokens<T::AccountId> for Pallet<T> {
    type CurrencyId = T::CurrencyId;
    type Balance = BalanceOf<T>;

    fn new_asset(
        name: Vec<u8>,
        symbol: Vec<u8>,
        decimals: u8,
    ) -> Result<Self::CurrencyId, DispatchError> {
        Self::inner_new_asset(name, symbol, decimals)
    }

    fn decimals(currency_id: Self::CurrencyId) -> Result<u8, DispatchError> {
        Self::inner_decimals(currency_id)
    }

    fn balance(currency_id: Self::CurrencyId, account: &T::AccountId) -> Self::Balance {
        Self::inner_balance_of(currency_id, &account)
    }

    fn transfer(
        currency_id: Self::CurrencyId,
        from: &T::AccountId,
        to: &T::AccountId,
        number: Self::Balance,
    ) -> Result<Self::Balance, DispatchError> {
        Self::inner_transfer_from(currency_id, &from, &to, number)
    }

    fn mint(
        currency_id: Self::CurrencyId,
        to: &T::AccountId,
        number: Self::Balance,
    ) -> Result<Self::Balance, DispatchError> {
        Self::inner_mint_to(currency_id, &to, number)
    }

    fn burn(
        currency_id: Self::CurrencyId,
        from: &T::AccountId,
        number: Self::Balance,
    ) -> Result<Self::Balance, DispatchError> {
        Self::inner_burn_from(currency_id, &from, number)
    }

    fn reserve(
        currency_id: Self::CurrencyId,
        who: &T::AccountId,
        value: Self::Balance,
    ) -> Result<Self::Balance, DispatchError> {
        Self::inner_reserve(currency_id, &who, value)
    }

    fn unreserve(
        currency_id: Self::CurrencyId,
        who: &T::AccountId,
        value: Self::Balance,
    ) -> Result<Self::Balance, DispatchError> {
        Self::inner_unreserve(currency_id, &who, value)
    }

    fn slash_reserved(
        currency_id: Self::CurrencyId,
        who: &T::AccountId,
        value: Self::Balance,
    ) -> Result<Self::Balance, DispatchError> {
        Self::inner_slash_reserved(currency_id, &who, value)
    }

    fn reserved_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
        Self::reserve_of(&who, currency_id).unwrap_or_else(Zero::zero)
    }

    fn donate(
        currency_id: Self::CurrencyId,
        from: &T::AccountId,
        value: Self::Balance,
    ) -> Result<Self::Balance, DispatchError> {
        let module_account: T::AccountId = T::ModuleId::get().into_account();
        Self::inner_transfer_from(currency_id, &from, &module_account, value)
    }

    fn mint_donate(
        currency_id: Self::CurrencyId,
        value: Self::Balance,
    ) -> Result<Self::Balance, DispatchError> {
        let module_account: T::AccountId = T::ModuleId::get().into_account();
        <Self as Tokens<T::AccountId>>::mint(currency_id, &module_account, value)
    }

    fn burn_donate(
        currency_id: Self::CurrencyId,
        value: Self::Balance,
    ) -> Result<Self::Balance, DispatchError> {
        let module_account: T::AccountId = T::ModuleId::get().into_account();
        <Self as Tokens<T::AccountId>>::burn(currency_id, &module_account, value)
    }

    fn appropriation(
        currency_id: Self::CurrencyId,
        to: &T::AccountId,
        value: Self::Balance,
    ) -> Result<Self::Balance, DispatchError> {
        let module_account: T::AccountId = T::ModuleId::get().into_account();
        Self::inner_transfer_from(currency_id, &module_account, &to, value)
    }
}
