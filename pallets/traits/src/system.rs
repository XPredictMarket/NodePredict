use crate::tokens::Tokens;
use codec::FullCodec;
use frame_support::{pallet_prelude::MaybeSerializeDeserialize, traits::Time};
use sp_runtime::traits::AtLeast32BitUnsigned;
use sp_std::fmt::Debug;

pub trait ProposalSystem<AccountId> {
    type Time: Time;
    type ProposalId: FullCodec
        + Eq
        + PartialEq
        + Copy
        + MaybeSerializeDeserialize
        + Debug
        + AtLeast32BitUnsigned;
    type CategoryId: FullCodec
        + Eq
        + PartialEq
        + Copy
        + MaybeSerializeDeserialize
        + Debug
        + AtLeast32BitUnsigned;
    type VersionId: FullCodec
        + Eq
        + PartialEq
        + Copy
        + MaybeSerializeDeserialize
        + Debug
        + AtLeast32BitUnsigned;
    type Tokens: Tokens<AccountId>;
}
