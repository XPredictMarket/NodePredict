<p align="center">
  <img src="https://test-app.x-predict.com/static/media/logo.4869a3f6.svg" width="500">
</p>

<!-- TOC -->

-   [1. Overview](#1-overview)
-   [2. Build](#2-build)
-   [3. Run](#3-run)
    -   [Single Node Development Chain](#single-node-development-chain)
    -   [Multi-Node Local Testnet](#multi-node-local-testnet)
    -   [Types](#types)
-   [4. Main Processes](#4-main-processes)
    -   [Mint New Asset](#mint-new-asset)
    -   [Create Proposal](#create-proposal)
    -   [Set Status](#set-status)
    -   [Buy and Sell](#buy-and-sell)
    -   [Set Result](#set-result)
    -   [Retrieval](#retrieval)
-   [5. Pallet's Documentation](#5-pallets-documentation)
-   [6. X predict market's Documentation](#6-x-predict-markets-documentation)
-   [7. Test Guide](#7-test-guide)
    -   [Integration tests](#integration-tests)
-   [8. Proposal Workflow](#8-proposal-workflow)

<!-- /TOC -->

## 1. Overview

X predict market is a decentralized forecasting market. It has its own independent asset management and liquidation system, supports multi currency cross chain transactions. It also supports users to create proposals on the chain, vote, provide or remove liquidity and other operations, and supports users to carry out cross chain asset transfer and other operations.

## 2. Build

First of all, make sure that you have installed the [rust and wasm compilation environment](https://substrate.dev/docs/en/knowledgebase/getting-started/), [keygen tool](https://substrate.dev/docs/en/tutorials/start-a-private-network/keygen), And [GNU make](https://www.gnu.org/software/make/) before running.

-   Compiling wasm and executable files

    ```bash
    make build
    ```

-   Or just build wasm

    ```bash
    make wasm
    ```

-   Generate documentation

    ```bash
    make doc
    ```

-   Generate and open the document in the browser

    ```bash
    make open-doc
    ```

## 3. Run

### Single Node Development Chain

Purge any existing developer chain state:

```bash
make purge-dev
```

Start a development chain with:

```bash
make run-dev-tmp
```

### Multi-Node Local Testnet

If you want to use your own node to join the local test network, you need to start at least two verification nodes to start the local test network, and then access your own account.
First, start the first node:

```bash
cargo run --bin node-predict -- \
    --dev \
    --validator \
    --base-path /tmp/validator01 \
    --name validator01 \
    --rpc-port 2343 \
    --ws-port 3454 \
    --port 1232
```

Then we get the local node identity of the first node, such as `12d3kooweyoppncux8yx66ov9fjnrixwccxwdua2kj6vnc6idep`
Then start the second node, access the port and local node identification of the first node

```bash
cargo run --bin node-predict -- \
    --dev \
    --validator \
    --base-path /tmp/validator02 \
    --name validator02 \
    --rpc-port 2344 \
    --ws-port 3455 \
    --port 1233 \
    --bootnodes /ip4/127.0.0.1/tcp/1232/p2p/<bootnode id>
```

After starting at least two verification nodes, ensure that a corresponding number of key pairshave been generated([generate step](https://substrate.dev/docs/en/tutorials/start-a-private-network/keygen)), Then enter the key of the node into keysore. First, the mnemonics of `validator01` and public key information

```bash
curl --location --request POST 'http://localhost:2343' \
    --header 'Content-Type: application/json' \
    --data-raw '{
      "jsonrpc":"2.0",
      "method":"author_insertKey",
      "params": [
        "aura",
        "<mnemonic phrase>",
        "<sr25519 public key>"
      ]
      "id":1,
    }'
```

Then input the grandpa mnemonics of `validator01` and the public key information

```bash
curl --location --request POST 'http://localhost:2343' \
    --header 'Content-Type: application/json' \
    --data-raw '{
      "jsonrpc":"2.0",
      "method":"author_insertKey",
      "params": [
        "gran",
        "<mnemonic phrase>",
        "<ed25519 public key>"
      ]
      "id":1,
    }'
```

After inputting the key of `validator01`, run the same steps of inputting the key of `validator02` again(Need to change the rpc port to `2344`, like `http://localhost:2344`). After completing these steps, the multi node local test network has been started, and then you can join your own node.

```bash
cargo run --bin node-predict -- \
    --dev \
    --base-path /tmp/node01 \
    --name node01 \
    --ws-external \
    --rpc-external  \
    --bootnodes /ip4/127.0.0.1/tcp/1232/p2p/<bootnode id>
```

After starting the single node or test node, add JSON types in the developer settings of polkadotjs

### Types

In file [runtime/types.json](./runtime/types.json)
Then you can start a transaction on the chain.

## 4. Main Processes

### Mint New Asset

In the polkajs page, users can create new assets (they need to start predict-dev to use sudo permission), and then distribute the corresponding assets to their own accounts. At the same time, they should pay attention to giving users some local assets (for initiating transactions), so that they can create proposals or vote later.

### Create Proposal

Users can create their own proposals in the forecast market. They can enter the title of recent hot events, voting options, closing time, proposal category, proposal settlement asset category, proposal details, gas fee, and initial liquidity. After the proposal is submitted, it will enter the voting period. When the proposal is approved, other users in the chain can vote, add and remove liquidity.

### Set Status

After the proposal is successfully launched, the proposal will enter the original prediction stage and wait for review. In the initial stage, for the convenience of management, we will review it by a unified sudo administrator. In the later stage, we will submit it to the community for voting management. After the approval, the proposal will enter the formal prediction stage, and users can vote and add liquidity.

### Buy and Sell

Users can vote on different proposals according to their preferences. It depends on the asset class and the options. In addition, you can cancel the votes cast during the voting period, and you can also add liquidity.

### Set Result

After the voting, the proposal will enter the stage of waiting for the result. At this time, the result of the proposal will be uploaded by different staking nodes, and the final result will be decided by the majority vote. If the malicious node is found to upload the wrong result, the user can report it and initiate a new round of voting. After the arbitration is determined, the assets of the malicious pledge node will be punished to the reporting user and voting user. In order to facilitate the management in the early stage, the setting result will be carried out by sudo administrator, and in the later stage, it will be handed over to the community for voting governance.

### Retrieval

After the result is announced, the voters and liquidity providers participating in the proposal will obtain the corresponding settlement assets and option assets. According to the voting results, the voters can carry out the liquidation of option assets converted into settlement assets, and the liquidity providers can carry out the liquidation of removing liquidity and option assets converted into settlement assets, so as to obtain their final settlement assets. In order to encourage users to initiate proposals and create new liquidity, a portion of the total handling fee charged by the proposal will be set as a reward for the proposal initiator and liquidity provider.

## 5. Pallet's Documentation

-   [xpmrl_autonomy](https://rustdoc.x-predict.com/xpmrl_autonomy)
-   [xpmrl_couple](https://rustdoc.x-predict.com/xpmrl_couple)
-   [xpmrl_proposals](https://rustdoc.x-predict.com/xpmrl_proposals)
-   [xpmrl_tokens](https://rustdoc.x-predict.com/xpmrl_tokens)
-   [xpmrl_traits](https://rustdoc.x-predict.com/xpmrl_traits)
-   [xpmrl_utils](https://rustdoc.x-predict.com/xpmrl_utils)

## 6. X predict market's Documentation

[User Tutorials](/XPredictMarket/NodePredict/wiki/Tutorials)

## 7. Test Guide

### Integration tests

Each module in the project has its own independent test. On the premise of meeting the compilation environment, the overall test is carried out:

```bash
make test
```

Or use a separate pallet test command:

```bash
cargo test -p <pallet name>
```

The test cases provided include:

User 1 creates a proposal and checks the index, new asset ID, initial liquidity and other data in the proposal. After the proposal is successfully launched, check the status of the proposal.

Add new assets, verify the asset ID and data, issue and destroy new assets for user 1, and transfer assets and authorization between user 1 and user 2.

User 1 creates a proposal. After the proposal is passed, liquidity is added. User 2 buys and sells the proposal. The proposal sets the result. After the proposal is finished, user 1 removes the liquidity and user 1, 2 clears the assets.

Users can also add their own test cases.

## 8. Proposal Workflow

see [doc/workflow.md](/XPredictMarket/NodePredict/wiki/Workflow)
