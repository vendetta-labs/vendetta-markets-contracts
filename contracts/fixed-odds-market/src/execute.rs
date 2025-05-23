use cosmwasm_std::{
    coin, Addr, BankMsg, CosmosMsg, Decimal, DepsMut, Env, Fraction, MessageInfo, Response,
    StdResult, Uint128,
};

use crate::{
    error::ContractError,
    logic::{calculate_max_bet, calculate_odds},
    msg::UpdateParams,
    state::{
        MarketResult, Status, ADDR_BETS_AWAY, ADDR_BETS_HOME, CLAIMS, CONFIG, MARKET,
        POTENTIAL_PAYOUT_AWAY, POTENTIAL_PAYOUT_HOME, TOTAL_BETS_AWAY, TOTAL_BETS_HOME,
    },
    validation::{
        validate_fee_spread_odds, validate_max_bet_risk_factor, validate_odd,
        validate_seed_liquidity_amplifier,
    },
};

/// Places a bet on the market
///
/// The total bets result, potential payout result are updated and the address bets result
/// that records the average odd and total bet amount per address is updated.
///
/// Then it will recalculate the new odds based on the new bet.
///
/// It will make the following checks:
/// - The market needs to be active
/// - The current block timestamp needs to be at least 5 minutes before the start timestamp
/// - The minimum odds need to be less than the current odds
/// - The bet amount needs to be greater than zero
/// - The bet amount needs to be less than the max allowed bet
pub fn execute_place_bet(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    result: MarketResult,
    min_odds: Decimal,
    receiver: Option<Addr>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut market = MARKET.load(deps.storage)?;

    let addr = match receiver {
        Some(receiver) => deps.api.addr_validate(receiver.as_str())?,
        None => info.sender.clone(),
    };

    if market.status != Status::ACTIVE {
        return Err(ContractError::MarketNotActive {});
    }

    // Bets are accepted up until 5 minutes before the start of the match
    if market.start_timestamp - 5 * 60 < env.block.time.seconds() {
        return Err(ContractError::BetsNotAccepted {});
    }

    let odds: Decimal = match result {
        MarketResult::HOME => market.home_odds,
        MarketResult::AWAY => market.away_odds,
    };

    if odds < min_odds {
        return Err(ContractError::MinimumOddsNotKept {});
    }

    let bet_amount = cw_utils::must_pay(&info, &config.denom);
    if bet_amount.is_err() {
        return Err(ContractError::PaymentError {});
    }
    let bet_amount = bet_amount.unwrap();

    let market_balance = deps
        .querier
        .query_balance(env.contract.address, &config.denom)?
        .amount;

    let potential_market_payout = match result {
        MarketResult::HOME => POTENTIAL_PAYOUT_HOME.load(deps.storage)?,
        MarketResult::AWAY => POTENTIAL_PAYOUT_AWAY.load(deps.storage)?,
    };

    let max_bet = calculate_max_bet(
        &config,
        market_balance - bet_amount,
        Uint128::from(potential_market_payout),
        odds,
    );
    if bet_amount > max_bet {
        return Err(ContractError::MaxBetExceeded {});
    }

    let bet_record = match result {
        MarketResult::HOME => ADDR_BETS_HOME.may_load(deps.storage, addr.clone())?,
        MarketResult::AWAY => ADDR_BETS_AWAY.may_load(deps.storage, addr.clone())?,
    };

    let mut total_bet_amount = bet_amount;
    let payout = bet_amount.multiply_ratio(odds.numerator(), odds.denominator());
    let mut total_payout = payout;
    if bet_record.is_some() {
        let (previous_total_bet_amount, previous_total_payout) = bet_record.unwrap();
        total_bet_amount += Uint128::from(previous_total_bet_amount);
        total_payout += Uint128::from(previous_total_payout);
    }

    match result {
        MarketResult::HOME => {
            TOTAL_BETS_HOME.update(deps.storage, |total| -> StdResult<_> {
                Ok((Uint128::from(total) + bet_amount).into())
            })?;
            ADDR_BETS_HOME.save(
                deps.storage,
                addr.clone(),
                &(total_bet_amount.into(), total_payout.into()),
            )?;
            POTENTIAL_PAYOUT_HOME.update(deps.storage, |total| -> StdResult<_> {
                Ok((Uint128::from(total) + payout).into())
            })?;
        }
        MarketResult::AWAY => {
            TOTAL_BETS_AWAY.update(deps.storage, |total| -> StdResult<_> {
                Ok((Uint128::from(total) + bet_amount).into())
            })?;
            ADDR_BETS_AWAY.save(
                deps.storage,
                addr.clone(),
                &(total_bet_amount.into(), total_payout.into()),
            )?;
            POTENTIAL_PAYOUT_AWAY.update(deps.storage, |total| -> StdResult<_> {
                Ok((Uint128::from(total) + payout).into())
            })?;
        }
    };

    let home_total_bets = TOTAL_BETS_HOME.load(deps.storage)?;
    let away_total_bets = TOTAL_BETS_AWAY.load(deps.storage)?;

    let (new_home_odds, new_away_odds) = calculate_odds(
        &config,
        market_balance,
        Uint128::from(home_total_bets),
        Uint128::from(away_total_bets),
    );
    market.home_odds = new_home_odds;
    market.away_odds = new_away_odds;
    MARKET.save(deps.storage, &market)?;

    let potential_payout_home = POTENTIAL_PAYOUT_HOME.load(deps.storage)?;
    let potential_payout_away = POTENTIAL_PAYOUT_AWAY.load(deps.storage)?;
    let new_home_max_bet = calculate_max_bet(
        &config,
        market_balance,
        Uint128::from(potential_payout_home),
        new_home_odds,
    );
    let new_away_max_bet = calculate_max_bet(
        &config,
        market_balance,
        Uint128::from(potential_payout_away),
        new_away_odds,
    );

    Ok(Response::new()
        .add_attribute("protocol", "vendetta-markets")
        .add_attribute("market_type", "fixed-odds")
        .add_attribute("action", "place_bet")
        .add_attribute("sender", info.sender)
        .add_attribute("receiver", addr)
        .add_attribute("result", result.to_string())
        .add_attribute("bet_amount", bet_amount.to_string())
        .add_attribute("odds", odds.to_string())
        .add_attribute("potential_payout", payout.to_string())
        .add_attribute("new_home_odds", market.home_odds.to_string())
        .add_attribute("new_away_odds", market.away_odds.to_string())
        .add_attribute("new_home_max_bet", new_home_max_bet.to_string())
        .add_attribute("new_away_max_bet", new_away_max_bet.to_string())
        .add_attribute("total_bets_home", home_total_bets.to_string())
        .add_attribute("total_bets_away", away_total_bets.to_string())
        .add_attribute("potential_payout_home", potential_payout_home.to_string())
        .add_attribute("potential_payout_away", potential_payout_away.to_string()))
}

