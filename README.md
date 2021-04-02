# X Predict Market

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

