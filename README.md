# X Predict Market
## 1.Overview
X Predict Market 是一个去中心化的预测市场。具有自己独立的资产管理和清算系统，支持多币种跨链交易，支持用户在链上创建提案，进行投票，提供或移除流动性等操作，支持用户进行跨链资产转账等操作。

## 2.Build
首先在运行前要确保已经[安装rust和wasm编译环境](https://substrate.dev/docs/en/knowledgebase/getting-started/)，并且[生成自己的密钥](https://substrate.dev/docs/en/tutorials/start-a-private-network/keygen)
编译wasm和本地环境
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
如果你想用自己的节点加入本地测试网，则初始至少需要启动两个验证节点以启动本地测试网，然后再将自己的账户接入。
首先，启动第一个节点：
```bash
./target/release/node-predict --dev --validator --base-path /tmp/node01 --name node01 --rpc-port 2343 --ws-port 3454 --port 1232 
```
然后得到第一个节点的 Local node identity，例如 12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp
接着启动第二个节点，接入第一个节点的端口和本地节点标识：
```bash
./target/release/node-predict --dev --validator --base-path /tmp/node02 --name node02 --rpc-port 2344 --ws-port 3455 --port 1233 --bootnodes /ip4/127.0.0.1/tcp/1232/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp
```
在启动了至少两个验证节点后，[确保已生成对应数量的密钥对](https://substrate.dev/docs/en/tutorials/start-a-private-network/keygen)，然后向Keystore中输入节点的密钥，首先是节点1的aura助记词，公钥信息
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
之后输入节点1的grandpa助记词，公钥信息
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
在输入完节点1的密钥后，再跑一遍相同的输入节点2的密钥步骤，完成这些步骤后，多节点本地测试网就已经启动完成了，然后就可以加入自己的节点了。
```bash
./node-predict --dev --base-path /tmp/node03 --name node03 --rpc-port 2345 --ws-port 3456 --port 1234 --ws-external --rpc-external  --bootnodes /ip4/127.0.0.1/tcp/1232/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp
```
在启动单节点或者测试节点后，在polkajs界面的开发者设置中，添加json types：
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
然后就可以在链上发起交易了
## 4.Main Processes
### Mint New Asset
在polkajs页面中，用户可以进行新资产的创建（需要启动predict-dev用到sudo权限），然后给自己的账户分发对应数量的资产，同时需要注意应给用户一定数目的本地资产（发起交易使用），以便后面提案创建或投票的使用。
### Create Proposal
用户可以在预测市场中创建属于自己的提案，可以输入最近的热点事件标题，投票选项，关闭时间，提案类别，提案结算资产种类，提案详情，并且自定义手续费，和初始提供的流动性。在提案提交后，会进入审核投票期，当提案审核通过后，链上其他用户便可进行投票和添加移除流动性等操作。
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
在提案发起成功后，提案会进入原始预测阶段，等待审核，初期为了方便管理，我们由统一的sudo管理员审核，在后期我们将交由社区进行投票审核管理，审核通过后，提案将进入正式预测阶段，用户可以进行投票，添加流动性。
### Buy and Sell
用户可以在不同的提案，根据资产类别的不同，选项的不同，按照自己的喜好进行投票，并且可以在投票期间撤销已经投出的票，还可以进行添加流动性的操作。
### Set Result
提案投票时间截止后，提案将进入等待结果阶段，此时提案结果将由不同的质押节点进行上传，最后结果由多数投票决定，如果发现作恶节点上传错误结果，用户可以进行举报，发起新一轮的投票。在确定仲裁后，作恶质押节点的资产将被惩罚给举报用户和投票用户。前期为了方便管理，设置结果将由sudo管理员进行，后期将会交由社区进行投票治理。
### Retrieval
提案结果公布后，参与提案的投票者和流动性提供者，会获得相应的结算资产和选项资产，根据投票结果，投票者可进行选项资产兑换为结算资产的清算，流动性提供者可进行移除流动性和选项资产兑换为结算资产的清算，以获得各自最终的结算资产。为了鼓励用户发起提案与创建新流动性，提案收取的总手续费会设置一部分作为提案发起者和流动性提供者的奖励。
## 5.Pallet
### Couple:
与结算资产，选项资产，流动性资产读写有关的操作。 
#### 添加提案流动性和币对
用户可以在正式预测状态的提案中添加自定义数量的资产的流动性，通过调用`add_liquidity(origin: OriginFor<T>,proposal_id: ProposalIdOf<T>,number: BalanceOf<T>)`
#### 移除流动性
在提案结束时，用户可以移除流动性以获得对应结算资产和选项资产，通过调用`remove_liquidity(origin: OriginFor<T>,proposal_id: ProposalIdOf<T>,number: BalanceOf<T>)`
#### 购买
用户可以选择自己喜欢的选项进行投票，根据购买数量决定返回的选项资产数量，通过调用`buy(origin: OriginFor<T>,proposal_id: ProposalIdOf<T>,optional_currency_id: CurrencyIdOf<T>,number: BalanceOf<T>)`
#### 出售
如果在正式预测阶段，用户想撤销已经投了的票，可以进行出售操作，通过调用`sell(origin: OriginFor<T>,proposal_id: ProposalIdOf<T>,optional_currency_id: CurrencyIdOf<T>,number: BalanceOf<T>)`进行对应数量的出售。
#### 清算
提案结束后，用户调用`retrieval(origin: OriginFor<T>,proposal_id: ProposalIdOf<T>,optional_currency_id: CurrencyIdOf<T>,number: BalanceOf<T>)`发起清算，系统根据提案结果和用户对应的选项资产数量返回用户对应的结算资产。
#### 设置提案结果
提案在等待结果状态中，可以通过sudo调用`set_result(origin: OriginFor<T>,proposal_id: ProposalIdOf<T>,currency_id: CurrencyIdOf<T>)`设置提案的选项最终结果，并且状态改为结束。
### Proposals:
与提案发起，存储，状态改变相关的操作。
#### 发起新提案
提案发起者可通过调用`new_proposal(origin: OriginFor<T>,title: Vec<u8>,optional: [Vec<u8>; 2],close_time: MomentOf<T>,category_id: T::CategoryId,currency_id: CurrencyIdOf<T>,number: BalanceOf<T>,earn_fee: u32,detail: Vec<u8>)`发起新的提案，并且设置对应的内容和参数。
#### 设置提案状态
sudo管理员可以通过调用`set_status(origin: OriginFor<T>,proposal_id: T::ProposalId,new_status: Status)`改变提案的状态。
### Tokens:
资产新建与管理模块。
#### 创建新的资产类型
sudo管理员可以通过调用`new_asset(origin: OriginFor<T>,name: Vec<u8>,symbol: Vec<u8>,decimals: u8)`生成新的资产。
#### 增加某账户的指定资产ID的资产
sudo管理员可以通过调用`mint(origin: OriginFor<T>,currency_id: T::CurrencyId,to: T::AccountId,number: BalanceType<T>)`增加某账户的指定资产ID的对应数量资产。
#### 销毁某账户的指定资产ID的资产
sudo管理员可以通过调用`burn(origin: OriginFor<T>,currency_id: T::CurrencyId,number: BalanceType<T>)`销毁某账户的指定资产ID的对应数量资产。
#### 被授权者销毁授权者的指定资产ID的指定数量资产
被授权者通过调用`burn_from(origin: OriginFor<T>,currency_id: T::CurrencyId,from: T::AccountId,number: BalanceType<T>,)`销毁授权者的指定资产ID的对应数量资产。
#### 转账
发起人通过调用`transfer(origin: OriginFor<T>,currency_id: T::CurrencyId,to: T::AccountId,number: BalanceType<T>,)`给某用户转账。
#### 授权转账
发起人通过调用`transfer_from(origin: OriginFor<T>,currency_id: T::CurrencyId,from: T::AccountId,to: T::AccountId,number: BalanceType<T>,)`，对指定账户转账指定资产ID的指定数量的授权者的资产。
#### 授权
发起人通过调用`approve(origin: OriginFor<T>,currency_id: T::CurrencyId,spender: T::AccountId,number: BalanceType<T>,)`对某账户进行指定资产ID的指定数目的资产授权。
### traits:
模块接口约束特征。
### utils:
项目中用到的宏。
# Test Guide
## Integration tests
项目中每一个模块都有自己独立的测试，在满足编译环境的前提下，进行整体测试命令：
```bash
Cargo test
```
或者进行单独pallet测试命令：
```bash
Cargo test -p xpmrl-pallet_name
```
提供的测试用例中有：
用户1创建提案，对提案中的索引，新建资产ID，初始流动性等数据进行校验。提案发起成功后，进行设置提案状态的校验。
新增资产，进行资产ID和数据的校验，对用户1进行新资产的发放和销毁，用户1和用户2之间进行资产的转移和授权转移。
用户1创建提案，提案通过后，添加流动性，用户2购买卖出，提案设置结果，结束后用户1移除流动性，用户1，2清算选项资产。
使用者也可以自行增加测试用例测试。
## Types

```json
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

## **RPC**

```js
const rpc = {
    proposal: {
        getProposalInfo: {
            description: "getProposalInfo",
            params: [{
                name: "versionId",
                type: "VersionId"
            }, {
                name: "proposalId",
                type: "ProposalId"
            }],
            type: "ProposalInfo",
        },
        getPersonalProposalInfo: {
            description: "getPersonalProposalInfo",
            params: [{
                name: "versionId",
                type: "VersionId"
            }, {
                name: "proposalId",
                type: "ProposalId"
            }, {
                name: "accountId",
                type: "AccountId"
            }],
            type: "PersonalProposalInfo",
        }
    }
}
```

