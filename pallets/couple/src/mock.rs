use crate::{self as couple, Error};
use frame_support::{dispatch::DispatchError, parameter_types, traits::Time};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    ModuleId,
};
use std::{cell::RefCell, collections::HashMap};
use xpmrl_traits::{pool::LiquidityPool, system::ProposalSystem, tokens::Tokens, ProposalStatus};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type AccountId = u64;
type Balance = u128;

thread_local! {
    static TIME: RefCell<u32> = RefCell::new(0);
    static PROPOSALS_WRAPPER: RefCell<ProposalsWrapper> = RefCell::new(ProposalsWrapper::new());
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
        PalletBalances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
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

parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
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

type TokensOf<T> = <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::Tokens;
type CurrencyIdOf<T> = <TokensOf<T> as Tokens<<T as frame_system::Config>::AccountId>>::CurrencyId;

type TimeOf<T> = <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::Time;
type MomentOf<T> = <TimeOf<T> as Time>::Moment;

type ProposalIdOf<T> = <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::ProposalId;
type VersionIdOf<T> = <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::VersionId;

pub struct ProposalsWrapper {
    pub next_proposal_id: ProposalIdOf<Test>,
    pub used_currency_id: HashMap<CurrencyIdOf<Test>, ()>,
    pub proposal_state: HashMap<ProposalIdOf<Test>, ProposalStatus>,
    pub proposal_owner: HashMap<ProposalIdOf<Test>, AccountId>,
}

impl ProposalsWrapper {
    fn new() -> ProposalsWrapper {
        ProposalsWrapper {
            next_proposal_id: 0,
            used_currency_id: HashMap::<CurrencyIdOf<Test>, ()>::new(),
            proposal_state: HashMap::<ProposalIdOf<Test>, ProposalStatus>::new(),
            proposal_owner: HashMap::<ProposalIdOf<Test>, AccountId>::new(),
        }
    }
}

pub struct Proposals;
impl LiquidityPool<Test> for Proposals {
    fn get_proposa_minimum_interval_time() -> MomentOf<Test> {
        0
    }

    fn is_currency_id_used(currency_id: CurrencyIdOf<Test>) -> bool {
        PROPOSALS_WRAPPER.with(|wrapper| -> bool {
            wrapper.borrow().used_currency_id.contains_key(&currency_id)
        })
    }

    fn get_next_proposal_id() -> Result<ProposalIdOf<Test>, DispatchError> {
        PROPOSALS_WRAPPER.with(|wrapper| -> Result<ProposalIdOf<Test>, DispatchError> {
            let id = wrapper.borrow().next_proposal_id;
            wrapper.borrow_mut().next_proposal_id =
                id.checked_add(1).ok_or(Error::<Test>::ProposalIdOverflow)?;
            Ok(id)
        })
    }

    fn init_proposal(
        proposal_id: ProposalIdOf<Test>,
        owner: &AccountId,
        state: ProposalStatus,
        _version: VersionIdOf<Test>,
    ) {
        PROPOSALS_WRAPPER.with(|wrapper| -> () {
            wrapper
                .borrow_mut()
                .proposal_owner
                .insert(proposal_id, *owner);
            wrapper
                .borrow_mut()
                .proposal_state
                .insert(proposal_id, state);
        })
    }

    fn append_used_currency(currency_id: CurrencyIdOf<Test>) {
        PROPOSALS_WRAPPER.with(|wrapper| -> () {
            wrapper
                .borrow_mut()
                .used_currency_id
                .insert(currency_id, ());
        })
    }

    fn max_proposal_id() -> ProposalIdOf<Test> {
        PROPOSALS_WRAPPER
            .with(|wrapper| -> ProposalIdOf<Test> { wrapper.borrow().next_proposal_id })
    }

    fn proposal_automatic_expiration_time() -> MomentOf<Test> {
        0
    }

    fn get_proposal_state(
        proposal_id: ProposalIdOf<Test>,
    ) -> Result<ProposalStatus, DispatchError> {
        PROPOSALS_WRAPPER.with(|wrapper| -> Result<ProposalStatus, DispatchError> {
            match wrapper.borrow().proposal_state.get(&proposal_id) {
                Some(v) => Ok(*v),
                None => Err(Error::<Test>::ProposalIdNotExist)?,
            }
        })
    }

    fn set_proposal_state(
        proposal_id: ProposalIdOf<Test>,
        new_state: ProposalStatus,
    ) -> Result<ProposalStatus, DispatchError> {
        PROPOSALS_WRAPPER.with(|wrapper| -> Result<ProposalStatus, DispatchError> {
            wrapper
                .borrow_mut()
                .proposal_state
                .insert(proposal_id, new_state);
            Ok(new_state)
        })
    }

    fn get_earn_trading_fee_decimals() -> u8 {
        4
    }

    fn proposal_liquidity_provider_fee_rate() -> u32 {
        200
    }

    fn proposal_owner(proposal_id: ProposalIdOf<Test>) -> Result<AccountId, DispatchError> {
        PROPOSALS_WRAPPER.with(|wrapper| -> Result<AccountId, DispatchError> {
            match wrapper.borrow().proposal_owner.get(&proposal_id) {
                Some(v) => Ok(*v),
                None => Err(Error::<Test>::ProposalIdNotExist)?,
            }
        })
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
    pub const CurrentLiquidateVersionId: VersionId = 1;
}

impl couple::Config for Test {
    type Event = Event;
    type Pool = Proposals;
    type CurrentLiquidateVersionId = CurrentLiquidateVersionId;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    let tokens_genesis = xpmrl_tokens::GenesisConfig::<Test> {
        tokens: vec![
            (
                "Tether USD".as_bytes().to_vec(),
                "USDT".as_bytes().to_vec(),
                6,
            ),
            ("Bitcoin".as_bytes().to_vec(), "BTC".as_bytes().to_vec(), 8),
        ],
        balances: vec![(1, 100000), (2, 31250)],
    };
    tokens_genesis.assimilate_storage(&mut t).unwrap();
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}
