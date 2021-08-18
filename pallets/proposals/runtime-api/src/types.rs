use codec::{Decode, Encode};
use sp_std::vec::Vec;
use xpmrl_traits::ProposalStatus;

#[cfg(feature = "std")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(feature = "std")]
use std::str;

#[derive(PartialEq, Eq, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, Deserialize, Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct ProposalInfo<CategoryId, Balance, Moment, CurrencyId> {
    #[cfg_attr(feature = "std", serde(serialize_with = "vec_u8_serialize_as_string"))]
    #[cfg_attr(
        feature = "std",
        serde(deserialize_with = "vec_u8_deserialize_from_string")
    )]
    pub title: Vec<u8>,
    pub category_id: CategoryId,
    #[cfg_attr(feature = "std", serde(serialize_with = "vec_u8_serialize_as_string"))]
    #[cfg_attr(
        feature = "std",
        serde(deserialize_with = "vec_u8_deserialize_from_string")
    )]
    pub detail: Vec<u8>,
    #[cfg_attr(
        feature = "std",
        serde(bound(serialize = "Balance: std::fmt::Display"))
    )]
    #[cfg_attr(feature = "std", serde(serialize_with = "balance_serialize_as_string"))]
    #[cfg_attr(
        feature = "std",
        serde(bound(deserialize = "Balance: std::str::FromStr"))
    )]
    #[cfg_attr(
        feature = "std",
        serde(deserialize_with = "balance_deserialize_from_string")
    )]
    pub yes: Balance,
    #[cfg_attr(feature = "std", serde(serialize_with = "vec_u8_serialize_as_string"))]
    #[cfg_attr(
        feature = "std",
        serde(deserialize_with = "vec_u8_deserialize_from_string")
    )]
    pub yes_name: Vec<u8>,
    #[cfg_attr(
        feature = "std",
        serde(bound(serialize = "Balance: std::fmt::Display"))
    )]
    #[cfg_attr(feature = "std", serde(serialize_with = "balance_serialize_as_string"))]
    #[cfg_attr(
        feature = "std",
        serde(bound(deserialize = "Balance: std::str::FromStr"))
    )]
    #[cfg_attr(
        feature = "std",
        serde(deserialize_with = "balance_deserialize_from_string")
    )]
    pub no: Balance,
    #[cfg_attr(feature = "std", serde(serialize_with = "vec_u8_serialize_as_string"))]
    #[cfg_attr(
        feature = "std",
        serde(deserialize_with = "vec_u8_deserialize_from_string")
    )]
    pub no_name: Vec<u8>,
    pub close_time: Moment,
    #[cfg_attr(
        feature = "std",
        serde(bound(serialize = "Balance: std::fmt::Display"))
    )]
    #[cfg_attr(feature = "std", serde(serialize_with = "balance_serialize_as_string"))]
    #[cfg_attr(
        feature = "std",
        serde(bound(deserialize = "Balance: std::str::FromStr"))
    )]
    #[cfg_attr(
        feature = "std",
        serde(deserialize_with = "balance_deserialize_from_string")
    )]
    pub liquidity: Balance,
    pub status: ProposalStatus,
    pub decimals: u8,
    pub token_id: CurrencyId,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, Deserialize, Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct PersonalProposalInfo<Balance, Moment, CurrencyId> {
    #[cfg_attr(feature = "std", serde(serialize_with = "vec_u8_serialize_as_string"))]
    #[cfg_attr(
        feature = "std",
        serde(deserialize_with = "vec_u8_deserialize_from_string")
    )]
    pub title: Vec<u8>,
    #[cfg_attr(feature = "std", serde(serialize_with = "vec_u8_serialize_as_string"))]
    #[cfg_attr(
        feature = "std",
        serde(deserialize_with = "vec_u8_deserialize_from_string")
    )]
    pub yes_name: Vec<u8>,
    #[cfg_attr(feature = "std", serde(serialize_with = "vec_u8_serialize_as_string"))]
    #[cfg_attr(
        feature = "std",
        serde(deserialize_with = "vec_u8_deserialize_from_string")
    )]
    pub no_name: Vec<u8>,
    pub currency_id: CurrencyId,
    pub yes_currency_id: CurrencyId,
    pub no_currency_id: CurrencyId,
    pub liquidity_currency_id: CurrencyId,
    pub decimals: u8,
    pub yes_decimals: u8,
    pub no_decimals: u8,
    pub liquidity_decimals: u8,
    pub fee_rate_decimals: u8,
    pub fee_rate: u32,
    #[cfg_attr(
        feature = "std",
        serde(bound(serialize = "Balance: std::fmt::Display"))
    )]
    #[cfg_attr(feature = "std", serde(serialize_with = "balance_serialize_as_string"))]
    #[cfg_attr(
        feature = "std",
        serde(bound(deserialize = "Balance: std::str::FromStr"))
    )]
    #[cfg_attr(
        feature = "std",
        serde(deserialize_with = "balance_deserialize_from_string")
    )]
    pub fee: Balance,
    #[cfg_attr(
        feature = "std",
        serde(bound(serialize = "Balance: std::fmt::Display"))
    )]
    #[cfg_attr(feature = "std", serde(serialize_with = "balance_serialize_as_string"))]
    #[cfg_attr(
        feature = "std",
        serde(bound(deserialize = "Balance: std::str::FromStr"))
    )]
    #[cfg_attr(
        feature = "std",
        serde(deserialize_with = "balance_deserialize_from_string")
    )]
    pub no: Balance,
    #[cfg_attr(
        feature = "std",
        serde(bound(serialize = "Balance: std::fmt::Display"))
    )]
    #[cfg_attr(feature = "std", serde(serialize_with = "balance_serialize_as_string"))]
    #[cfg_attr(
        feature = "std",
        serde(bound(deserialize = "Balance: std::str::FromStr"))
    )]
    #[cfg_attr(
        feature = "std",
        serde(deserialize_with = "balance_deserialize_from_string")
    )]
    pub yes: Balance,
    #[cfg_attr(
        feature = "std",
        serde(bound(serialize = "Balance: std::fmt::Display"))
    )]
    #[cfg_attr(feature = "std", serde(serialize_with = "balance_serialize_as_string"))]
    #[cfg_attr(
        feature = "std",
        serde(bound(deserialize = "Balance: std::str::FromStr"))
    )]
    #[cfg_attr(
        feature = "std",
        serde(deserialize_with = "balance_deserialize_from_string")
    )]
    pub total: Balance,
    #[cfg_attr(
        feature = "std",
        serde(bound(serialize = "Balance: std::fmt::Display"))
    )]
    #[cfg_attr(feature = "std", serde(serialize_with = "balance_serialize_as_string"))]
    #[cfg_attr(
        feature = "std",
        serde(bound(deserialize = "Balance: std::str::FromStr"))
    )]
    #[cfg_attr(
        feature = "std",
        serde(deserialize_with = "balance_deserialize_from_string")
    )]
    pub liquidity: Balance,
    #[cfg_attr(
        feature = "std",
        serde(bound(serialize = "Balance: std::fmt::Display"))
    )]
    #[cfg_attr(feature = "std", serde(serialize_with = "balance_serialize_as_string"))]
    #[cfg_attr(
        feature = "std",
        serde(bound(deserialize = "Balance: std::str::FromStr"))
    )]
    #[cfg_attr(
        feature = "std",
        serde(deserialize_with = "balance_deserialize_from_string")
    )]
    pub balance: Balance,
    pub close_time: Moment,
    pub status: ProposalStatus,
}

#[cfg(feature = "std")]
fn balance_serialize_as_string<S: Serializer, T: std::fmt::Display>(
    t: &T,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&t.to_string())
}

#[cfg(feature = "std")]
fn balance_deserialize_from_string<'de, D: Deserializer<'de>, T: std::str::FromStr>(
    deserializer: D,
) -> Result<T, D::Error> {
    let s = String::deserialize(deserializer)?;
    s.parse::<T>()
        .map_err(|_| serde::de::Error::custom("Parse from string failed"))
}

#[cfg(feature = "std")]
fn vec_u8_serialize_as_string<S: Serializer>(t: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
    let s = str::from_utf8(t).map_err(|_| serde::ser::Error::custom("cannot convert to string"))?;
    serializer.serialize_str(s)
}

#[cfg(feature = "std")]
fn vec_u8_deserialize_from_string<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<u8>, D::Error> {
    let s = String::deserialize(deserializer)?;
    Ok(s.as_bytes().to_vec())
}
