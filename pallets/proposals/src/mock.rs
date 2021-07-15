use crate as proposals;
use frame_support::{
    dispatch::DispatchError,
    parameter_types,
    traits::{GenesisBuild, Time},
};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    ModuleId,
};
use std::cell::RefCell;
use xpmrl_traits::{
    couple::LiquidityCouple, pool::LiquiditySubPool, system::ProposalSystem, tokens::Tokens,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type AccountId = u64;
type Balance = u128;

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
        ProposalsModule: proposals::{Module, Call, Storage, Event<T>},
        XPMRLTokens: xpmrl_tokens::{Module, Call, Config<T>, Storage, Event<T>},
        Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
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
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
}

type TokensOf<T> = <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::Tokens;
type CurrencyIdOf<T> = <TokensOf<T> as Tokens<<T as frame_system::Config>::AccountId>>::CurrencyId;

type TimeOf<T> = <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::Time;
type MomentOf<T> = <TimeOf<T> as Time>::Moment;

type ProposalIdOf<T> = <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::ProposalId;
pub struct Couple;
impl LiquiditySubPool<Test> for Couple {
    fn finally_locked(_proposal_id: ProposalIdOf<Test>) -> Result<(), DispatchError> {
        Ok(())
    }
}

impl LiquidityCouple<Test> for Couple {
    fn proposal_announcement_time(
        _proposal_id: ProposalIdOf<Test>,
    ) -> Result<MomentOf<Test>, DispatchError> {
        Ok(0)
    }

    fn proposal_pair(
        _proposal_id: ProposalIdOf<Test>,
    ) -> Result<(CurrencyIdOf<Test>, CurrencyIdOf<Test>), DispatchError> {
        Ok((1, 1))
    }

    fn set_proposal_result(
        _proposal_id: ProposalIdOf<Test>,
        _result: CurrencyIdOf<Test>,
    ) -> Result<(), DispatchError> {
        Ok(())
    }

    fn proposal_liquidate_currency_id(
        _proposal_id: ProposalIdOf<Test>,
    ) -> Result<CurrencyIdOf<Test>, DispatchError> {
        Ok(1)
    }
}

type ProposalId = u32;
type VersionId = u32;
type CategoryId = u32;

impl ProposalSystem<AccountId> for Test {
    type ProposalId = ProposalId;
    type CategoryId = CategoryId;
    type Tokens = XPMRLTokens;
    type Time = Timestamp;
    type VersionId = VersionId;
}

parameter_types! {
    pub const ExistentialDeposit: u128 = 500;
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Test {
    type Balance = Balance;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = frame_system::Pallet<Test>;
    type MaxLocks = ();
    type WeightInfo = ();
}

parameter_types! {
    pub const NativeCurrencyId: CurrencyId = 0;
    pub const TokensModuleId: ModuleId = ModuleId(*b"xptokens");
}

pub type CurrencyId = u32;

impl xpmrl_tokens::Config for Test {
    type Event = Event;
    type CurrencyId = CurrencyId;
    type Currency = Balances;
    type NativeCurrencyId = NativeCurrencyId;
    type ModuleId = TokensModuleId;
}

parameter_types! {
    pub const EarnTradingFeeDecimals: u8 = 4;
}

impl proposals::Config for Test {
    type Event = Event;
    type EarnTradingFeeDecimals = EarnTradingFeeDecimals;
    type SubPool = Couple;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    let tokens_genesis = xpmrl_tokens::GenesisConfig::<Test> {
        tokens: vec![],
        balances: vec![],
    };
    let proposals_genesis = proposals::GenesisConfig {
        expiration_time: 3 * 24 * 60 * 60 * 1000,
        liquidity_provider_fee_rate: 9000,
        minimum_interval_time: 60 * 1000,
    };
    GenesisBuild::<Test>::assimilate_storage(&proposals_genesis, &mut t).unwrap();
    tokens_genesis.assimilate_storage(&mut t).unwrap();
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}
