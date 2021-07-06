use hex_literal::hex;
use predict_runtime::{
    AccountId, AuraConfig, AutonomyConfig, BalancesConfig, GenesisConfig, GrandpaConfig,
    ProposalsConfig, SudoConfig, SystemConfig, TokensConfig, WASM_BINARY,
};
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::crypto::{Ss58Codec, UncheckedInto};
use sp_finality_grandpa::AuthorityId as GrandpaId;

pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

pub fn get_account_id_from_address(address: &str) -> AccountId {
    if let Ok(account) = AccountId::from_ss58check(address) {
        account
    } else {
        Default::default()
    }
}

pub fn mainnet_test_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "mainnet test wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "main testnet",
        // ID
        "main_testnet",
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
                vec![("Test Coin", "TestC", 8)],
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
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Properties
        None,
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
        None,
        // Extensions
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
    tokens: Vec<(&str, &str, u8)>,
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
            key: root_key,
        }),
        tokens: Some(TokensConfig {
            tokens: tokens
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
            expiration_time: 3 * 24 * 60 * 60 * 1000,
            liquidity_provider_fee_rate: 9000,
            minimum_interval_time: 10 * 60 * 1000,
        }),
        autonomy: Some(AutonomyConfig {
            minimal_number: 10000 * 100000000,
            interval: 2 * 24 * 60 * 60 * 1000, // 2 days
        }),
    }
}
