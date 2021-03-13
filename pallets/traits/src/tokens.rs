use codec::FullCodec;
use frame_support::pallet_prelude::MaybeSerializeDeserialize;
use sp_runtime::{traits::AtLeast32BitUnsigned, DispatchError};
use sp_std::{ vec::Vec, fmt::Debug };

pub trait Tokens<AccountId> {
    type CurrencyId: FullCodec
        + Eq
        + PartialEq
        + Copy
        + MaybeSerializeDeserialize
        + Debug
        + AtLeast32BitUnsigned;
    type Balance: AtLeast32BitUnsigned
        + FullCodec
        + Copy
        + MaybeSerializeDeserialize
        + Debug
        + Default;

    fn new_asset(
        name: Vec<u8>,
        symbol: Vec<u8>,
        decimals: u8,
    ) -> Result<Self::CurrencyId, DispatchError>;
    fn transfer(
        currency_id: Self::CurrencyId,
        from: &AccountId,
        to: &AccountId,
        number: Self::Balance,
    ) -> Result<Self::Balance, DispatchError>;
    fn mint(
        currency_id: Self::CurrencyId,
        to: &AccountId,
        number: Self::Balance,
    ) -> Result<Self::Balance, DispatchError>;
    fn burn(
        currency_id: Self::CurrencyId,
        from: &AccountId,
        number: Self::Balance,
    ) -> Result<Self::Balance, DispatchError>;
    fn reserve(
        currency_id: Self::CurrencyId,
        who: &AccountId,
        value: Self::Balance,
    ) -> Result<Self::Balance, DispatchError>;
    fn unreserve(
        currency_id: Self::CurrencyId,
        who: &AccountId,
        value: Self::Balance,
    ) -> Result<Self::Balance, DispatchError>;
    fn reserved_balance(currency_id: Self::CurrencyId, who: &AccountId) -> Self::Balance;
}
