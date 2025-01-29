use cosmwasm_std::{
    coin, Addr, BankMsg, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};

use crate::{
    error::ContractError,
    logic::calculate_parimutuel_winnings,
    msg::UpdateParams,
    state::{
        MarketResult, Status, CLAIMS, CONFIG, MARKET, POOL_AWAY, POOL_DRAW, POOL_HOME, TOTAL_AWAY,
        TOTAL_DRAW, TOTAL_HOME,
    },
    validation::validate_fee_bps,
};

/// Places a bet on the market
///
/// The total result is updated and the pool result that records the bet amount per address is updated.
///
/// It will make the following checks:
/// - If the bet is on draw, the market needs to be drawable
/// - The market needs to be active
/// - The current block timestamp needs to be at least 5 minutes before the start timestamp
/// - The bet amount needs to be greater than zero
pub fn execute_place_bet(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    result: MarketResult,
    receiver: Option<Addr>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let market = MARKET.load(deps.storage)?;

    let addr = match receiver {
        Some(receiver) => deps.api.addr_validate(receiver.as_str())?,
        None => info.sender.clone(),
    };

    if !market.is_drawable && result == MarketResult::DRAW {
        return Err(ContractError::MarketNotDrawable {});
    }

    if market.status != Status::ACTIVE {
        return Err(ContractError::MarketNotActive {});
    }

    // Bets are accepted up until 5 minutes before the start of the match
    if market.start_timestamp - 5 * 60 < env.block.time.seconds() {
        return Err(ContractError::BetsNotAccepted {});
    }

    let bet_amount = cw_utils::must_pay(&info, &config.denom);
    if bet_amount.is_err() {
        return Err(ContractError::PaymentError {});
    }
    let bet_amount = bet_amount.unwrap();

    match result {
        MarketResult::HOME => {
            if !POOL_HOME.has(deps.storage, addr.clone()) {
                POOL_HOME.save(deps.storage, addr.clone(), &0)?;
            }

            POOL_HOME.update(deps.storage, addr.clone(), |pool| -> StdResult<_> {
                Ok(pool.unwrap() + bet_amount.u128())
            })?;
            TOTAL_HOME.update(deps.storage, |total| -> StdResult<_> {
                Ok(total + bet_amount.u128())
            })?
        }
        MarketResult::AWAY => {
            if !POOL_AWAY.has(deps.storage, addr.clone()) {
                POOL_AWAY.save(deps.storage, addr.clone(), &0)?;
            }

            POOL_AWAY.update(deps.storage, addr.clone(), |pool| -> StdResult<_> {
                Ok(pool.unwrap() + bet_amount.u128())
            })?;
            TOTAL_AWAY.update(deps.storage, |total| -> StdResult<_> {
                Ok(total + bet_amount.u128())
            })?
        }
        MarketResult::DRAW => {
            if !POOL_DRAW.has(deps.storage, addr.clone()) {
                POOL_DRAW.save(deps.storage, addr.clone(), &0)?;
            }

            POOL_DRAW.update(deps.storage, addr.clone(), |pool| -> StdResult<_> {
                Ok(pool.unwrap() + bet_amount.u128())
            })?;
            TOTAL_DRAW.update(deps.storage, |total| -> StdResult<_> {
                Ok(total + bet_amount.u128())
            })?
        }
    };

    Ok(Response::new()
        .add_attribute("protocol", "vendetta-markets")
        .add_attribute("market_type", "parimutuel")
        .add_attribute("action", "place_bet")
        .add_attribute("sender", info.sender)
        .add_attribute("receiver", addr)
        .add_attribute("bet_amount", bet_amount.to_string())
        .add_attribute("result", result.to_string())
        .add_attribute("total_home", TOTAL_HOME.load(deps.storage)?.to_string())
        .add_attribute("total_away", TOTAL_AWAY.load(deps.storage)?.to_string())
        .add_attribute("total_draw", TOTAL_DRAW.load(deps.storage)?.to_string()))
}

/// Claims winnings for the sender or the receiver if defined
/// or returns all bets made if the market was cancelled
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

    let payout;

    let addr_pool_home = if POOL_HOME.has(deps.storage, addr.clone()) {
        POOL_HOME.load(deps.storage, addr.clone())?
    } else {
        0
    };

    let addr_pool_away = if POOL_AWAY.has(deps.storage, addr.clone()) {
        POOL_AWAY.load(deps.storage, addr.clone())?
    } else {
        0
    };

    let addr_pool_draw = if POOL_DRAW.has(deps.storage, addr.clone()) {
        POOL_DRAW.load(deps.storage, addr.clone())?
    } else {
        0
    };

    if market.status == Status::CANCELLED {
        payout = addr_pool_home + addr_pool_away + addr_pool_draw;
    } else {
        let bet_amount = match market.result {
            Some(MarketResult::HOME) => addr_pool_home,
            Some(MarketResult::AWAY) => addr_pool_away,
            Some(MarketResult::DRAW) => addr_pool_draw,
            None => 0,
        };

        let total_home = TOTAL_HOME.load(deps.storage)?;
        let total_away = TOTAL_AWAY.load(deps.storage)?;
        let total_draw = TOTAL_DRAW.load(deps.storage)?;

        let team_bets = match market.result {
            Some(MarketResult::HOME) => total_home,
            Some(MarketResult::AWAY) => total_away,
            Some(MarketResult::DRAW) => total_draw,
            None => 0,
        };

        let mut fee_amount = Uint128::zero();
        if config.fee_bps > 0 {
            fee_amount = Uint128::from(total_home + total_away + total_draw)
                .multiply_ratio(Uint128::from(config.fee_bps), Uint128::from(10000_u128));
        }

        payout = calculate_parimutuel_winnings(
            total_home + total_away + total_draw - fee_amount.u128(),
            team_bets,
            bet_amount,
        );
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
        .add_attribute("market_type", "parimutuel")
        .add_attribute("action", "claim_winnings")
        .add_attribute("sender", info.sender)
        .add_attribute("receiver", addr)
        .add_attribute("payout", payout.to_string()))
}

