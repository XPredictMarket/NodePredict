use crate as mining;
use frame_support::{parameter_types, traits::Time};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    DispatchError, ModuleId,
};
use std::{cell::RefCell, collections::HashMap};
use xpmrl_traits::{
    couple::LiquidityCouple, system::ProposalSystem, tokens::Tokens, ProposalStatus,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type Balance = u128;
pub type AccountId = u64;

thread_local! {
    static PROPOSALS_WRAPPER: RefCell<ProposalsWrapper> = RefCell::new(ProposalsWrapper::new());
}

pub struct Timestamp;
impl Time for Timestamp {
    type Moment = u64;

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
        XPMRLTokens: xpmrl_tokens::{Module, Call, Storage, Event<T>},
        PalletBalances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
        MiningModule: mining::{Module, Call, Storage, Event<T>},
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

pub type TokensOf<T> = <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::Tokens;
pub type CurrencyIdOf<T> =
    <TokensOf<T> as Tokens<<T as frame_system::Config>::AccountId>>::CurrencyId;
pub type BalanceOf<T> = <TokensOf<T> as Tokens<<T as frame_system::Config>::AccountId>>::Balance;

pub type ProposalIdOf<T> =
    <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::ProposalId;

pub struct ProposalsWrapper {
    pub next_proposal_id: ProposalIdOf<Test>,
    pub used_currency_id: HashMap<CurrencyIdOf<Test>, ()>,
    pub proposal_state: HashMap<ProposalIdOf<Test>, ProposalStatus>,
    pub proposal_owner: HashMap<ProposalIdOf<Test>, AccountId>,
    pub proposal_pair: HashMap<ProposalIdOf<Test>, (CurrencyIdOf<Test>, CurrencyIdOf<Test>)>,
    pub proposal_result: HashMap<ProposalIdOf<Test>, CurrencyIdOf<Test>>,
    pub proposal_lp: HashMap<ProposalIdOf<Test>, CurrencyIdOf<Test>>,
}

impl ProposalsWrapper {
    fn new() -> ProposalsWrapper {
        ProposalsWrapper {
            next_proposal_id: 0,
            used_currency_id: HashMap::<CurrencyIdOf<Test>, ()>::new(),
            proposal_state: HashMap::<ProposalIdOf<Test>, ProposalStatus>::new(),
            proposal_owner: HashMap::<ProposalIdOf<Test>, AccountId>::new(),
            proposal_result: HashMap::<ProposalIdOf<Test>, CurrencyIdOf<Test>>::new(),
            proposal_pair:
                HashMap::<ProposalIdOf<Test>, (CurrencyIdOf<Test>, CurrencyIdOf<Test>)>::new(),
            proposal_lp: HashMap::<ProposalIdOf<Test>, CurrencyIdOf<Test>>::new(),
        }
    }
}

pub struct Proposals;

impl Proposals {
    pub fn new_couple_proposal(
        who: AccountId,
        currency_id: CurrencyIdOf<Test>,
        number: BalanceOf<Test>,
    ) -> Result<(), DispatchError> {
        PROPOSALS_WRAPPER.with(|wrapper| -> Result<(), DispatchError> {
            let id = wrapper.borrow().next_proposal_id;
            wrapper.borrow_mut().next_proposal_id =
                id.checked_add(1).ok_or("ProposalIdOverflow")?;
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
            <XPMRLTokens as Tokens<AccountId>>::mint(lp_id, &who, number)?;
            wrapper.borrow_mut().used_currency_id.insert(lp_id, ());
            wrapper.borrow_mut().used_currency_id.insert(yes_id, ());
            wrapper.borrow_mut().used_currency_id.insert(no_id, ());
            wrapper
                .borrow_mut()
                .proposal_pair
                .insert(id, (yes_id, no_id));
            wrapper
                .borrow_mut()
                .proposal_state
                .insert(id, ProposalStatus::OriginalPrediction);
            wrapper.borrow_mut().proposal_lp.insert(id, lp_id);
            wrapper.borrow_mut().proposal_owner.insert(id, who);
            Ok(())
        })
    }
}

impl LiquidityCouple<Test> for Proposals {
    fn proposal_pair(
        proposal_id: ProposalIdOf<Test>,
    ) -> Result<(CurrencyIdOf<Test>, CurrencyIdOf<Test>), DispatchError> {
        PROPOSALS_WRAPPER.with(
            |wrapper| -> Result<(CurrencyIdOf<Test>, CurrencyIdOf<Test>), DispatchError> {
                match wrapper.borrow().proposal_pair.get(&proposal_id) {
                    Some(val) => Ok(*val),
                    None => Err("ProposalIdNotExist")?,
                }
            },
        )
    }

    fn set_proposal_result(
        proposal_id: ProposalIdOf<Test>,
        result: CurrencyIdOf<Test>,
    ) -> Result<(), DispatchError> {
        PROPOSALS_WRAPPER.with(|wrapper| -> Result<(), DispatchError> {
            wrapper
                .borrow_mut()
                .proposal_result
                .insert(proposal_id, result);
            wrapper
                .borrow_mut()
                .proposal_state
                .insert(proposal_id, ProposalStatus::End);
            Ok(())
        })
    }

    fn proposal_liquidate_currency_id(
        proposal_id: ProposalIdOf<Test>,
    ) -> Result<CurrencyIdOf<Test>, DispatchError> {
        PROPOSALS_WRAPPER.with(|wrapper| -> Result<CurrencyIdOf<Test>, DispatchError> {
            match wrapper.borrow().proposal_lp.get(&proposal_id) {
                Some(val) => Ok(*val),
                None => Err("ProposalIdNotExist")?,
            }
        })
    }

    fn get_proposal_result(
        proposal_id: ProposalIdOf<Test>,
    ) -> Result<CurrencyIdOf<Test>, DispatchError> {
        PROPOSALS_WRAPPER.with(|wrapper| -> Result<CurrencyIdOf<Test>, DispatchError> {
            match wrapper.borrow().proposal_result.get(&proposal_id) {
                Some(val) => Ok(*val),
                None => Err("ProposalIdNotExist")?,
            }
        })
    }
}

parameter_types! {
    pub const MiningModuleId: ModuleId = ModuleId(*b"xpmining");
    pub const MineTokenCurrencyId: CurrencyId = 1;
    pub const ScaleUpper: Balance = 100_000_000;
}

impl mining::Config for Test {
    type Event = Event;
    type ModuleId = MiningModuleId;
    type MineTokenCurrencyId = MineTokenCurrencyId;
    type ScaleUpper = ScaleUpper;
    type CouplePool = Proposals;
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
        balances: vec![(1, 1000000000), (2, 1000000000)],
    };
    tokens_genesis.assimilate_storage(&mut t).unwrap();
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}
