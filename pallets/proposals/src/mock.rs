use crate as proposals;
use frame_support::{dispatch::DispatchError, parameter_types, traits::Time};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
use std::cell::RefCell;
use xpmrl_traits::pool::LiquidityPool;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

thread_local! {
	static TIME: RefCell<u32> = RefCell::new(0);
}

pub struct Timestamp;
impl Time for Timestamp {
	type Moment = u32;

	fn now() -> Self::Moment {
		TIME.with(|v| *v.borrow())
	}
}

pub struct Proposal<CategoryId> {
	pub title: Vec<u8>,
	pub category_id: CategoryId,
	pub detail: Vec<u8>,
}

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
		ProposalsModule: proposals::{Module, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
}

type VersionId = u32;

parameter_types! {
	pub const EarnTradingFeeDecimals: u8 = 4;
	pub const CurrentLiquidateVersionId: VersionId = 1;
}

pub struct Couple;

impl LiquidityPool<u64, u32, u32, u32> for Couple {
	type CurrencyId = u32;
	type Balance = u128;

	fn new_liquidity_pool(
		_who: &u64,
		_proposal_id: u32,
		_title: Vec<u8>,
		_close_time: u32,
		_category_id: u32,
		_currency_id: Self::CurrencyId,
		_optional: [Vec<u8>; 2],
		_number: Self::Balance,
		_earn_fee: u32,
		_detail: Vec<u8>,
	) -> Result<(u32, u32, u32), DispatchError> {
		Ok((1, 2 , 3))
	}

	fn time(_proposal_id: u32) -> Result<(u32, u32), DispatchError> {
		Ok((0, 0))
	}
}

impl proposals::Config for Test {
	type Event = Event;
	type Time = Timestamp;
	type ProposalId = u32;
	type CategoryId = u32;
	type VersionId = VersionId;
	type EarnTradingFeeDecimals = EarnTradingFeeDecimals;
	type LiquidityPool = Couple;
	type CurrentLiquidateVersionId = CurrentLiquidateVersionId;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}
