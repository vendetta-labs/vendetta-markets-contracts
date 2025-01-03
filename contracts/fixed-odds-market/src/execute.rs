use cosmwasm_std::{
    coin, Addr, BankMsg, CosmosMsg, Decimal, DepsMut, Env, Fraction, MessageInfo, Response,
    StdResult, Uint128,
};

use crate::{
    calculate_odds,
    state::{
        MarketResult, Status, AWAY_BETS, AWAY_TOTAL_BETS, AWAY_TOTAL_PAYOUT, CLAIMS, CONFIG,
        HOME_BETS, HOME_TOTAL_BETS, HOME_TOTAL_PAYOUT, MARKET,
    },
    ContractError,
};

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

    let payout = bet_amount.multiply_ratio(odds.numerator(), odds.denominator());

    let market_balance = deps
        .querier
        .query_balance(env.contract.address, &config.denom)?
        .amount;
    let potential_market_payout = match result {
        MarketResult::HOME => HOME_TOTAL_PAYOUT.load(deps.storage)?,
        MarketResult::AWAY => AWAY_TOTAL_PAYOUT.load(deps.storage)?,
    };
    let available_market_balance = market_balance - potential_market_payout;
    let max_bet = Uint128::from(1_000_000_u128);
    if bet_amount > max_bet {
        return Err(ContractError::MaxBetExceeded {});
    }

    let previous_bet = match result {
        MarketResult::HOME => HOME_BETS.may_load(deps.storage, addr.clone())?,
        MarketResult::AWAY => AWAY_BETS.may_load(deps.storage, addr.clone())?,
    };

    let mut average_odds = odds;
    let mut total_bet_amount = bet_amount;
    if previous_bet.is_some() {
        let (previous_odds, previous_bet_amount) = previous_bet.unwrap();
        let previous_payout = previous_bet_amount
            .multiply_ratio(previous_odds.numerator(), previous_odds.denominator());
        let total_payout = payout + previous_payout;
        total_bet_amount = previous_bet_amount + bet_amount;
        average_odds = odds;
    }

    match result {
        MarketResult::HOME => {
            HOME_BETS.save(
                deps.storage,
                addr.clone(),
                &(average_odds.into(), total_bet_amount),
            )?;
            HOME_TOTAL_PAYOUT
                .update(deps.storage, |total| -> StdResult<_> { Ok(total + payout) })?;
            HOME_TOTAL_BETS.update(deps.storage, |total| -> StdResult<_> {
                Ok(total + bet_amount)
            })?;
        }
        MarketResult::AWAY => {
            AWAY_BETS.save(
                deps.storage,
                addr.clone(),
                &(average_odds.into(), total_bet_amount),
            )?;
            AWAY_TOTAL_PAYOUT
                .update(deps.storage, |total| -> StdResult<_> { Ok(total + payout) })?;
            AWAY_TOTAL_BETS.update(deps.storage, |total| -> StdResult<_> {
                Ok(total + bet_amount)
            })?;
        }
    };

    let home_total_bets = HOME_TOTAL_BETS.load(deps.storage)?;
    let away_total_bets = AWAY_TOTAL_BETS.load(deps.storage)?;

    let (new_home_odds, new_away_odds) =
        calculate_odds(&config, market_balance, home_total_bets, away_total_bets);
    market.home_odds = new_home_odds;
    market.away_odds = new_away_odds;
    MARKET.save(deps.storage, &market)?;

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
        .add_attribute("new_home_odds", new_home_odds.to_string())
        .add_attribute("new_away_odds", new_away_odds.to_string()))
}

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

    let bet = match market.result {
        Some(MarketResult::HOME) => HOME_BETS.may_load(deps.storage, addr.clone())?,
        Some(MarketResult::AWAY) => AWAY_BETS.may_load(deps.storage, addr.clone())?,
        None => None,
    };

    let mut payout = Uint128::zero();
    if let Some((odds, bet_amount)) = bet {
        if market.status == Status::CANCELLED {
            payout = bet_amount;
        } else {
            payout = bet_amount.multiply_ratio(odds.numerator(), odds.denominator());
        }
    }

    let mut messages: Vec<CosmosMsg> = vec![];
    if payout > Uint128::zero() {
        messages.push(
            BankMsg::Send {
                to_address: addr.to_string(),
                amount: vec![coin(payout.into(), config.denom)],
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

pub fn execute_update(
    deps: DepsMut,
    info: MessageInfo,
    fee_spread_odds: Option<Decimal>,
    max_bet_risk_factor: Option<Decimal>,
    seed_liquidity_amplifier: Option<Decimal>,
    initial_home_odds: Option<Decimal>,
    initial_away_odds: Option<Decimal>,
    start_timestamp: Option<u64>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    let mut market = MARKET.load(deps.storage)?;

    if info.sender != config.admin_addr {
        return Err(ContractError::Unauthorized {});
    }

    if market.status != Status::ACTIVE {
        return Err(ContractError::MarketNotActive {});
    }

    let mut fee_spread_odds_update = String::default();
    if let Some(fee_spread_odds) = fee_spread_odds {
        config.fee_spread_odds = fee_spread_odds;
        fee_spread_odds_update = fee_spread_odds.to_string();
    }

    let mut max_bet_risk_factor_update = String::default();
    if let Some(max_bet_risk_factor) = max_bet_risk_factor {
        config.max_bet_risk_factor = max_bet_risk_factor;
        max_bet_risk_factor_update = max_bet_risk_factor.to_string();
    }

    let mut seed_liquidity_amplifier_update = String::default();
    if let Some(seed_liquidity_amplifier) = seed_liquidity_amplifier {
        config.seed_liquidity_amplifier = seed_liquidity_amplifier;
        seed_liquidity_amplifier_update = seed_liquidity_amplifier.to_string();
    }

    let mut initial_home_odds_update = String::default();
    if let Some(initial_home_odds) = initial_home_odds {
        config.initial_home_odds = initial_home_odds;
        market.home_odds = initial_home_odds;
        initial_home_odds_update = initial_home_odds.to_string();
    }

    let mut initial_away_odds_update = String::default();
    if let Some(initial_away_odds) = initial_away_odds {
        config.initial_away_odds = initial_away_odds;
        market.away_odds = initial_away_odds;
        initial_away_odds_update = initial_away_odds.to_string();
    }

    let mut start_timestamp_update = String::default();
    if let Some(start_timestamp) = start_timestamp {
        market.start_timestamp = start_timestamp;
        start_timestamp_update = start_timestamp.to_string();
    }

    MARKET.save(deps.storage, &market)?;

    Ok(Response::new()
        .add_attribute("protocol", "vendetta-markets")
        .add_attribute("market_type", "fixed-odds")
        .add_attribute("action", "update_market")
        .add_attribute("sender", info.sender)
        .add_attribute("fee_spread_odds", fee_spread_odds_update)
        .add_attribute("max_bet_risk_factor", max_bet_risk_factor_update)
        .add_attribute("seed_liquidity_amplifier", seed_liquidity_amplifier_update)
        .add_attribute("initial_home_odds", initial_home_odds_update)
        .add_attribute("initial_away_odds", initial_away_odds_update)
        .add_attribute("start_timestamp", start_timestamp_update))
}

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
        .query_balance(env.contract.address, &config.denom)?
        .amount;
    let market_payout = match result {
        MarketResult::HOME => HOME_TOTAL_PAYOUT.load(deps.storage)?,
        MarketResult::AWAY => AWAY_TOTAL_PAYOUT.load(deps.storage)?,
    };

    let fee_collected = market_balance - market_payout;

    let mut messages: Vec<CosmosMsg> = vec![];
    if fee_collected > Uint128::zero() {
        messages.push(
            BankMsg::Send {
                to_address: config.treasury_addr.to_string(),
                amount: vec![coin(fee_collected.into(), &config.denom)],
            }
            .into(),
        );
    }

    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("protocol", "vendetta-markets")
        .add_attribute("market_type", "fixed-odds")
        .add_attribute("action", "score_market")
        .add_attribute("sender", info.sender)
        .add_attribute("status", Status::CLOSED.to_string())
        .add_attribute("result", result.to_string())
        .add_attribute("fee_collected", fee_collected))
}

pub fn execute_cancel(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
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

    Ok(Response::new()
        .add_attribute("protocol", "vendetta-markets")
        .add_attribute("market_type", "fixed-odds")
        .add_attribute("action", "cancel_market")
        .add_attribute("sender", info.sender)
        .add_attribute("status", Status::CANCELLED.to_string()))
}