/// Updates the market with the new params
///
/// It will make the following checks:
/// - The sender needs to be the admin
/// - The market needs to be active
pub fn execute_update(
    deps: DepsMut,
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

    let mut fee_bps_update = String::default();
    if let Some(fee_bps) = params.fee_bps {
        validate_fee_bps(fee_bps)?;
        config.fee_bps = fee_bps;
        fee_bps_update = fee_bps.to_string();
    }

    let mut start_timestamp_update = String::default();
    if let Some(start_timestamp) = params.start_timestamp {
        market.start_timestamp = start_timestamp;
        start_timestamp_update = start_timestamp.to_string();
    }

    CONFIG.save(deps.storage, &config)?;
    MARKET.save(deps.storage, &market)?;

    Ok(Response::new()
        .add_attribute("protocol", "vendetta-markets")
        .add_attribute("market_type", "parimutuel")
        .add_attribute("action", "update_market")
        .add_attribute("sender", info.sender)
        .add_attribute("admin_addr", admin_addr_update)
        .add_attribute("treasury_addr", treasury_addr_update)
        .add_attribute("fee_bps", fee_bps_update)
        .add_attribute("start_timestamp", start_timestamp_update)
        .add_attribute("total_home", TOTAL_HOME.load(deps.storage)?.to_string())
        .add_attribute("total_away", TOTAL_AWAY.load(deps.storage)?.to_string())
        .add_attribute("total_draw", TOTAL_DRAW.load(deps.storage)?.to_string()))
}

/// Scores the market and collects fees to the treasury,
/// based on the fee_bps in the config
///
/// It will make the following checks:
/// - The sender needs to be the admin
/// - If the result is DRAW, the market needs to be drawable
/// - The market needs to be active
/// - The current block timestamp needs to be at least 30 minutes after the start timestamp
/// - There needs to be some winnings
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

    if !market.is_drawable && result == MarketResult::DRAW {
        return Err(ContractError::MarketNotDrawable {});
    }

    if market.status != Status::ACTIVE {
        return Err(ContractError::MarketNotActive {});
    }

    // Market can only be scored after 30 minutes of its start timestamp
    if env.block.time.seconds() < market.start_timestamp + 30 * 60 {
        return Err(ContractError::MarketNotScoreable {});
    }

    let total_home = TOTAL_HOME.load(deps.storage)?;
    let total_away = TOTAL_AWAY.load(deps.storage)?;
    let total_draw = TOTAL_DRAW.load(deps.storage)?;

    let winning_side = match result {
        MarketResult::HOME => total_home,
        MarketResult::AWAY => total_away,
        MarketResult::DRAW => total_draw,
    };

    let losing_side = match result {
        MarketResult::HOME => total_away + total_draw,
        MarketResult::AWAY => total_home + total_draw,
        MarketResult::DRAW => total_home + total_away,
    };

    if winning_side == 0 || losing_side == 0 {
        return Err(ContractError::NoWinnings {});
    }

    let mut market = market;
    market.status = Status::CLOSED;
    market.result = Some(result.clone());
    MARKET.save(deps.storage, &market)?;

    let mut fee_amount = Uint128::zero();
    if config.fee_bps > 0 {
        fee_amount = Uint128::from(total_home + total_away + total_draw)
            .multiply_ratio(Uint128::from(config.fee_bps), Uint128::from(10000_u128));
    }

    let mut messages: Vec<CosmosMsg> = vec![];

    if fee_amount > Uint128::zero() {
        messages.push(
            BankMsg::Send {
                to_address: config.treasury_addr.to_string(),
                amount: vec![coin(fee_amount.into(), config.denom)],
            }
            .into(),
        );
    }

    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("protocol", "vendetta-markets")
        .add_attribute("market_type", "parimutuel")
        .add_attribute("action", "score_market")
        .add_attribute("sender", info.sender)
        .add_attribute("status", Status::CLOSED.to_string())
        .add_attribute("result", result.to_string())
        .add_attribute("fee_collected", fee_amount)
        .add_attribute("total_home", total_home.to_string())
        .add_attribute("total_away", total_away.to_string())
        .add_attribute("total_draw", total_draw.to_string()))
}

/// Cancels the market
///
/// It will make the following checks:
/// - The sender needs to be the admin
/// - The market needs to be active
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
        .add_attribute("market_type", "parimutuel")
        .add_attribute("action", "cancel_market")
        .add_attribute("sender", info.sender)
        .add_attribute("status", Status::CANCELLED.to_string())
        .add_attribute("total_home", TOTAL_HOME.load(deps.storage)?.to_string())
        .add_attribute("total_away", TOTAL_AWAY.load(deps.storage)?.to_string())
        .add_attribute("total_draw", TOTAL_DRAW.load(deps.storage)?.to_string()))
}
