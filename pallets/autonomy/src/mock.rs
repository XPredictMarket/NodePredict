#![allow(clippy::from_over_into)]

use crate::{self as autonomy, *};

use frame_support::{
    construct_runtime, parameter_types,
    traits::{Hooks, Time},
};
use frame_system::{
    limits, mocking,
    offchain::{SendTransactionTypes, SigningTypes},
};
use sp_core::{
    sr25519::{self, Signature},
    H256,
};
use sp_keystore::{testing::KeyStore, SyncCryptoStore};
use sp_runtime::{
    testing::{Header, TestXt},
    traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
    ModuleId,
};
use std::{cell::RefCell, collections::HashMap};
use xpmrl_traits::{pool::LiquidityPool, system::ProposalSystem, tokens::Tokens};

pub type Extrinsic = TestXt<Call, ()>;
type UncheckedExtrinsic = mocking::MockUncheckedExtrinsic<Test>;
type Block = mocking::MockBlock<Test>;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
type BlockNumber = u64;
type Balance = u128;
type Public = <Signature as Verify>::Signer;

thread_local! {
    static PROPOSALS_WRAPPER: RefCell<ProposalsWrapper> = RefCell::new(ProposalsWrapper::new());
}

pub struct Timestamp;
impl Time for Timestamp {
    type Moment = BlockNumber;
    fn now() -> Self::Moment {
        System::block_number()
    }
}

