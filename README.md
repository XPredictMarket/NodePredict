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
        "title": "Text"
    }
}
```

## **RPC**

    ```js
    const rpc = {
        proposal: {
            getProposalInfo: {
                description: "haha",
                params: [{
                    name: "version_id",
                    type: "VersionId"
                }, {
                    name: "proposal_id",
                    type: "ProposalId"
                }],
                type: "ProposalInfo",
            }
        }
    }
    ```
