#![allow(clippy::too_many_arguments)]

use hex_literal::hex;
use predict_runtime::{
    AccountId, AuraConfig, AutonomyConfig, BalancesConfig, CoupleConfig, GenesisConfig,
    GrandpaConfig, ProposalsConfig, RulerConfig, Signature, SudoConfig, SystemConfig, TokensConfig,
    WASM_BINARY,
};
use sc_service::ChainType;
use serde_json::{map::Map, value::Value};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{
    crypto::{Ss58Codec, UncheckedInto},
    sr25519, Pair, Public,
};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};

pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn get_account_id_from_address(address: &str) -> AccountId {
    if let Ok(account) = AccountId::from_ss58check(address) {
        account
    } else {
        Default::default()
    }
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
    (get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

pub fn properties() -> Option<Map<String, Value>> {
    let mut properties = Map::new();
    properties.insert("tokenSymbol".into(), vec!["PGAS"].into());
    properties.insert("tokenDecimals".into(), vec![12].into());
    Some(properties)
}

pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Development",
        // ID
        "dev",
        ChainType::Development,
        move || {
            predict_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![authority_keys_from_seed("Alice")],
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Pre-funded accounts
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    get_account_id_from_address("5DLqpJLQBSytLM2Zjgn9Ab8tcdkrvteSfx6yK3vTiwrEuFnp"),
                    get_account_id_from_address("5Fk7dfYpuWT8sK8BMcDYDHaz2H6ZuaJpwAHajURwkzAmRX7C"),
                    get_account_id_from_address("5CqffqfKmUmi9hBEsM8PVkAkw8PUYtjMPi1zrbrgsKa9u5Ui"),
                ],
                true,
                vec![
                    (
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        100000000000000000000000000,
                    ),
                    (
                        get_account_id_from_seed::<sr25519::Public>("Bob"),
                        100000000000000000000000000,
                    ),
                    (
                        get_account_id_from_address(
                            "5DLqpJLQBSytLM2Zjgn9Ab8tcdkrvteSfx6yK3vTiwrEuFnp",
                        ),
                        100000000000000000000000000,
                    ),
                    (
                        get_account_id_from_address(
                            "5Fk7dfYpuWT8sK8BMcDYDHaz2H6ZuaJpwAHajURwkzAmRX7C",
                        ),
                        100000000000000000000000000,
                    ),
                    (
                        get_account_id_from_address(
                            "5CqffqfKmUmi9hBEsM8PVkAkw8PUYtjMPi1zrbrgsKa9u5Ui",
                        ),
                        100000000000000000000000000,
                    ),
                ],
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Properties
        properties(),
        None,
    ))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Local Testnet wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Local Testnet",
        // ID
        "local_testnet",
        ChainType::Local,
        move || {
            predict_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![
                    authority_keys_from_seed("Alice"),
                    authority_keys_from_seed("Bob"),
                ],
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Pre-funded accounts
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                    get_account_id_from_address("5DLqpJLQBSytLM2Zjgn9Ab8tcdkrvteSfx6yK3vTiwrEuFnp"),
                    get_account_id_from_address("5Fk7dfYpuWT8sK8BMcDYDHaz2H6ZuaJpwAHajURwkzAmRX7C"),
                    get_account_id_from_address("5CqffqfKmUmi9hBEsM8PVkAkw8PUYtjMPi1zrbrgsKa9u5Ui"),
                ],
                true,
                vec![
                    (
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        100000000000000000000000000,
                    ),
                    (
                        get_account_id_from_seed::<sr25519::Public>("Bob"),
                        100000000000000000000000000,
                    ),
                    (
                        get_account_id_from_address(
                            "5DLqpJLQBSytLM2Zjgn9Ab8tcdkrvteSfx6yK3vTiwrEuFnp",
                        ),
                        100000000000000000000000000,
                    ),
                    (
                        get_account_id_from_address(
                            "5Fk7dfYpuWT8sK8BMcDYDHaz2H6ZuaJpwAHajURwkzAmRX7C",
                        ),
                        100000000000000000000000000,
                    ),
                    (
                        get_account_id_from_address(
                            "5CqffqfKmUmi9hBEsM8PVkAkw8PUYtjMPi1zrbrgsKa9u5Ui",
                        ),
                        100000000000000000000000000,
                    ),
                ],
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Properties
        properties(),
        // Extensions
        None,
    ))
}