// For testing the module, we construct a mock runtime.
construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Module, Call, Config, Storage, Event<T>},
        XPMRLTokens: xpmrl_tokens::{Module, Call, Storage, Event<T>},
        PalletBalances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
        AutonomyModule: autonomy::{Module, Call, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub BlockWeights: limits::BlockWeights = limits::BlockWeights::simple_max(1024);
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
    type AccountId = sr25519::Public;
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
    type SS58Prefix = ();
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

impl SigningTypes for Test {
    type Public = Public;
    type Signature = Signature;
}

impl<C> SendTransactionTypes<C> for Test
where
    Call: From<C>,
{
    type OverarchingCall = Call;
    type Extrinsic = Extrinsic;
}

pub type TokensOf<T> = <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::Tokens;
pub type CurrencyIdOf<T> =
    <TokensOf<T> as Tokens<<T as frame_system::Config>::AccountId>>::CurrencyId;
pub type BalanceOf<T> = <TokensOf<T> as Tokens<<T as frame_system::Config>::AccountId>>::Balance;

pub type TimeOf<T> = <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::Time;
pub type MomentOf<T> = <TimeOf<T> as Time>::Moment;

pub type ProposalIdOf<T> =
    <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::ProposalId;
type VersionIdOf<T> = <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::VersionId;

pub struct ProposalsWrapper {
    pub next_proposal_id: ProposalIdOf<Test>,
    pub interval_time: MomentOf<Test>,
    pub used_currency_id: HashMap<CurrencyIdOf<Test>, ()>,
    pub announcement_time: HashMap<ProposalIdOf<Test>, MomentOf<Test>>,
    pub create_time: HashMap<ProposalIdOf<Test>, MomentOf<Test>>,
    pub close_time: HashMap<ProposalIdOf<Test>, MomentOf<Test>>,
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
            interval_time: 5,
            used_currency_id: HashMap::<CurrencyIdOf<Test>, ()>::new(),
            announcement_time: HashMap::<ProposalIdOf<Test>, MomentOf<Test>>::new(),
            create_time: HashMap::<ProposalIdOf<Test>, MomentOf<Test>>::new(),
            close_time: HashMap::<ProposalIdOf<Test>, MomentOf<Test>>::new(),
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
    pub fn set_create_time(
        proposal_id: ProposalIdOf<Test>,
        time: MomentOf<Test>,
    ) -> Result<MomentOf<Test>, DispatchError> {
        PROPOSALS_WRAPPER.with(|wrapper| -> Result<MomentOf<Test>, DispatchError> {
            wrapper
                .borrow_mut()
                .create_time
                .insert(proposal_id, time);
            Ok(time)
        })
    }

    pub fn set_close_time(
        proposal_id: ProposalIdOf<Test>,
        time: MomentOf<Test>,
    ) -> Result<MomentOf<Test>, DispatchError> {
        PROPOSALS_WRAPPER.with(|wrapper| -> Result<MomentOf<Test>, DispatchError> {
            wrapper
                .borrow_mut()
                .close_time
                .insert(proposal_id, time);
            Ok(time)
        })
    }

    pub fn new_couple_proposal(
        who: AccountId,
        currency_id: CurrencyIdOf<Test>,
    ) -> Result<(), DispatchError> {
        PROPOSALS_WRAPPER.with(|wrapper| -> Result<(), DispatchError> {
            let id = wrapper.borrow().next_proposal_id;
            wrapper.borrow_mut().next_proposal_id =
                id.checked_add(1).ok_or(Error::<Test>::ProposalIdOverflow)?;
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

impl LiquidityPool<Test> for Proposals {
    fn get_proposal_minimum_interval_time() -> MomentOf<Test> {
        PROPOSALS_WRAPPER.with(|wrapper| -> MomentOf<Test> { wrapper.borrow().interval_time })
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
        _create_time: MomentOf<Test>,
        _close_time: MomentOf<Test>,
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

    fn proposal_create_time(
        proposal_id: ProposalIdOf<Test>,
    ) -> Result<MomentOf<Test>, DispatchError> {
        PROPOSALS_WRAPPER.with(|wrapper| -> Result<MomentOf<Test>, DispatchError> {
            match wrapper.borrow().create_time.get(&proposal_id) {
                Some(v) => Ok(*v),
                None => Err("ProposalIdNotExist".into()),
            }
        })
    }

    fn proposal_close_time(
        proposal_id: ProposalIdOf<Test>,
    ) -> Result<MomentOf<Test>, DispatchError> {
        PROPOSALS_WRAPPER.with(|wrapper| -> Result<MomentOf<Test>, DispatchError> {
            match wrapper.borrow().close_time.get(&proposal_id) {
                Some(v) => Ok(*v),
                None => Err("ProposalIdNotExist".into()),
            }
        })
    }

    fn get_proposal_state(
        proposal_id: ProposalIdOf<Test>,
    ) -> Result<ProposalStatus, DispatchError> {
        PROPOSALS_WRAPPER.with(|wrapper| -> Result<ProposalStatus, DispatchError> {
            match wrapper.borrow().proposal_state.get(&proposal_id) {
                Some(v) => Ok(*v),
                None => Err("ProposalIdNotExist".into()),
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

    fn proposal_owner(proposal_id: ProposalIdOf<Test>) -> Result<AccountId, DispatchError> {
        PROPOSALS_WRAPPER.with(|wrapper| -> Result<AccountId, DispatchError> {
            match wrapper.borrow().proposal_owner.get(&proposal_id) {
                Some(v) => Ok(*v),
                None => Err("ProposalIdNotExist".into()),
            }
        })
    }

    fn proposal_announcement_time(
        proposal_id: ProposalIdOf<Test>,
    ) -> Result<MomentOf<Test>, DispatchError> {
        PROPOSALS_WRAPPER.with(|wrapper| -> Result<MomentOf<Test>, DispatchError> {
            match wrapper.borrow().announcement_time.get(&proposal_id) {
                Some(v) => Ok(*v),
                None => Err("ProposalIdNotExist".into()),
            }
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
                    None => Err("ProposalIdNotExist".into()),
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
                .insert(proposal_id, ProposalStatus::ResultAnnouncement);
            Ok(())
        })
    }

    fn set_proposal_result_when_end(
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
                None => Err("ProposalIdNotExist".into()),
            }
        })
    }

    fn get_proposal_result(
        proposal_id: ProposalIdOf<Test>,
    ) -> Result<CurrencyIdOf<Test>, DispatchError> {
        PROPOSALS_WRAPPER.with(|wrapper| -> Result<CurrencyIdOf<Test>, DispatchError> {
            match wrapper.borrow().proposal_result.get(&proposal_id) {
                Some(val) => Ok(*val),
                None => Err("ProposalIdNotExist".into()),
            }
        })
    }
}

parameter_types! {
    pub const StakeCurrencyId: CurrencyId = 1;
    pub const AutonomyId: ModuleId = ModuleId(*b"xpgovern");
}

impl autonomy::Config for Test {
    type Event = Event;
    type Call = Call;
    type AuthorityId = crypto::OcwAuthId;
    type StakeCurrencyId = StakeCurrencyId;
    type Pool = Proposals;
    type CouplePool = Proposals;
    type AutonomyId = AutonomyId;
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

pub fn new_test_ext<F>(f: F) -> ()
where
    F: FnOnce(Vec<Public>) -> (),
{
    // Initialize key store
    let keystore = KeyStore::new();
    keystore.sr25519_generate_new(KEY_TYPE, None).unwrap();
    keystore.sr25519_generate_new(KEY_TYPE, None).unwrap();
    keystore.sr25519_generate_new(KEY_TYPE, None).unwrap();
    keystore.sr25519_generate_new(KEY_TYPE, None).unwrap();

    // get public key array from key store
    let public_key_array = keystore.sr25519_public_keys(KEY_TYPE);

    let tokens_genesis = xpmrl_tokens::GenesisConfig::<Test> {
        tokens: vec![
            (
                "Tether USD".as_bytes().to_vec(),
                "USDT".as_bytes().to_vec(),
                6,
            ),
            ("Bitcoin".as_bytes().to_vec(), "BTC".as_bytes().to_vec(), 8),
        ],
        balances: public_key_array
            .iter()
            .map(|x| (*x, 100000u128))
            .collect(),
    };

    let autonomy_genesis = autonomy::GenesisConfig::<Test> {
        minimal_stake_number: 1000,
        minimal_review_number: 100,
        minimal_report_number: 10000,
        review_cycle: 5,
        result_upload_cycle: 0,
        publicity_period: 0,
    };

    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    tokens_genesis.assimilate_storage(&mut t).unwrap();
    autonomy_genesis.assimilate_storage(&mut t).unwrap();

    let mut ext = sp_io::TestExternalities::new(t);

    ext.execute_with(|| System::set_block_number(1));
    ext.execute_with(|| f(public_key_array))
}
