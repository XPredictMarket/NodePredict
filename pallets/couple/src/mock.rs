use crate as couple;
use frame_support::{
	parameter_types,
	traits::{GenesisBuild, Time},
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	ModuleId,
};
use std::cell::RefCell;

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

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
		CoupleModule: couple::{Module, Call, Storage, Event<T>},
		XPMRLTokens: xpmrl_tokens::{Module, Call, Storage, Event<T>},
		XPMRLProposals: xpmrl_proposals::{Module, Call, Storage, Event<T>},
		PalletBalances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
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
	type AccountData = pallet_balances::AccountData<u128>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Test {
	type Balance = u128;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = frame_system::Pallet<Test>;
	type MaxLocks = ();
	type WeightInfo = ();
}

type CurrencyId = u32;

parameter_types! {
	pub const NativeCurrencyId: CurrencyId = 0;
	pub const TokensModuleId: ModuleId = ModuleId(*b"xptokens");
}

impl xpmrl_tokens::Config for Test {
	type Event = Event;
	type CurrencyId = CurrencyId;
	type Currency = PalletBalances;
	type NativeCurrencyId = NativeCurrencyId;
	type ModuleId = TokensModuleId;
}

type VersionId = u32;

parameter_types! {
	pub const EarnTradingFeeDecimals: u8 = 4;
	pub const CurrentLiquidateVersionId: VersionId = 1;
}

impl xpmrl_proposals::Config for Test {
	type Event = Event;
	type Time = Timestamp;
	type ProposalId = u32;
	type CategoryId = u32;
	type VersionId = VersionId;
	type EarnTradingFeeDecimals = EarnTradingFeeDecimals;
	type LiquidityPool = CoupleModule;
	type CurrentLiquidateVersionId = CurrentLiquidateVersionId;
}

impl couple::Config for Test {
	type Event = Event;
	type Tokens = XPMRLTokens;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();
	let tokens_genesis = xpmrl_tokens::GenesisConfig::<Test> {
		tokens: vec![
			("USDT".as_bytes().to_vec(), 6),
			("BTC".as_bytes().to_vec(), 8),
		],
		balances: vec![(1, 100000), (2, 31250)],
	};
	tokens_genesis.assimilate_storage(&mut t).unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}
