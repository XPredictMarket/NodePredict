#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::{Codec, Decode, Encode};
use sp_runtime::traits::{MaybeDisplay, MaybeFromStr};
use sp_std::vec::Vec;

#[cfg(feature = "std")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(feature = "std")]
use std::str;

#[derive(PartialEq, Eq, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, Deserialize, Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct ProposalInfo<CategoryId, Balance, Moment> {
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
fn vec_u8_serialize_as_string<S: Serializer>(
	t: &Vec<u8>,
	serializer: S,
) -> Result<S::Ok, S::Error> {
	let s =
		str::from_utf8(&t).map_err(|_| serde::ser::Error::custom("cannot convert to string"))?;
	serializer.serialize_str(s)
}

#[cfg(feature = "std")]
fn vec_u8_deserialize_from_string<'de, D: Deserializer<'de>>(
	deserializer: D,
) -> Result<Vec<u8>, D::Error> {
	let s = String::deserialize(deserializer)?;
	Ok(s.as_bytes().to_vec())
}

sp_api::decl_runtime_apis! {
	pub trait CoupleInfoApi<ProposalId, CategoryId, Balance, Moment> where
		ProposalId: Codec,
		CategoryId: Codec,
		Balance: Codec + MaybeDisplay + MaybeFromStr,
		Moment: Codec,
	{
		fn get_proposal_info(proposal_id: ProposalId) -> ProposalInfo<CategoryId, Balance, Moment>;
	}
}
