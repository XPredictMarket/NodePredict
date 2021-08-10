# Proposal workflow

## Explain new features

Our new features are mainly for on-chain proposal state transition governance and result upload governance.

We have also refactored the old code. We have independently transformed the proposal-related information in the old code into a trait. All other proposal-related pallets are derived from this trait, so that all pallets that need to operate on the proposal do not need to rely on the proposal pallet.

## Project directory structure

https://github.com/XPredictMarket/NodePredict/tree/master/pallets

```
- pallets
    + autonomy
    + couple
    - proposals
        + rpc
        + runtime-api
        + src
    + ruler
    + tokens
    + traits
    + utils
```

- [autonomy](https://rustdoc.x-predict.com/xpmrl_autonomy/index.html)

  Ordinary users can stake enough governance token to be tagged as qualified for result uploading. When a result is uploaded, if anyone is not satisfied with the result, he/she can always challenge the result by submitting a new result，which other users can chose whether to second the new report or not. In other words, any new submitted report will also go through the challenge period. Final result will be taken once users satisfied with the current result.

- [couple](https://rustdoc.x-predict.com/xpmrl_couple/index.html)

  This pallet includes functions that allow users to create new proposals, add liquidity and remove liquidity for predictions, buy and sell the binomial voting options, retrieve the settlement token when prediction ends, and mark the result of each prediction and change the prediction status to end.

- [proposals](https://rustdoc.x-predict.com/xpmrl_proposals/index.html)

  This pallet is designed to manage the most basic information of proposals and voting functions. Proposals cannot be created through this pallet, but proposals can be changed from original proposals to formal proposals through this pallet. If the user is interested in this proposal, the user needs to stake a certain amount of governance tokens to vote for this proposal. When the number of votes for approval is greater than the minimum required number of votes and greater than the number of votes against, the voting result becomes effective, and the proposal changes from the original proposal to a formal proposal.

- [ruler](https://rustdoc.x-predict.com/xpmrl_ruler/index.html)

  This pallet saves the addresses of our project parties. These addresses may be granted with special permissions, or they may be the addresses for receiving team reward tokens. For the time being, only the addresses for receiving team rewards will be saved, and the addresses with special permissions will be added later.

- [tokens](https://rustdoc.x-predict.com/xpmrl_tokens/index.html)

  This pallet is designed to manage assets on the chain, and each asset is distinguished by id. Users can perform operations such as minting, burning, and transferring assets.

- [traits](https://rustdoc.x-predict.com/xpmrl_traits/index.html)

  This pallet defines all the `traits` used in the entire project.

- [utils](https://rustdoc.x-predict.com/xpmrl_utils/index.html)

  This pallet defines all the macros and tool functions used in the entire project.

## workflow

> In the following documents, the naming rules for functions are: `<directory name>::<file name>::<function name>`, All valid codes of pallet are in the `src` folder under the corresponding folder

- New proposal

  > github link: https://github.com/XPredictMarket/NodePredict/blob/master/pallets/couple/src/lib.rs

  > rustdoc link: https://rustdoc.x-predict.com/xpmrl_couple/index.html

  When creating a new proposal, users do not need any pre-staking, and any account holding cryptocurrency can create a proposal. To create a proposal, you need to call the function [couple::lib::new_proposal](https://rustdoc.x-predict.com/xpmrl_couple/pallet/struct.Pallet.html#method.new_proposal), and the parameters that need to be passed in are:

  | name          | meaning                                                                                                                                                                                                                                                        |
  | :------------ | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
  | `title`       | The title of the proposal. This has no effect on the entire code logic. It is only for display to the dapp after storage. Generally, it will not be changed after creation.                                                                                    |
  | `optional`    | The text description of the two options in the proposal makes users more clear about the meaning of the two options.                                                                                                                                           |
  | `close_time`  | The proposal will be automatically closed when it does not enter the formal proposal status and exceeds the voting end time. And if the proposal enters the formal proposal status, it will automatically enter the state of waiting for the result.           |
  | `category_id` | The category of the proposal. These data have no impact on the business. It is only used to display to dapp after storage, and will not be changed after creation.                                                                                             |
  | `currency_id` | Proposal settlement currency ID. The assets of this ID can be used to add liquidity, buy, and sell.                                                                                                                                                            |
  | `number`      | The amount of settlement currency deposited in the proposal in advance, this amount of settlement currency will be directly added to the liquidity of the proposal, and the proposal will be returned to the creator with the same amount of liquidity tokens. |
  | `earn_fee`    | The transaction fee rate of the proposal. When the user buys and sells, the fee is deposited into the pool through this rate and can be withdrawn when the proposal ends.                                                                                      |
  | `detail`      | Additional details of the proposal for dapp display.                                                                                                                                                                                                           |

  The proposal after creation is in the [_original proposal state_](https://rustdoc.x-predict.com/xpmrl_traits/enum.ProposalStatus.html#variant.OriginalPrediction). If you want to enter the [_formal proposal state_](https://rustdoc.x-predict.com/xpmrl_traits/enum.ProposalStatus.html#variant.FormalPrediction), you need to vote, but some special proposals cannot be passed. At this time, we can use the administrator account to directly end the corresponding issue.

- Proposal voting

  > github link: https://github.com/XPredictMarket/NodePredict/blob/master/pallets/proposals/src/lib.rs

  > rustdoc link: https://rustdoc.x-predict.com/xpmrl_proposals/index.html

  When voting, at the beginning of the project, there are no restrictions. Everyone can vote. Users use the function [proposals::lib::stake_to](https://rustdoc.x-predict.com/xpmrl_proposals/pallet/struct.Pallet.html#method.stake_to) to vote. Parameter description:

  | name          | meaning                                                                                                                                  |
  | :------------ | :--------------------------------------------------------------------------------------------------------------------------------------- |
  | `proposal_id` | The ID of the proposal. After the new_proposal call is successful, the corresponding event will contain the ID of the created proposal.  |
  | `number`      | The number of votes for or against this proposal.                                                                                        |
  | `opinion`     | The user’s attitude towards this proposal is to support this proposal or oppose it, pass `true` for support, pass `false` for objection. |

  Within the minimum expiration time of [ProposalAutomaticExpirationTime](https://rustdoc.x-predict.com/xpmrl_proposals/pallet/type.ProposalAutomaticExpirationTime.html) we set, all users can vote freely. When the minimum expiration time is exceeded, the `Hooks` function (`on_initialize`) will automatically determine the number of each option. If the number of votes for approval exceeds the minimum amount of approval vote &When the number of approval votes exceed the rejection votes, the initial proposal automatically becomes a formal proposal; if it is unsuccessful, the proposal is automatically closed and enters the end state. At this time, the proposal maker can use the [couple::lib::remove_liquidity](https://rustdoc.x-predict.com/xpmrl_couple/pallet/struct.Pallet.html#method.remove_liquidity) function (the specific function parameters will be described in detail in the document below) to remove the liquidity provided at the time of creation and obtain the deposited settlement tokens.

  When the status of the proposal changes, the tokens staked by the user's vote will also be released, and the user needs to manually call the [proposals::lib::unstake_from](https://rustdoc.x-predict.com/xpmrl_proposals/pallet/struct.Pallet.html#method.unstake_from) function to take out the staked tokens. The parameters are similar to the [proposals::lib::stake_to](https://rustdoc.x-predict.com/xpmrl_proposals/pallet/struct.Pallet.html#method.stake_to) function.

  For some proposals, those who vote for it will get some rewards, which will encourage users to vote for it and increase the data on our chain. The user needs to use the [proposals::lib::withdrawal_reward](https://rustdoc.x-predict.com/xpmrl_proposals/pallet/struct.Pallet.html#method.withdrawal_reward) function to extract the reward. The parameters are similar to the [proposals::lib::stake_to](https://rustdoc.x-predict.com/xpmrl_proposals/pallet/struct.Pallet.html#method.stake_to) function. This reward requires us to use [proposals::lib::deposit_reward](https://rustdoc.x-predict.com/xpmrl_proposals/pallet/struct.Pallet.html#method.deposit_reward) in advance to deposit a certain amount of reward tokens.

- Proposal transaction

  > github link: https://github.com/XPredictMarket/NodePredict/blob/master/pallets/couple/src/lib.rs

  > rustdoc link: https://rustdoc.x-predict.com/xpmrl_couple/index.html

  When the proposal becomes a formal proposal, users can buy the option tokens they support according to their own ideas, or they can sell the option tokens they bought.

  All users holding settlement tokens or option tokens can participate in the transaction of the proposal. Users can add liquidity to the proposal through the function [couple::lib::add_liquidity](https://rustdoc.x-predict.com/xpmrl_couple/pallet/struct.Pallet.html#method.add_liquidity), and use [couple::lib::buy](https://rustdoc.x-predict.com/xpmrl_couple/pallet/struct.Pallet.html#method.buy) to purchase For the corresponding option coins, use [couple::lib::sell](https://rustdoc.x-predict.com/xpmrl_couple/pallet/struct.Pallet.html#method.sell) to sell the option coins you purchased.

  - `couple::lib::add_liquidity`:

    | name          | meaning                                                                                                                                    |
    | :------------ | :----------------------------------------------------------------------------------------------------------------------------------------- |
    | `proposal_id` | Same meaning as above.                                                                                                                     |
    | `number`      | The number of settlement tokens, used to deposit liquidity for the proposal, the proposal will return the same amount of liquidity tokens. |

  - `couple::lib::buy`:

    | name                   | meaning                                                                                                                                                   |
    | :--------------------- | :-------------------------------------------------------------------------------------------------------------------------------------------------------- |
    | `proposal_id`          | Same meaning as above.                                                                                                                                    |
    | `optional_currency_id` | Option token ID corresponding to the option to be purchased.                                                                                              |
    | `number`               | The number of settlement tokens. This is not the number of option tokens. The number of option tokens is calculated from the number of settlement tokens. |

  - `couple::lib::sell`:

    | name                   | meaning                                                                                              |
    | :--------------------- | :--------------------------------------------------------------------------------------------------- |
    | `proposal_id`          | Same meaning as above                                                                                |
    | `optional_currency_id` | Option token ID corresponding to the option to be purchased                                          |
    | `number`               | The user needs to sell the number of option tokens, this is not the number of settlement currencies. |

- End of proposal

  > github link: https://github.com/XPredictMarket/NodePredict/blob/master/pallets/autonomy/src/lib.rs

  > rustdoc link: https://rustdoc.x-predict.com/xpmrl_autonomy/index.html

  When the current time exceeds the end time of the formal proposal, the proposal enters the [_waiting for result state_](https://rustdoc.x-predict.com/xpmrl_traits/enum.ProposalStatus.html#variant.WaitingForResults). This process does not require anyone to participate and is completed in the chain `Hooks` function.

  When the proposal enters the state of waiting for the result, users are only permitted to upload the result if they staked certain amount of governance tokens and tagged as qualified by the official review.

  Any user can complete the staking through the function [autonomy::lib::stake](https://rustdoc.x-predict.com/xpmrl_autonomy/pallet/struct.Pallet.html#method.stake). Only users who staked enough governance tokens can be officially reviewed and tagged as qualified (Use the function [autonomy::lib::tagging](https://rustdoc.x-predict.com/xpmrl_autonomy/pallet/struct.Pallet.html#method.tagging) to tag. If we feel that some users upload malicious results, we will use the function [autonomy::lib::untagging](https://rustdoc.x-predict.com/xpmrl_autonomy/pallet/struct.Pallet.html#method.untagging) to remove tags, and in serious cases, we will use [autonomy::lib::slash](https://rustdoc.x-predict.com/xpmrl_autonomy/pallet/struct.Pallet.html#method.slash) to punish the malicious user), so that the results of the proposal can be uploaded. Users who have staked governance tokens can also withdraw the pledged tokens through [autonomy::lib::unstake](https://rustdoc.x-predict.com/xpmrl_autonomy/pallet/struct.Pallet.html#method.unstake). Withdrawal of the tokens also means giving up the qualification to upload results.

  - `autonomy::lib::stake`

    No parameters

  - `autonomy::lib::unstake`

    No parameters

  - `autonomy::lib::tagging`

    **Can only be called through the sudo module.**

    | name      | meaning                                                          |
    | :-------- | :--------------------------------------------------------------- |
    | `account` | The marked account, the account needs to stake governance tokens |

  - `autonomy::lib::untagging`

    **Can only be called through the sudo module.**

    Cancel the tag, the parameters are similar to tagging

  - `autonomy::lib::slash`

    **Can only be called through the sudo module.**

    the parameters are similar to tagging

  The address that has been reviewed and tagged by the project party can upload the result through unsigned transaction with signed payload. The name of this function is [autonomy::lib::upload_result](https://rustdoc.x-predict.com/xpmrl_autonomy/pallet/struct.Pallet.html#method.upload_result), and the parameters are:

  ```rust
  pub struct Payload<Public, ProposalId, ResultId> {
      /// The id of the proposal that needs to upload the result
      pub proposal_id: ProposalId,
      /// The asset id of the proposal result
      ///
      /// The proposal option is a token, so here only the id of the corresponding token needs to be uploaded
      pub result: ResultId,
      // Public key of the account
      pub public: Public,
  }
  ```

  | name         | meaning                               |
  | :----------- | :------------------------------------ |
  | `payload`    | Instance of `Payload` structures      |
  | `_signature` | Payload signed by account private key |

  If the user is not satisfied with the uploaded result, he can report the user who uploaded the result through the function [autonomy::lib::report](https://rustdoc.x-predict.com/xpmrl_autonomy/pallet/struct.Pallet.html#method.report), and other users can chose whether to support this new report (function [autonomy::lib::seconded_report](https://rustdoc.x-predict.com/xpmrl_autonomy/pallet/struct.Pallet.html#method.seconded_report)) , When the report period ends, the staked governance tokens can be taken out through the function [autonomy::lib::take_out](https://rustdoc.x-predict.com/xpmrl_autonomy/pallet/struct.Pallet.html#method.take_out).

  - `autonomy::lib::report`

    | name          | meaning                                                                                                                               |
    | :------------ | :------------------------------------------------------------------------------------------------------------------------------------ |
    | `proposal_id` | The ID of the proposal.                                                                                                               |
    | `target`      | Reported user                                                                                                                         |
    | `number`      | The tokens staked at the time of reporting will only take effect if it is more than twice the amount pledged by the reported account. |

  - `autonomy::lib::seconded_report`

    | name          | meaning                                                                       |
    | :------------ | :---------------------------------------------------------------------------- |
    | `proposal_id` | Same meaning as above                                                         |
    | `target`      | Same meaning as above                                                         |
    | `number`      | Same meaning as above                                                         |
    | `support`     | The attitude towards this report, `true` means yes, `false` means disapproval |

  - `autonomy::lib::take_out`

    | name          | meaning               |
    | :------------ | :-------------------- |
    | `proposal_id` | Same meaning as above |
    | `target`      | Same meaning as above |

- Proposal End

  > github link: https://github.com/XPredictMarket/NodePredict/blob/master/pallets/autonomy/src/lib.rs

  After the end of the publicity period, the corresponding results will be automatically merged. If there is a report and the report is successful, the result opposite to the upload result will be merged. Functions used in this are same as above step of "End of prediction".