/// Claims winnings for the sender or the receiver if defined or returns all bets
/// made if the market was cancelled, it will calculate the winnings based on the
/// average odds and the total bet amount for the address.
///
/// It will make the following checks:
/// - The market needs to be closed
/// - The address can't have claimed already
/// - The address needs to have some amount to claim
pub fn execute_claim_winnings(
    deps: DepsMut,
    info: MessageInfo,
    receiver: Option<Addr>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let market = MARKET.load(deps.storage)?;

    let addr = match receiver {
        Some(receiver) => deps.api.addr_validate(receiver.as_str())?,
        None => info.sender.clone(),
    };

    if market.status == Status::ACTIVE {
        return Err(ContractError::MarketNotClosed {});
    }

    if CLAIMS.has(deps.storage, addr.clone()) {
        return Err(ContractError::ClaimAlreadyMade {});
    }

    let home_bet = ADDR_BETS_HOME.may_load(deps.storage, addr.clone())?;
    let away_bet = ADDR_BETS_AWAY.may_load(deps.storage, addr.clone())?;

    let mut payout = 0;
    if market.status == Status::CANCELLED {
        payout = home_bet.unwrap_or((0, 0)).0 + away_bet.unwrap_or((0, 0)).0;
    } else {
        match market.result {
            Some(MarketResult::HOME) => {
                if let Some((_, total_payout)) = home_bet {
                    payout = total_payout;
                }
            }
            Some(MarketResult::AWAY) => {
                if let Some((_, total_payout)) = away_bet {
                    payout = total_payout;
                }
            }
            None => (),
        }
    }

    let mut messages: Vec<CosmosMsg> = vec![];
    if payout > 0 {
        messages.push(
            BankMsg::Send {
                to_address: addr.to_string(),
                amount: vec![coin(payout, config.denom)],
            }
            .into(),
        );
    } else {
        return Err(ContractError::NoWinnings {});
    }

    CLAIMS.save(deps.storage, addr.clone(), &true)?;

    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("protocol", "vendetta-markets")
        .add_attribute("market_type", "fixed-odds")
        .add_attribute("action", "claim_winnings")
        .add_attribute("sender", info.sender)
        .add_attribute("receiver", addr)
        .add_attribute("payout", payout.to_string()))
}