pub fn test_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "mainnet test wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "testnet",
        // ID
        "testnet",
        ChainType::Live,
        move || {
            predict_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![
                    (
                        hex!["bc73ba3c456d4a80da66ef8bfbffbe9746fcced656d1910615000dbfb5d2b214"]
                            .unchecked_into(),
                        hex!["a37db576a726f6b2b186a2de87dd303e63646e1341be70c75c969146422bc865"]
                            .unchecked_into(),
                    ),
                    (
                        hex!["e08365c0d35799fecd6685b6f12b46178e63c4461c15d80675241b0694974839"]
                            .unchecked_into(),
                        hex!["88565e210a0364859fdb00103d7c1f5af6ff23358445e42ff864ca7fcd2f291d"]
                            .unchecked_into(),
                    ),
                ],
                // Sudo account
                hex!["ec548f5f534d715555648d2ca7d56a22be9c13b13f1678586bc8932189788656"].into(),
                // Pre-funded accounts
                vec![
                    get_account_id_from_address("5DLqpJLQBSytLM2Zjgn9Ab8tcdkrvteSfx6yK3vTiwrEuFnp"),
                    get_account_id_from_address("5Fk7dfYpuWT8sK8BMcDYDHaz2H6ZuaJpwAHajURwkzAmRX7C"),
                    get_account_id_from_address("5CqffqfKmUmi9hBEsM8PVkAkw8PUYtjMPi1zrbrgsKa9u5Ui"),
                    hex!["ec548f5f534d715555648d2ca7d56a22be9c13b13f1678586bc8932189788656"].into(),
                ],
                true,
                vec![
                    (
                        get_account_id_from_address(
                            "5DLqpJLQBSytLM2Zjgn9Ab8tcdkrvteSfx6yK3vTiwrEuFnp",
                        ),
                        100000000000000000000000000,
                    ),
                    (
                        get_account_id_from_address(
                            "5Fk7dfYpuWT8sK8BMcDYDHaz2H6ZuaJpwAHajURwkzAmRX7C",
                        ),
                        100000000000000000000000000,
                    ),
                    (
                        get_account_id_from_address(
                            "5CqffqfKmUmi9hBEsM8PVkAkw8PUYtjMPi1zrbrgsKa9u5Ui",
                        ),
                        100000000000000000000000000,
                    ),
                ],
            )
        },
        // Bootnodes
        // "/ip4/127.0.0.1/tcp/30333/p2p/QmSk5HQbn6LhUwDiNMseVUjuRYhEtYj4aUZ6WfWoGURpdV".parse().unwrap()
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Properties
        properties(),
        // Extensions
        None,
    ))
}

pub fn mainnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "mainnet wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "mainnet",
        // ID
        "mainnet",
        ChainType::Live,
        move || {
            predict_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![
                    (
                        hex!["bc73ba3c456d4a80da66ef8bfbffbe9746fcced656d1910615000dbfb5d2b214"]
                            .unchecked_into(),
                        hex!["a37db576a726f6b2b186a2de87dd303e63646e1341be70c75c969146422bc865"]
                            .unchecked_into(),
                    ),
                    (
                        hex!["e08365c0d35799fecd6685b6f12b46178e63c4461c15d80675241b0694974839"]
                            .unchecked_into(),
                        hex!["88565e210a0364859fdb00103d7c1f5af6ff23358445e42ff864ca7fcd2f291d"]
                            .unchecked_into(),
                    ),
                ],
                // Sudo account
                hex!["ec548f5f534d715555648d2ca7d56a22be9c13b13f1678586bc8932189788656"].into(),
                // Pre-funded accounts
                vec![
                    hex!["ec548f5f534d715555648d2ca7d56a22be9c13b13f1678586bc8932189788656"].into(),
                ],
                true,
                vec![],
            )
        },
        // Bootnodes
        // "/ip4/127.0.0.1/tcp/30333/p2p/QmSk5HQbn6LhUwDiNMseVUjuRYhEtYj4aUZ6WfWoGURpdV".parse().unwrap()
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Properties
        properties(),
        None,
    ))
}

/// Configure initial storage state for FRAME modules.
fn predict_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    _enable_println: bool,
    balances: Vec<(AccountId, u128)>,
) -> GenesisConfig {
    GenesisConfig {
        frame_system: Some(SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        }),
        pallet_balances: Some(BalancesConfig {
            // Configure endowed accounts with initial balance of 1 << 60.
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1 << 60))
                .collect(),
        }),
        pallet_aura: Some(AuraConfig {
            authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
        }),
        pallet_grandpa: Some(GrandpaConfig {
            authorities: initial_authorities
                .iter()
                .map(|x| (x.1.clone(), 1))
                .collect(),
        }),
        pallet_sudo: Some(SudoConfig {
            // Assign network admin rights.
            key: root_key.clone(),
        }),
        tokens: Some(TokensConfig {
            tokens: vec![
                ("P POT", "PPOT", 8),
                ("Test Coin", "TestC", 8),
                ("P Ethereum", "PETH", 18),
            ]
            .iter()
            .map(|x| {
                (
                    <&str>::clone(&x.0).as_bytes().to_vec(),
                    <&str>::clone(&x.1).as_bytes().to_vec(),
                    x.2,
                )
            })
            .collect(),
            balances,
        }),
        proposals: Some(ProposalsConfig {
            expiration_time: 7 * 24 * 60 * 60 * 1000,
            minimum_interval_time: 10 * 60 * 1000,
            minimum_vote: 10_000 * 100_000_000,
            default_reward: 10 * 100_000_000,
        }),
        couple: Some(CoupleConfig {
            liquidity_provider_fee_rate: 9000,
            withdrawal_fee_rate: 50,
        }),
        autonomy: Some(AutonomyConfig {
            minimal_number: 10000 * 100000000,
            publicity_interval: 2 * 24 * 60 * 60 * 1000, // 2 days
            report_interval: 3 * 24 * 60 * 60 * 1000,
        }),
        ruler: Some(RulerConfig {
            dividend_address: root_key,
        }),
    }
}
