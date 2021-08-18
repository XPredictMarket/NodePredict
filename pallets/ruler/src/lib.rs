//! <!-- markdown-link-check-disable -->
//! # Ruler
//!
//! Run `cargo doc --package xpmrl-ruler --open` to view this pallet's documentation.
//!
//! A module that manages a number of addresses with control rights
//!
//! - [`xpmrl_ruler::Config`](./pallet/trait.Config.html)
//! - [`Call`](./pallet/enum.Call.html)
//! - [`Pallet`](./pallet/struct.Pallet.html)
//!
//! ## Overview
//!
//! Use the control address to manage some modules and transfer the authority to other addresses
//!

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub use pallet::*;

use frame_support::dispatch::DispatchError;
use xpmrl_traits::{ruler::RulerAccounts, RulerModule};

#[cfg(feature = "std")]
use frame_support::traits::GenesisBuild;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use xpmrl_traits::RulerModule;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// The address of the ruler, which stores the ruler of each module
    #[pallet::storage]
    #[pallet::getter(fn ruler_address)]
    pub type RulerAddress<T: Config> =
        StorageMap<_, Blake2_128Concat, RulerModule, T::AccountId, OptionQuery>;

    /// If the ruler needs to transfer permissions to other accounts, in order to prevent transfer
    /// errors, an intermediate waiting state is set, and then the transferred account accepts the
    /// permissions.
    #[pallet::storage]
    #[pallet::getter(fn pending_ruler_address)]
    pub type PendingRulerAddress<T: Config> =
        StorageMap<_, Blake2_128Concat, RulerModule, T::AccountId, OptionQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub dividend_address: T::AccountId,
        pub burn_address: T::AccountId,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                dividend_address: Default::default(),
                burn_address: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            RulerAddress::<T>::insert(RulerModule::PlatformDividend, self.dividend_address.clone());
            RulerAddress::<T>::insert(RulerModule::CrossChainBurn, self.burn_address.clone());
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        PendingRulerAddress(RulerModule, T::AccountId, T::AccountId),
        AcceptRulerAddress(RulerModule, T::AccountId),
    }

    #[pallet::error]
    pub enum Error<T> {
        NotTransferSelf,
        ModuleNotAllowed,
        PermissionDenied,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Transfer control authority
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn transfer_ruler_address(
            origin: OriginFor<T>,
            module: RulerModule,
            address: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(who != address, Error::<T>::NotTransferSelf);
            let old = RulerAddress::<T>::get(module).ok_or(Error::<T>::ModuleNotAllowed)?;
            ensure!(who == old, Error::<T>::PermissionDenied);
            PendingRulerAddress::<T>::insert(module, address.clone());
            Self::deposit_event(Event::PendingRulerAddress(module, who, address));
            Ok(().into())
        }

        /// Accept control authority
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn accept_ruler_address(
            origin: OriginFor<T>,
            module: RulerModule,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            PendingRulerAddress::<T>::try_mutate_exists(
                module,
                |optional| -> Result<(), DispatchError> {
                    match optional {
                        Some(account) => {
                            ensure!(who == *account, Error::<T>::ModuleNotAllowed);
                            RulerAddress::<T>::insert(module, account);
                            *optional = None;
                            Ok(())
                        }
                        None => Err(Error::<T>::PermissionDenied.into()),
                    }
                },
            )?;
            Self::deposit_event(Event::AcceptRulerAddress(module, who));
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

impl<T: Config> RulerAccounts<T> for Pallet<T> {
    fn get_account(module: RulerModule) -> Result<T::AccountId, DispatchError> {
        match RulerAddress::<T>::get(module) {
            Some(account) => Ok(account),
            None => Err(Error::<T>::ModuleNotAllowed.into()),
        }
    }
}