/// Updates the market with the new params, it will recalculate
/// the new odds based on the new params.
///
/// It will make the following checks:
/// - The sender needs to be the admin
/// - The market needs to be active
pub fn execute_update(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    params: UpdateParams,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    let mut market = MARKET.load(deps.storage)?;

    if info.sender != config.admin_addr {
        return Err(ContractError::Unauthorized {});
    }

    if market.status != Status::ACTIVE {
        return Err(ContractError::MarketNotActive {});
    }

    let mut admin_addr_update = String::default();
    if let Some(admin_addr) = params.admin_addr {
        config.admin_addr = admin_addr.clone();
        admin_addr_update = admin_addr.to_string();
    }

    let mut treasury_addr_update = String::default();
    if let Some(treasury_addr) = params.treasury_addr {
        config.treasury_addr = treasury_addr.clone();
        treasury_addr_update = treasury_addr.to_string();
    }

    let mut fee_spread_odds_update = String::default();
    if let Some(fee_spread_odds) = params.fee_spread_odds {
        validate_fee_spread_odds(fee_spread_odds)?;

        config.fee_spread_odds = fee_spread_odds;
        fee_spread_odds_update = fee_spread_odds.to_string();
    }

    let mut max_bet_risk_factor_update = String::default();
    if let Some(max_bet_risk_factor) = params.max_bet_risk_factor {
        validate_max_bet_risk_factor(max_bet_risk_factor)?;

        config.max_bet_risk_factor = max_bet_risk_factor;
        max_bet_risk_factor_update = max_bet_risk_factor.to_string();
    }

    let mut seed_liquidity_amplifier_update = String::default();
    if let Some(seed_liquidity_amplifier) = params.seed_liquidity_amplifier {
        validate_seed_liquidity_amplifier(seed_liquidity_amplifier)?;

        config.seed_liquidity_amplifier = seed_liquidity_amplifier;
        seed_liquidity_amplifier_update = seed_liquidity_amplifier.to_string();
    }

    let mut initial_odds_home_update = String::default();
    let mut initial_odds_away_update = String::default();
    let initial_odds_home = match params.initial_odds_home {
        Some(initial_odds_home) => initial_odds_home,
        None => Decimal::zero(),
    };
    let initial_odds_away = match params.initial_odds_away {
        Some(initial_odds_away) => initial_odds_away,
        None => Decimal::zero(),
    };
    if initial_odds_home != Decimal::zero() || initial_odds_away != Decimal::zero() {
        if initial_odds_home == Decimal::zero() || initial_odds_away == Decimal::zero() {
            return Err(ContractError::InvalidOddsCombination);
        }
        validate_odd(initial_odds_home)?;
        validate_odd(initial_odds_away)?;

        config.initial_odds_home = initial_odds_home;
        config.initial_odds_away = initial_odds_away;
        initial_odds_home_update = initial_odds_home.to_string();
        initial_odds_away_update = initial_odds_away.to_string();
    }

    let mut start_timestamp_update = String::default();
    if let Some(start_timestamp) = params.start_timestamp {
        market.start_timestamp = start_timestamp;
        start_timestamp_update = start_timestamp.to_string();
    }

    CONFIG.save(deps.storage, &config)?;

    let market_balance = deps
        .querier
        .query_balance(&env.contract.address, &config.denom)?
        .amount;
    let home_total_bets = TOTAL_BETS_HOME.load(deps.storage)?;
    let away_total_bets = TOTAL_BETS_AWAY.load(deps.storage)?;

    let (new_home_odds, new_away_odds) = calculate_odds(
        &config,
        market_balance,
        Uint128::from(home_total_bets),
        Uint128::from(away_total_bets),
    );
    market.home_odds = new_home_odds;
    market.away_odds = new_away_odds;
    MARKET.save(deps.storage, &market)?;

    let potential_payout_home = POTENTIAL_PAYOUT_HOME.load(deps.storage)?;
    let potential_payout_away = POTENTIAL_PAYOUT_AWAY.load(deps.storage)?;
    let new_home_max_bet = calculate_max_bet(
        &config,
        market_balance,
        Uint128::from(potential_payout_home),
        new_home_odds,
    );
    let new_away_max_bet = calculate_max_bet(
        &config,
        market_balance,
        Uint128::from(potential_payout_away),
        new_away_odds,
    );

    Ok(Response::new()
        .add_attribute("protocol", "vendetta-markets")
        .add_attribute("market_type", "fixed-odds")
        .add_attribute("action", "update_market")
        .add_attribute("sender", info.sender)
        .add_attribute("admin_addr", admin_addr_update)
        .add_attribute("treasury_addr", treasury_addr_update)
        .add_attribute("fee_spread_odds", fee_spread_odds_update)
        .add_attribute("max_bet_risk_factor", max_bet_risk_factor_update)
        .add_attribute("seed_liquidity_amplifier", seed_liquidity_amplifier_update)
        .add_attribute("initial_odds_home", initial_odds_home_update)
        .add_attribute("initial_odds_away", initial_odds_away_update)
        .add_attribute("start_timestamp", start_timestamp_update)
        .add_attribute("home_odds", market.home_odds.to_string())
        .add_attribute("away_odds", market.away_odds.to_string())
        .add_attribute("home_max_bet", new_home_max_bet.to_string())
        .add_attribute("away_max_bet", new_away_max_bet.to_string())
        .add_attribute("total_bets_home", home_total_bets.to_string())
        .add_attribute("total_bets_away", away_total_bets.to_string())
        .add_attribute("potential_payout_home", potential_payout_home.to_string())
        .add_attribute("potential_payout_away", potential_payout_away.to_string()))
}

