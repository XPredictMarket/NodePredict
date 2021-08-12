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
    -   [New Asset](#new-asset)
    -   [Mint Tokens](#mint-tokens)
    -   [Make Proposal](#make-proposal)
    -   [Vote](#vote)
    -   [Buy and Sell](#buy-and-sell)
    -   [Upload Result](#upload-result)
    -   [Retrieval](#retrieval)
-   [5. Pallet's Documentation](#5-pallets-documentation)
-   [6. X predict market's Documentation](#6-x-predict-markets-documentation)
-   [7. Test Guide](#7-test-guide)
    -   [Integration tests](#integration-tests)
-   [8. Proposal Workflow](#8-proposal-workflow)

<!-- /TOC -->

## 1. Overview

X Predict Market is a decentralized prediction market. The objective of X Predict Market is to enable users to participate in the prediction process in various ways by creating topics, discussing, predicting and approving the results.

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

If you want to use a local test network (private network), you need to set up at least two verification nodes and set up Aura and Grandpa verification accounts (because PoA consensus is used at the beginning of the chain, and the account generation uses the subkey tool). firstly, run the first validator node with the following command:
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

Here you must take note of the node identity on program output log, such as: `12D3KooWJvVUoAa7R8gjCSQ45x69Ahh3HcdVSH1dvpcA52vKawHL`, and the IP address `127.0.0.1` and p2p port `--port = 1232`, These values are for this specific example, but for your node, they will be different and required for other nodes to directly connect to it (without a bootnode in the `chain spec`).

Then run the second validator node and join the first validator node. This can be done by specifying the `--bootnodes` parameter, similar to the first validator node.

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

Once your two nodes are running, you will notice that no blocks are being produced. At this point, you need to add your keys(The key generated by the [subkey tool](https://substrate.dev/docs/en/tutorials/start-a-private-network/keyge)) into the keystore. You will add two types of keys for each node: `Aura` and `GRANDPA` keys. `Aura` keys are necessary for block production; `GRANDPA` keys are necessary for block finalization.You can insert a key into the keystore by using curl from the command line.

```bash
curl --location --request POST 'http://localhost:2343' \
    --header 'Content-Type: application/json' \
    --data-raw '{
      "jsonrpc":"2.0",
      "method":"author_insertKey",
      "params": [
        "aura|gran",
        "<mnemonic phrase>",
        "<public key>"
      ]
      "id":1,
    }'
```

After inserting the key into `validator01`, you can insert another key into the keystore of `validator02` in the same way (you need to change the rpc port to 2344, such as `http://localhost:2344`).

It should be noted that substrate nodes require a restart after inserting a `GRANDPA` key. You need to kill your nodes and restart them with the same commands you used previously.
After completing these steps, the multi-node local test network has been set up, and then other nodes (full node or light node) can be connected to the network through the following command:

```bash
cargo run --bin node-predict -- \
    --dev \
    --base-path /tmp/node01 \
    --name node01 \
    --ws-external \
    --rpc-external  \
    --bootnodes /ip4/127.0.0.1/tcp/1232/p2p/<bootnode id>
```

After starting a single node or local test node, if you need to test our features in polkadotjs, you need to add the JSON type in the developer settings of polkadotjs.

### Types

In file [runtime/types.json](./runtime/types.json)

Then you can start a transaction on the chain.

## 4. Main Processes

### New Asset

Since native currency is not allowed to make proposals, users are not allowed to make proposals when there is no other currency on the chain. At this time, organizations or individuals with sudo permissions are required to call the new asset method to create an asset.

### Mint Tokens

After the asset is created, the total supply of assets is zero, and the user balance is also zero. At this time, it is still not allowed to make proposals, so you need to use sudo permissions to mint the corresponding assets.

### Make Proposal

Users can make their own proposals in the proposal market. They can enter the title of recent hot events, voting options, closing time, proposal category, proposal settlement token, category, proposal details, transaction fee, and initial liquidity. After the proposal is submitted to the chain, it will enter the voting period.

### Vote

After the proposal is successfully put on the chain, the state of the proposal is the original prediction and it will enter the Predict Market after the user's vote is passed. When the proposal is approved, other users on the chain can buy, sell, add and remove liquidity in the Predict Market.

### Buy and Sell

Users can vote for different options according to their predicted result. It depends on the asset token and the options. In addition, users can cancel the votes during the prediction period, and they can also add liquidity.

### Upload Result

After the current time exceeds the closing time of the proposal, the prediction will enter the stage of waiting for the result. At this time, the result of the prediction will be uploaded by different governance nodes, and the final result will be decided by the majority vote. If the malicious node is found to upload the wrong result, the users can report it and initiate a new round of voting to finalize the result.

### Retrieval

After the result is announced, the voters and liquidity providers participating in the prediction will obtain the corresponding settlement tokens and option tokens according to the voting results, the voters can settle the option tokens with the settlement tokens, and the liquidity providers can withdraw the liquidity and convert the option tokens into settlement tokens. In order to encourage users to make proposals and add liquidity, a portion of the transaction fee charged by the prediction will be distributed among the prediction proposal maker and liquidity providers.

## 5. Pallet's Documentation

-   [xpmrl_autonomy](https://rustdoc.x-predict.com/xpmrl_autonomy)
-   [xpmrl_couple](https://rustdoc.x-predict.com/xpmrl_couple)
-   [xpmrl_proposals](https://rustdoc.x-predict.com/xpmrl_proposals)
-   [xpmrl_tokens](https://rustdoc.x-predict.com/xpmrl_tokens)
-   [xpmrl_traits](https://rustdoc.x-predict.com/xpmrl_traits)
-   [xpmrl_utils](https://rustdoc.x-predict.com/xpmrl_utils)

## 6. X predict market's Documentation

see [wiki/Tutorials](https://github.com/XPredictMarket/NodePredict/wiki/Tutorials)

## 7. Test Guide

### Integration tests

Each module in the project has its own independent test. With required compiling environment, the overall test is carried out:

```bash
make test
```

Or use a separate pallet test command:

```bash
cargo test -p <pallet name>
```

## 8. Proposal Workflow

see [wiki/Workflow](https://github.com/XPredictMarket/NodePredict/wiki/Workflow)
