use codec::FullCodec;
use frame_support::pallet_prelude::MaybeSerializeDeserialize;
use sp_runtime::{traits::AtLeast32BitUnsigned, DispatchError};
use sp_std::{fmt::Debug, vec::Vec};

pub trait LiquidityPool<AccountId, ProposalId, Moment, CategoryId> {
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

	fn new_liquidity_pool(
		who: &AccountId,
		proposal_id: ProposalId,
		title: Vec<u8>,
		close_time: Moment,
		category_id: CategoryId,
		currency_id: Self::CurrencyId,
		optional: [Vec<u8>; 2],
		number: Self::Balance,
		earn_fee: u32,
		detail: Vec<u8>,
	) -> Result<(), DispatchError>;

	fn time(proposal_id: ProposalId) -> Result<(Moment, Moment), DispatchError>;
}