/// Scores the market and collects the outstanding balance to the treasury, the
/// outstanding balance is calculated by deducing the total payout matching
/// the market result.
///
/// It will make the following checks:
/// - The sender needs to be the admin
/// - The market needs to be active
/// - The current block timestamp needs to be at least 30 minutes after the start timestamp
pub fn execute_score(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    result: MarketResult,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let market = MARKET.load(deps.storage)?;

    if info.sender != config.admin_addr {
        return Err(ContractError::Unauthorized {});
    }

    if market.status != Status::ACTIVE {
        return Err(ContractError::MarketNotActive {});
    }

    // Market can only be scored after 30 minutes of its start timestamp
    if env.block.time.seconds() < market.start_timestamp + 30 * 60 {
        return Err(ContractError::MarketNotScoreable {});
    }

    let mut market = market;
    market.status = Status::CLOSED;
    market.result = Some(result.clone());
    MARKET.save(deps.storage, &market)?;

    let market_balance = deps
        .querier
        .query_balance(&env.contract.address, &config.denom)?
        .amount;
    let market_payout = match result {
        MarketResult::HOME => POTENTIAL_PAYOUT_HOME.load(deps.storage)?,
        MarketResult::AWAY => POTENTIAL_PAYOUT_AWAY.load(deps.storage)?,
    };

    let market_outstanding_balance = market_balance - Uint128::from(market_payout);

    let mut messages: Vec<CosmosMsg> = vec![];
    let mut market_profit: Uint128 = Uint128::zero();
    let mut market_loss: Uint128 = Uint128::zero();
    if market_outstanding_balance > Uint128::zero() {
        if market_outstanding_balance > config.seed_liquidity {
            market_profit = market_outstanding_balance - config.seed_liquidity;
            messages.push(
                BankMsg::Send {
                    to_address: config.treasury_addr.to_string(),
                    amount: vec![coin(market_profit.into(), &config.denom)],
                }
                .into(),
            );
            messages.push(
                BankMsg::Send {
                    to_address: config.admin_addr.to_string(),
                    amount: vec![coin(config.seed_liquidity.into(), &config.denom)],
                }
                .into(),
            );
        } else {
            market_loss = config.seed_liquidity - market_outstanding_balance;
            messages.push(
                BankMsg::Send {
                    to_address: config.admin_addr.to_string(),
                    amount: vec![coin(market_outstanding_balance.into(), &config.denom)],
                }
                .into(),
            );
        }
    } else {
        market_loss = config.seed_liquidity;
    }

    let potential_payout_home = POTENTIAL_PAYOUT_HOME.load(deps.storage)?;
    let potential_payout_away = POTENTIAL_PAYOUT_AWAY.load(deps.storage)?;
    let home_max_bet = calculate_max_bet(
        &config,
        market_balance,
        Uint128::from(potential_payout_home),
        market.home_odds,
    );
    let away_max_bet = calculate_max_bet(
        &config,
        market_balance,
        Uint128::from(potential_payout_away),
        market.away_odds,
    );

    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("protocol", "vendetta-markets")
        .add_attribute("market_type", "fixed-odds")
        .add_attribute("action", "score_market")
        .add_attribute("sender", info.sender)
        .add_attribute("status", Status::CLOSED.to_string())
        .add_attribute("result", result.to_string())
        .add_attribute("market_outstanding_balance", market_outstanding_balance)
        .add_attribute("market_profit", market_profit)
        .add_attribute("market_loss", market_loss)
        .add_attribute("home_odds", market.home_odds.to_string())
        .add_attribute("away_odds", market.away_odds.to_string())
        .add_attribute("home_max_bet", home_max_bet.to_string())
        .add_attribute("away_max_bet", away_max_bet.to_string())
        .add_attribute(
            "total_bets_home",
            TOTAL_BETS_HOME.load(deps.storage)?.to_string(),
        )
        .add_attribute(
            "total_bets_away",
            TOTAL_BETS_AWAY.load(deps.storage)?.to_string(),
        )
        .add_attribute(
            "potential_payout_home",
            POTENTIAL_PAYOUT_HOME.load(deps.storage)?.to_string(),
        )
        .add_attribute(
            "potential_payout_away",
            POTENTIAL_PAYOUT_AWAY.load(deps.storage)?.to_string(),
        ))
}

