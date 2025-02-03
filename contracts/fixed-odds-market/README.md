# Fixed Odds Market

## Tests

### Create Market
- [X] It properly creates a market
- [X] It cant create a market if no seed liquidity is provided
- [X] It cant create a market with invalid fee spread odds
- [X] It cant create a market with invalid max bet risk factor
- [X] It cant create a market with invalid seed liquidity amplifier
- [X] It cant create a market with invalid initial odds

### Place Bet
- [X] It properly accepts bets
- [X] It properly averages bets when there are multiple bets from the same address
- [X] The receiver will be the beneficiary when defined
- [X] It cant place bet if market isn't active
- [X] It can only place bets up until 5 minutes before market start timestamp
- [X] It cant place bet without sending funds in the market denom
- [X] It cant place bet if the min odds requirement is not met
- [X] It cant place bet with amount higher than the max allowed bet

### Claim winnings
- [X] It properly claims winnings
- [X] It can claim on behalf of the receiver when defined
- [X] It will return all bets made if market was cancelled
- [X] It cant claim winnings while market is active
- [X] It cant claim winnings twice
- [X] It cant claim winnings when there is nothing to claim

### Update market
- [X] It properly updates market admin addr
- [X] It properly updates market treasury addr
- [X] It properly updates market start timestamp
- [X] It properly updates market fee spread odds
- [X] It properly updates market max bet risk factor
- [X] It properly updates market seed liquidity amplifier
- [X] It properly updates market initial odds
- [X] It cant update market if sender isnt the admin
- [X] It cant update market if it is no longer active
- [X] It cant update market with invalid fee spread odds
- [X] It cant update market with invalid max bet risk factor
- [X] It cant update market with invalid seed liquidity amplifier
- [X] It cant update market with invalid initial odds
- [X] It cant update market with only one initial odd

### Score market
- [X] It properly scores the market and collects fees
- [X] It cant score the market if sender isnt the admin
- [X] It cant score the market if it is no longer active
- [X] It can only score the market after 30 minutes of its start timestamp

### Cancel market
- [X] It properly cancels the market
- [X] It cant cancel the market if sender isnt the admin
- [X] It cant cancel the market if it is no longer active
