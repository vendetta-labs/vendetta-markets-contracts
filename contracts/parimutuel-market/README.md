# Parimutuel Market

## Tests

### Place Bet
- [x] It properly accepts bets
- [x] The receiver will be the beneficiary when defined
- [x] It cant place bet on draw when market isn't drawable
- [x] It cant place bet if market isn't active
- [x] It can only place bets up until 5 minutes before market start timestamp
- [x] It cant place bet without sending funds in the market denom

### Claim winnings
- [x] It properly claims winnings
- [x] It can claim on behalf of the receiver when defined
- [x] It will return all bets made if market was cancelled
- [x] It cant claim winnings while market is active
- [x] It cant claim winnings twice
- [x] It cant claim winnings when there is nothing to claim

### Update market
- [x] It properly updates market
- [x] It cant update market if sender isnt the admin
- [x] It cant update market if it is no longer active

### Score market
- [x] It properly scores the market and collects fees
- [x] It doesnt collect fees when its set to zero
- [x] It cant score the market if sender isnt the admin
- [x] It cant score the market with DRAW if the market isnt drawable
- [x] It cant score the market if it is no longer active
- [x] It can only score the market after 30 minutes of its start timestamp
- [x] It cant score the market if there are no winnings
- [x] It cant score the market if there are no winners

### Cancel market
- [x] It properly cancels the market
- [x] It cant cancel the market if sender isnt the admin
- [x] It cant cancel the market if it is no longer active
