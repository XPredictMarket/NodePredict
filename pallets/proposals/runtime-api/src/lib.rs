#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use sp_runtime::traits::{MaybeDisplay, MaybeFromStr};

pub mod types;

sp_api::decl_runtime_apis! {
	pub trait CoupleInfoApi<VersionId, ProposalId, CategoryId, Balance, Moment, CurrencyId, AccountId> where
		VersionId: Codec,
		ProposalId: Codec,
		CategoryId: Codec,
		Balance: Codec + MaybeDisplay + MaybeFromStr,
		Moment: Codec,
		CurrencyId: Codec,
		AccountId: Codec + Clone,
	{
		fn get_proposal_info(version_id: VersionId, proposal_id: ProposalId) -> types::ProposalInfo<CategoryId, Balance, Moment, CurrencyId>;
		fn get_personal_proposal_info(version_id: VersionId, proposal_id: ProposalId, account_id: AccountId) -> types::PersonalProposalInfo<Balance, Moment, CurrencyId>;
	}
}
