# Fixed Odds Market


## Tests

### Place Bet
- [ ] It properly accepts bets
- [ ] The receiver will be the beneficiary when defined
- [ ] It cant place bet if market isn't active
- [ ] It can only place bets up until 5 minutes before market start timestamp
- [ ] It cant place bet without sending funds in the market denom
- [ ] It cant place bet with amount higher than the max allowed bet
- [ ] It cant place bet if market doesnt have enough funds to pay the bet

### Claim winnings
- [ ] It properly claims winnings
- [ ] It can claim on behalf of the receiver when defined
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
- [ ] It cant score the market if sender isnt the admin
- [ ] It cant score the market if it is no longer active
- [ ] It can only score the market after 30 minutes of its start timestamp

### Cancel market
- [X] It properly cancels the market
- [X] It cant cancel the market if sender isnt the admin
- [X] It cant cancel the market if it is no longer active
