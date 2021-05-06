# X Predict Market

## Getting Started

### Setup

First, complete the [basic Rust setup instructions](./doc/rust-setup.md).

If necessary, refer to the setup instructions at the
[Substrate Developer Hub](https://substrate.dev/docs/en/knowledgebase/getting-started/#manual-installation).

### Build

Once the development environment is set up, build the node template. This command will build the
[Wasm](https://substrate.dev/docs/en/knowledgebase/advanced/executor#wasm-execution) and
[native](https://substrate.dev/docs/en/knowledgebase/advanced/executor#native-execution) code:

```bash
cargo build --release
```

## Run

### Local Testnet

Polkadot (`rococo-v1` branch):

```bash
git clone -b rococo-v1 https://github.com/paritytech/polkadot.git

cargo build --release --features real-overseer

./target/release/polkadot build-spec --chain rococo-local --raw --disable-default-bootnode > rococo_local.json

./target/release/polkadot \
  --chain ./rococo_local.json \
  --tmp \
  --ws-port 9944 \
  --port 30333 \
  --validator \
  --alice

./target/release/polkadot \
  --chain ./rococo_local.json \
  --tmp \
  --ws-port 9955 \
  --port 30334 \
  --validator \
  --bob \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/<ALICE's peer id>
```

### Run Parachain

```bash
cargo build --release

./target/release/predict-rococo-collator \
  --collator \
  --tmp \
  --parachain-id 200 \
  --port 40333 \
  --ws-port 9844 \
  --ws-external \
  --rpc-cors all \
  --alice \
  -- \
  --execution wasm \
  --chain ../polkadot/rococo_local.json \
  --port 30343 \
  --ws-port 9977 \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/<ALICE's peer id>
```

### Registering on Local Relay Chain

In order to produce blocks you will need to register the parachain as detailed in the [Substrate Cumulus Workshop](https://substrate.dev/cumulus-workshop/#/en/3-parachains/2-register) by going to

`Developer -> sudo -> paraSudoWrapper -> sudoScheduleParaInitialize(id, genesis)`

- id: 200
- genesisHead: upload the file `para-200-genesis`
- validationCode: upload the file `para-200.wasm`
- parachain: Yes

The files you will need are in the `./resources` folder, if you need to build them because you modified the code you can use the following commands

```bash
mkdir resources
./target/release/predict-rococo-collator export-genesis-state --parachain-id 200 > ./resources/para-200-genesis
./target/release/predict-rococo-collator export-genesis-wasm > ./resources/para-200.wasm
```

### Types

```bash
{
    "PRC20": {
        "name": "Text",
        "symbol": "Text",
        "decimals": "u8"
    },
    "CategoryId": "u32",
    "Proposal": {
        "title": "Text",
        "category_id": "CategoryId",
        "detail": "Text"
    },
    "ProposalId": "u32",
    "ProposalIdOf": "ProposalId",
    "BalanceOf": "u128",
    "BalanceType": "BalanceOf",
    "CurrencyId": "u32",
    "CurrencyIdOf": "CurrencyId",
    "MomentOf": "u64",
    "VersionId": "u32",
    "Address": "MultiAddress",
    "LookupSource": "MultiAddress",
    "ProposalStatus": {
        "_enum": {
            "FormalPrediction": "Null",
            "OriginalPrediction": "Null",
            "WaitingForResults": "Null",
            "ResultAnnouncement": "Null",
            "Inlitigation": "Null",
            "End": "Null"
        }
    },
    "Status": "ProposalStatus",
    "ProposalInfo": {
        "categoryId": "CategoryId",
        "closeTime": "MomentOf",
        "detail": "Text",
        "liquidity": "BalanceOf",
        "no": "BalanceOf",
        "noName": "Text",
        "yes": "BalanceOf",
        "yesName": "Text",
        "title": "Text",
        "status": "ProposalStatus",
        "tokenId": "CurrencyId",
        "decimals": "u8"
    },
    "PersonalProposalInfo": {
        "title": "Text",
        "yesName": "Text",
        "noName": "Text",
        "currencyId": "CurrencyId",
        "yesCurrencyId": "CurrencyId",
        "noCurrencyId": "CurrencyId",
        "liquidityCurrencyId": "CurrencyId",
        "decimals": "u8",
        "yesDecimals": "u8",
        "noDecimals": "u8",
        "liquidityDecimals": "u8",
        "feeRateDecimals": "u8",
        "feeRate": "u32",
        "fee": "BalanceOf",
        "no": "BalanceOf",
        "yes": "BalanceOf",
        "total": "BalanceOf",
        "liquidity": "BalanceOf",
        "balance": "BalanceOf",
        "closeTime": "MomentOf",
        "status": "ProposalStatus"
    }
}
```

Then you can start a transaction on the chain.

## 4.Main Processes

### Mint New Asset

In the polkajs page, users can create new assets (they need to start predict-dev to use sudo permission), and then distribute the corresponding assets to their own accounts. At the same time, they should pay attention to giving users some local assets (for initiating transactions), so that they can create proposals or vote later.

### Create Proposal

Users can create their own proposals in the forecast market. They can enter the title of recent hot events, voting options, closing time, proposal category, proposal settlement asset category, proposal details, gas fee, and initial liquidity. After the proposal is submitted, it will enter the voting period. When the proposal is approved, other users in the chain can vote, add and remove liquidity.

```bash
ProposalInfo: {
    title,
    detail,
    OptionA,
    OptionB,
    CategoryId,
    closeTime,
    feerate,
    liquidity,
    tokenId,
    }
```

### Set Status

After the proposal is successfully launched, the proposal will enter the original prediction stage and wait for review. In the initial stage, for the convenience of management, we will review it by a unified sudo administrator. In the later stage, we will submit it to the community for voting management. After the approval, the proposal will enter the formal prediction stage, and users can vote and add liquidity.

### Buy and Sell

Users can vote on different proposals according to their preferences. It depends on the asset class and the options. In addition, you can cancel the votes cast during the voting period, and you can also add liquidity.

### Set Result

After the voting, the proposal will enter the stage of waiting for the result. At this time, the result of the proposal will be uploaded by different staking nodes, and the final result will be decided by the majority vote. If the malicious node is found to upload the wrong result, the user can report it and initiate a new round of voting. After the arbitration is determined, the assets of the malicious pledge node will be punished to the reporting user and voting user. In order to facilitate the management in the early stage, the setting result will be carried out by sudo administrator, and in the later stage, it will be handed over to the community for voting governance.

### Retrieval

After the result is announced, the voters and liquidity providers participating in the proposal will obtain the corresponding settlement assets and option assets. According to the voting results, the voters can carry out the liquidation of option assets converted into settlement assets, and the liquidity providers can carry out the liquidation of removing liquidity and option assets converted into settlement assets, so as to obtain their final settlement assets. In order to encourage users to initiate proposals and create new liquidity, a portion of the total handling fee charged by the proposal will be set as a reward for the proposal initiator and liquidity provider.

## 5.Pallet

### Couple:

About the reading of settlement assets, option assets and liquid assets.

#### Add liquidity and currency pairs

The user can add the liquidity of a user-defined number of assets in the proposal of formal forecast status by calling `add_liquidity(origin: OriginFor<T>,proposal_id: ProposalIdOf<T>,number: BalanceOf<T>)`

#### Remove liquidity

At the end of the proposal, the user can remove liquidity to obtain the corresponding settlement assets and option assets by calling `remove_liquidity(origin: OriginFor<T>,proposal_id: ProposalIdOf<T>,number: BalanceOf<T>)`

#### buy

Users can choose their favorite options to vote, and determine the number of option assets returned according to the purchase quantity. By calling `buy(origin: OriginFor<T>,proposal_id: ProposalIdOf<T>,optional_currency_id: CurrencyIdOf<T>,number: BalanceOf<T>)`

#### sell

If you want to cancel a vote in the formal prediction stage, you can sell it by calling `sell(origin: OriginFor<T>,proposal_id: ProposalIdOf<T>,optional_currency_id: CurrencyIdOf<T>,number: BalanceOf<T>)`.

#### Liquidation

After the proposal is finished, the user can call `retrieval(origin: OriginFor<T>,proposal_id: ProposalIdOf<T>,optional_currency_id: CurrencyIdOf<T>,number: BalanceOf<T>)` to itnitiate liquidation, and the system returns the user's corresponding settlement assets according to the proposal result and the user's corresponding number of option assets.

#### Set proposal results

When the proposal is waiting for the result, you can call `set_result(origin: OriginFor<T>,proposal_id: ProposalIdOf<T>,currency_id: CurrencyIdOf<T>)` through sudo sets the option final result of the proposal, and the status changes to end.

### Proposals:

About proposal initiation, storage and status change.

#### Launch a new proposal

The sponsor of the proposal can call `new_proposal(origin: OriginFor<T>,title: Vec<u8>,optional: [Vec<u8>; 2],close_time: MomentOf<T>,category_id: T::CategoryId,currency_id: CurrencyIdOf<T>,number: BalanceOf<T>,earn_fee: u32,detail: Vec<u8>)`launch a new proposal, and set the content and parameters.

#### Set proposal status

Sudo administrators can call `set_status(origin: OriginFor<T>,proposal_id: T::ProposalId,new_status: Status)` changes the status of the proposal.

### Tokens:

Asset creation and management module.

#### Create a new asset type

Sudo administrators can call `new_asset(origin: OriginFor<T>,name: Vec<u8>,symbol: Vec<u8>,decimals: u8)`. create new asset

#### Increase the specified asset ID of an account

Sudo administrators can call `mint(origin: OriginFor<T>,currency_id: T::CurrencyId,to: T::AccountId,number: BalanceType<T>)` add the assets with the specified asset ID of an account.

#### Destroy the asset with the specified asset ID of an account

The sudo administrator can call `burn(origin: OriginFor<T>,currency_id: T::CurrencyId,number: BalanceType<T>)` destroy the assets with the specified asset ID of an account.

#### The authorized person destroys assets of the authorized person's specified asset ID

By calling `burn_from(origin: OriginFor<T>,currency_id: T::CurrencyId,from: T::AccountId,number: BalanceType<T>,)` destroy the assets of the authorized person's specified asset ID.
####transfer
The initiator calls`transfer(origin: OriginFor<T>,currency_id: T::CurrencyId,to: T::AccountId,number: BalanceType<T>,)` transfer to a user.

#### Authorize the transfer

The initiator calls `transfer_from(origin: OriginFor<T>,currency_id: T::CurrencyId,from: T::AccountId,to: T::AccountId,number: BalanceType<T>,)`, transfer the assets of the authorized person with the specified asset ID to the specified account.

#### Authorize

The initiator calls `approve(origin: OriginFor<T>,currency_id: T::CurrencyId,spender: T::AccountId,number: BalanceType<T>,)` authorize a specified number of assets with the specified asset ID to an account.

### traits:

Module interface constraint features

### utils:

Macroinstruction used in project

# Test Guide

## Integration tests

Each module in the project has its own independent test. On the premise of meeting the compilation environment, the overall test is carried out:

```bash
Cargo test
```

Or use a separate pallet test command:

```bash
Cargo test -p xpmrl-pallet_name
```

The test cases provided include:
User 1 creates a proposal and checks the index, new asset ID, initial liquidity and other data in the proposal. After the proposal is successfully launched, check the status of the proposal.
Add new assets, verify the asset ID and data, issue and destroy new assets for user 1, and transfer assets and authorization between user 1 and user 2.
User 1 creates a proposal. After the proposal is passed, liquidity is added. User 2 buys and sells the proposal. The proposal sets the result. After the proposal is finished, user 1 removes the liquidity and user 1, 2 clears the assets.
Users can also add their own test cases.
