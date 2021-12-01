#![allow(clippy::from_over_into)]

use crate::{self as proposals, Error};
use frame_support::{
    dispatch::DispatchError,
    parameter_types,
    traits::{Hooks, Time},
};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    ModuleId,
};
use std::{cell::RefCell, collections::HashMap};
use xpmrl_traits::{
    couple::LiquidityCouple,
    pool::{LiquidityPool, LiquiditySubPool},
    system::ProposalSystem,
    tokens::Tokens,
    ProposalStatus,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
pub type AccountId = u64;
type Balance = u128;
pub type BlockNumber = u64;

thread_local! {
    static COUPLE_WRAPPER: RefCell<CoupleWrapper> = RefCell::new(CoupleWrapper::new());
}

pub struct Timestamp;
impl Time for Timestamp {
    type Moment = BlockNumber;
    fn now() -> Self::Moment {
        System::block_number()
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
    type BlockNumber = BlockNumber;
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

pub type TokensOf<T> = <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::Tokens;
type CurrencyIdOf<T> = <TokensOf<T> as Tokens<<T as frame_system::Config>::AccountId>>::CurrencyId;

type TimeOf<T> = <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::Time;
pub type MomentOf<T> = <TimeOf<T> as Time>::Moment;

type ProposalIdOf<T> = <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::ProposalId;

pub struct CoupleWrapper {
    pub announcement_time: HashMap<ProposalIdOf<Test>, MomentOf<Test>>,
    pub proposal_pair: HashMap<ProposalIdOf<Test>, (CurrencyIdOf<Test>, CurrencyIdOf<Test>)>,
    pub proposal_result: HashMap<ProposalIdOf<Test>, CurrencyIdOf<Test>>,
    pub proposal_lp: HashMap<ProposalIdOf<Test>, CurrencyIdOf<Test>>,
}

impl CoupleWrapper {
    fn new() -> CoupleWrapper {
        CoupleWrapper {
            announcement_time: HashMap::<ProposalIdOf<Test>, MomentOf<Test>>::new(),
            proposal_result: HashMap::<ProposalIdOf<Test>, CurrencyIdOf<Test>>::new(),
            proposal_pair:
                HashMap::<ProposalIdOf<Test>, (CurrencyIdOf<Test>, CurrencyIdOf<Test>)>::new(),
            proposal_lp: HashMap::<ProposalIdOf<Test>, CurrencyIdOf<Test>>::new(),
        }
    }
}

pub struct Couple;
impl Couple {
    pub fn new_couple_proposal(
        who: AccountId,
        currency_id: CurrencyIdOf<Test>,
        close_time: MomentOf<Test>,
    ) -> Result<(), DispatchError> {
        COUPLE_WRAPPER.with(|wrapper| -> Result<(), DispatchError> {
            let id = <ProposalsModule as LiquidityPool<Test>>::get_next_proposal_id()?;
            let decimals = <XPMRLTokens as Tokens<AccountId>>::decimals(currency_id)?;
            let lp_id = <XPMRLTokens as Tokens<AccountId>>::new_asset(
                "HAHA".as_bytes().to_vec(),
                "HAHA".as_bytes().to_vec(),
                decimals,
            )?;
            let yes_id = <XPMRLTokens as Tokens<AccountId>>::new_asset(
                "YES".as_bytes().to_vec(),
                "YES".as_bytes().to_vec(),
                decimals,
            )?;
            let no_id = <XPMRLTokens as Tokens<AccountId>>::new_asset(
                "YES".as_bytes().to_vec(),
                "YES".as_bytes().to_vec(),
                decimals,
            )?;
            <ProposalsModule as LiquidityPool<Test>>::append_used_currency(lp_id);
            <ProposalsModule as LiquidityPool<Test>>::append_used_currency(yes_id);
            <ProposalsModule as LiquidityPool<Test>>::append_used_currency(no_id);
            wrapper
                .borrow_mut()
                .proposal_pair
                .insert(id, (yes_id, no_id));
            <ProposalsModule as LiquidityPool<Test>>::init_proposal(
                id,
                &who,
                ProposalStatus::OriginalPrediction,
                <TimeOf<Test> as Time>::now(),
                close_time,
                1,
            );
            wrapper.borrow_mut().proposal_lp.insert(id, lp_id);
            Ok(())
        })
    }
}

impl LiquiditySubPool<Test> for Couple {
    fn finally_locked(_proposal_id: ProposalIdOf<Test>) -> Result<(), DispatchError> {
        Ok(())
    }
}

impl LiquidityCouple<Test> for Couple {
    fn proposal_pair(
        proposal_id: ProposalIdOf<Test>,
    ) -> Result<(CurrencyIdOf<Test>, CurrencyIdOf<Test>), DispatchError> {
        COUPLE_WRAPPER.with(
            |wrapper| -> Result<(CurrencyIdOf<Test>, CurrencyIdOf<Test>), DispatchError> {
                match wrapper.borrow().proposal_pair.get(&proposal_id) {
                    Some(val) => Ok(*val),
                    None => Err(Error::<Test>::ProposalIdNotExist.into()),
                }
            },
        )
    }

    fn set_proposal_result(
        proposal_id: ProposalIdOf<Test>,
        result: CurrencyIdOf<Test>,
    ) -> Result<(), DispatchError> {
        COUPLE_WRAPPER.with(|wrapper| -> Result<(), DispatchError> {
            wrapper
                .borrow_mut()
                .proposal_result
                .insert(proposal_id, result);
            <ProposalsModule as LiquidityPool<Test>>::set_proposal_state(
                proposal_id,
                ProposalStatus::End,
            )?;
            Ok(())
        })
    }

    fn set_proposal_result_when_end(
        proposal_id: ProposalIdOf<Test>,
        result: CurrencyIdOf<Test>,
    ) -> Result<(), DispatchError> {
        COUPLE_WRAPPER.with(|wrapper| -> Result<(), DispatchError> {
            wrapper
                .borrow_mut()
                .proposal_result
                .insert(proposal_id, result);
            <ProposalsModule as LiquidityPool<Test>>::set_proposal_state(
                proposal_id,
                ProposalStatus::End,
            )?;
            Ok(())
        })
    }

    fn get_proposal_result(
        proposal_id: ProposalIdOf<Test>,
    ) -> Result<CurrencyIdOf<Test>, DispatchError> {
        COUPLE_WRAPPER.with(|wrapper| -> Result<CurrencyIdOf<Test>, DispatchError> {
            match wrapper.borrow().proposal_result.get(&proposal_id) {
                Some(result) => Ok(*result),
                None => Err("ProposalNotResult".into()),
            }
        })
    }

    fn proposal_liquidate_currency_id(
        proposal_id: ProposalIdOf<Test>,
    ) -> Result<CurrencyIdOf<Test>, DispatchError> {
        COUPLE_WRAPPER.with(|wrapper| -> Result<CurrencyIdOf<Test>, DispatchError> {
            match wrapper.borrow().proposal_lp.get(&proposal_id) {
                Some(val) => Ok(*val),
                None => Err(Error::<Test>::ProposalIdNotExist.into()),
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
    pub const GovernanceCurrencyId: CurrencyIdOf<Test> = 1;
    pub const RewardId: ModuleId = ModuleId(*b"xpreward");
}

impl proposals::Config for Test {
    type Event = Event;
    type SubPool = Couple;
    type GovernanceCurrencyId = GovernanceCurrencyId;
    type RewardId = RewardId;
}

pub fn run_to_block<Module: Hooks<BlockNumber>>(n: BlockNumber) {
    while System::block_number() < n {
        Module::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Module::on_initialize(System::block_number());
    }
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
        balances: vec![(1, 100_000), (2, 100_000), (3, 100_000), (4, 100_000)],
    };
    let proposals_genesis = proposals::GenesisConfig::<Test> {
        expiration_time: 100,
        minimum_interval_time: 60 * 1_000,
        minimum_vote: 1_000,
        default_reward: 100,
    };
    proposals_genesis.assimilate_storage(&mut t).unwrap();
    tokens_genesis.assimilate_storage(&mut t).unwrap();
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}
