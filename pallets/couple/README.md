# XPMRL Couple Pallet

About the reading and writing of settlement assets, option assets and liquid assets.

##Interface
###Dispatchable Functions
* `add_liquidity` - The user can add the liquidity of a user-defined number of assets in the proposal of formal forecast status
* `remove_liquidity` - At the end of the proposal, the user can remove liquidity to obtain the corresponding settlement assets and option assets
* `buy` - Users can choose their favorite options to vote, and determine the number of option assets returned according to the purchase quantity.
* `sell` - If you want to cancel a vote in the formal prediction stage, you can sell it 
* `retrieval` - After the proposal is finished, the user can call to itnitiate liquidation, and the system returns the user's corresponding settlement assets according to the proposal result and the user's corresponding number of option assets.
* `set_result` - Sets the option final result of the proposal, and the status changes to end.
