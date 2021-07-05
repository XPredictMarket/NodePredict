# X Predict Market
## 1.Overview
X predict market is a decentralized forecasting market. It has its own independent asset management and liquidation system, supports multi currency cross chain transactions. It also supports users to create proposals on the chain, vote, provide or remove liquidity and other operations, and supports users to carry out cross chain asset transfer and other operations.
## 2.Build
First of all, make sure that you have [installed the rust and wasm compilation environment before running](https://substrate.dev/docs/en/knowledgebase/getting-started/)，And [create your own key](https://substrate.dev/docs/en/tutorials/start-a-private-network/keygen)

Compiling wasm and local environment
```bash
cargo build --release
```

## 3.Run
### Single Node Development Chain
Purge any existing developer chain state:

```bash
./target/release/node-predict-dev purge-chain --dev
```

Start a development chain with:

```bash
./target/release/node-predict-dev --dev --tmp
```
### Multi-Node Local Testnet
If you want to use your own node to join the local test network, you need to start at least two verification nodes to start the local test network, and then access your own account.
First, start the first node:
```bash
./target/release/node-predict --dev --validator --base-path /tmp/node01 --name node01 --rpc-port 2343 --ws-port 3454 --port 1232 
```
Then we get the local node identity of the first node, such as 12d3kooweyoppncux8yx66ov9fjnrixwccxwdua2kj6vnc6idep
Then start the second node, access the port and local node identification of the first node
```bash
./target/release/node-predict --dev --validator --base-path /tmp/node02 --name node02 --rpc-port 2344 --ws-port 3455 --port 1233 --bootnodes /ip4/127.0.0.1/tcp/1232/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp
```
After starting at least two verification nodes, [ensure that a corresponding number of key pairs have been generated](https://substrate.dev/docs/en/tutorials/start-a-private-network/keygen)，Then enter the key of the node into keysore. First, the mnemonics of node 1 and public key information
```bash
curl --location --request POST 'http://localhost:2343' \
--header 'Content-Type: application/json' \
--data-raw '{
  "jsonrpc":"2.0",
  "method":"author_insertKey",
  "params": [
    "aura",
    "<mnemonic phrase>",
    "<public key>"
  ]
  "id":1,
}' -s | jq .
```
Then input the grandpa mnemonics of node 1 and the public key information
```bash
curl --location --request POST 'http://localhost:2343' \
--header 'Content-Type: application/json' \
--data-raw '{
  "jsonrpc":"2.0",
  "method":"author_insertKey",
  "params": [
    "gran",
    "<mnemonic phrase>",
    "<public key>"
  ]
  "id":1,
}' -s | jq .
```
After inputting the key of node 1, run the same steps of inputting the key of node 2 again. After completing these steps, the multi node local test network has been started, and then you can join your own node.
```bash
./node-predict --dev --base-path /tmp/node03 --name node03 --rpc-port 2345 --ws-port 3456 --port 1234 --ws-external --rpc-external  --bootnodes /ip4/127.0.0.1/tcp/1232/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp
```
After starting the single node or test node, add JSON types in the developer settings of polkadotjs 
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
  "ChainId": "u32",
  "CrossInfo": {
    "to": "Text",
    "currencyId": "CurrencyId",
    "number": "Balance",
    "chainId": "ChainId"
  },
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

## 5.Pallet's Documentation
- [xpmrl_autonomy](https://rustdoc.x-predict.com/xpmrl_autonomy)
- [xpmrl_couple](https://rustdoc.x-predict.com/xpmrl_couple)
- [xpmrl_proposals](https://rustdoc.x-predict.com/xpmrl_proposals)
- [xpmrl_tokens](https://rustdoc.x-predict.com/xpmrl_tokens)
- [xpmrl_traits](https://rustdoc.x-predict.com/xpmrl_traits)
- [xpmrl_utils](https://rustdoc.x-predict.com/xpmrl_utils)

## 6.X predict market's Documentation
[User Tutorials](https://docs.google.com/document/d/1lW0NzIf45IKt3Z1Ggx-ZsKVRKI_4sSpmcDEuloACSig/edit?usp=sharing)
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

