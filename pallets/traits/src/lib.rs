#![cfg_attr(not(feature = "std"), no_std)]

pub mod pool;
pub mod tokens;

use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;

#[derive(PartialEq, Eq, Clone, Copy, RuntimeDebug, Encode, Decode)]
pub enum ProposalStatus {
	FormalPrediction,
	OriginalPrediction,
	WaitingForResults,
	ResultAnnouncement,
	Inlitigation,
	End,
}
