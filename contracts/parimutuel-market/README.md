# Parimutuel Market

## Unit Tests

### Place Bet
- [ ] It properly accepts bets
- [ ] The receiver will be the beneficiary when defined
- [ ] It cant place bet on draw when market isn't drawable
- [ ] It cant place bet if market isn't active
- [ ] It can only place bets up until 5 minutes before market start timestamp
- [ ] It cant place bet without sending funds in the market denom

### Claim winnings
- [ ] It properly claims winnings
- [ ] The receiver will be the beneficiary when defined
- [ ] It will return all bets made if market was cancelled
- [ ] It cant claim winnings while market is active
- [ ] It cant claim winnings twice
- [ ] It cant claim winnings when there is nothing to claim

### Update market
- [ ] It properly updates market
- [ ] It cant update market if sender isnt the admin
- [ ] It cant update market if it is no longer active

### Score market
- [ ] It properly scores the market and collects fees
- [ ] It doesnt collect fees when its set to zero
- [ ] It cant score the market if sender isnt the admin
- [ ] It cant score the market with DRAW if the market isnt drawable
- [ ] It cant score the market if it is no longer active
- [ ] It can only score the market after 30 minutes of its start timestamp
- [ ] It cant score the market if there are no winnings

### Cancel market
- [x] It properly cancels the market
- [x] It cant cancel the market if sender isnt the admin
- [x] It cant cancel the market if it is no longer active
