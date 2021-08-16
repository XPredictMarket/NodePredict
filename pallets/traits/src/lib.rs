//! <!-- markdown-link-check-disable -->
//! # Couple
//!
//! Run `cargo doc --package xpmrl-traits --open` to view this pallet's documentation.
//!
//! Define common traits
//!

#![cfg_attr(not(feature = "std"), no_std)]

pub mod autonomy;
pub mod couple;
pub mod pool;
pub mod ruler;
pub mod system;
pub mod tokens;

use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Clone, Copy, RuntimeDebug, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Deserialize, Serialize))]
pub enum ProposalStatus {
    FormalPrediction,
    OriginalPrediction,
    WaitingForResults,
    ResultAnnouncement,
    Inlitigation,
    End,
}

#[derive(PartialEq, Eq, Clone, Copy, RuntimeDebug, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Deserialize, Serialize))]
pub enum RulerModule {
    PlatformDividend,
    CrossChainBurn,
    NotUsed,
}
