use codec::FullCodec;
use frame_support::pallet_prelude::MaybeSerializeDeserialize;
use sp_runtime::{traits::AtLeast32BitUnsigned, DispatchError};
use sp_std::{fmt::Debug, vec::Vec};
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
    fn decimals(currency_id: Self::CurrencyId) -> Result<u8, DispatchError>;
    fn balance(currency_id: Self::CurrencyId, account: &AccountId) -> Self::Balance;
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
    fn slash_reserved(
        currency_id: Self::CurrencyId,
        who: &AccountId,
        value: Self::Balance,
    ) -> Result<Self::Balance, DispatchError>;
    fn reserved_balance(currency_id: Self::CurrencyId, who: &AccountId) -> Self::Balance;
    fn donate(
        currency_id: Self::CurrencyId,
        from: &AccountId,
        value: Self::Balance,
    ) -> Result<Self::Balance, DispatchError>;
    fn mint_donate(
        currency_id: Self::CurrencyId,
        value: Self::Balance,
    ) -> Result<Self::Balance, DispatchError>;
    fn burn_donate(
        currency_id: Self::CurrencyId,
        value: Self::Balance,
    ) -> Result<Self::Balance, DispatchError>;
    fn appropriation(
        currency_id: Self::CurrencyId,
        to: &AccountId,
        value: Self::Balance,
    ) -> Result<Self::Balance, DispatchError>;
}
