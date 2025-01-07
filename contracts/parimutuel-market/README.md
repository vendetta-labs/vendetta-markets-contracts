# Parimutuel Market

## Tests

### Place Bet
- [X] It properly accepts bets
- [X] The receiver will be the beneficiary when defined
- [X] It cant place bet on draw when market isn't drawable
- [X] It cant place bet if market isn't active
- [X] It can only place bets up until 5 minutes before market start timestamp
- [X] It cant place bet without sending funds in the market denom

### Claim winnings
- [X] It properly claims winnings
- [X] It can claim on behalf of the receiver when defined
- [X] It will return all bets made if market was cancelled
- [X] It cant claim winnings while market is active
- [X] It cant claim winnings twice
- [X] It cant claim winnings when there is nothing to claim

### Update market
- [X] It properly updates market
- [X] It cant update market if sender isnt the admin
- [X] It cant update market if it is no longer active

### Score market
- [X] It properly scores the market and collects fees
- [X] It doesnt collect fees when its set to zero
- [X] It cant score the market if sender isnt the admin
- [X] It cant score the market with DRAW if the market isnt drawable
- [X] It cant score the market if it is no longer active
- [X] It can only score the market after 30 minutes of its start timestamp
- [X] It cant score the market if there are no winnings
- [X] It cant score the market if there are no winners

### Cancel market
- [X] It properly cancels the market
- [X] It cant cancel the market if sender isnt the admin
- [X] It cant cancel the market if it is no longer active