/// Cancels the market
///
/// It will make the following checks:
/// - The sender needs to be the admin
/// - The market needs to be active
pub fn execute_cancel(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let market = MARKET.load(deps.storage)?;

    if info.sender != config.admin_addr {
        return Err(ContractError::Unauthorized {});
    }

    if market.status != Status::ACTIVE {
        return Err(ContractError::MarketNotActive {});
    }

    MARKET.update(deps.storage, |mut market| -> Result<_, ContractError> {
        market.status = Status::CANCELLED;
        Ok(market)
    })?;

    let market_balance = deps
        .querier
        .query_balance(&env.contract.address, &config.denom)?
        .amount;
    let total_bets_home = TOTAL_BETS_HOME.load(deps.storage)?;
    let total_bets_away = TOTAL_BETS_AWAY.load(deps.storage)?;

    let market_outstanding_balance =
        market_balance - Uint128::from(total_bets_home) - Uint128::from(total_bets_away);

    let mut messages: Vec<CosmosMsg> = vec![];
    let mut market_profit: Uint128 = Uint128::zero();
    let mut market_loss: Uint128 = Uint128::zero();
    if market_outstanding_balance > Uint128::zero() {
        if market_outstanding_balance > config.seed_liquidity {
            market_profit = market_outstanding_balance - config.seed_liquidity;
            messages.push(
                BankMsg::Send {
                    to_address: config.treasury_addr.to_string(),
                    amount: vec![coin(market_profit.into(), &config.denom)],
                }
                .into(),
            );
            messages.push(
                BankMsg::Send {
                    to_address: config.admin_addr.to_string(),
                    amount: vec![coin(config.seed_liquidity.into(), &config.denom)],
                }
                .into(),
            );
        } else {
            market_loss = config.seed_liquidity - market_outstanding_balance;
            messages.push(
                BankMsg::Send {
                    to_address: config.admin_addr.to_string(),
                    amount: vec![coin(market_outstanding_balance.into(), &config.denom)],
                }
                .into(),
            );
        }
    } else {
        market_loss = config.seed_liquidity;
    }

    let potential_payout_home = POTENTIAL_PAYOUT_HOME.load(deps.storage)?;
    let potential_payout_away = POTENTIAL_PAYOUT_AWAY.load(deps.storage)?;
    let home_max_bet = calculate_max_bet(
        &config,
        market_balance,
        Uint128::from(potential_payout_home),
        market.home_odds,
    );
    let away_max_bet = calculate_max_bet(
        &config,
        market_balance,
        Uint128::from(potential_payout_away),
        market.away_odds,
    );

    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("protocol", "vendetta-markets")
        .add_attribute("market_type", "fixed-odds")
        .add_attribute("action", "cancel_market")
        .add_attribute("sender", info.sender)
        .add_attribute("status", Status::CANCELLED.to_string())
        .add_attribute("market_outstanding_balance", market_outstanding_balance)
        .add_attribute("market_profit", market_profit)
        .add_attribute("market_loss", market_loss)
        .add_attribute("home_odds", market.home_odds.to_string())
        .add_attribute("away_odds", market.away_odds.to_string())
        .add_attribute("home_max_bet", home_max_bet.to_string())
        .add_attribute("away_max_bet", away_max_bet.to_string())
        .add_attribute(
            "total_bets_home",
            TOTAL_BETS_HOME.load(deps.storage)?.to_string(),
        )
        .add_attribute(
            "total_bets_away",
            TOTAL_BETS_AWAY.load(deps.storage)?.to_string(),
        )
        .add_attribute(
            "potential_payout_home",
            POTENTIAL_PAYOUT_HOME.load(deps.storage)?.to_string(),
        )
        .add_attribute(
            "potential_payout_away",
            POTENTIAL_PAYOUT_AWAY.load(deps.storage)?.to_string(),
        ))
}
