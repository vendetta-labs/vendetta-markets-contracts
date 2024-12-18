use cosmwasm_std::{coin, Addr, BankMsg, CosmosMsg, DepsMut, Env, MessageInfo, Response, Uint128};

use crate::{
    state::{MarketResult, Status, CLAIMS, CONFIG, MARKET},
    ContractError,
};

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
        MarketResult::HOME => {}
        MarketResult::AWAY => {}
    };

    Ok(Response::new()
        .add_attribute("protocol", "vendetta-markets")
        .add_attribute("market_type", "fixed-odds")
        .add_attribute("action", "place_bet")
        .add_attribute("sender", info.sender)
        .add_attribute("receiver", addr)
        .add_attribute("bet_amount", bet_amount.to_string())
        .add_attribute("result", result.to_string()))
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

    let payout = 0;

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

pub fn execute_update(
    deps: DepsMut,
    info: MessageInfo,
    start_timestamp: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let market = MARKET.load(deps.storage)?;

    if info.sender != config.admin_addr {
        return Err(ContractError::Unauthorized {});
    }

    if market.status != Status::ACTIVE {
        return Err(ContractError::MarketNotActive {});
    }

    let mut market = MARKET.load(deps.storage)?;
    market.start_timestamp = start_timestamp;

    MARKET.save(deps.storage, &market)?;

    Ok(Response::new()
        .add_attribute("protocol", "vendetta-markets")
        .add_attribute("market_type", "fixed-odds")
        .add_attribute("action", "update_market")
        .add_attribute("sender", info.sender)
        .add_attribute("start_timestamp", start_timestamp.to_string()))
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

    let mut fee_amount = Uint128::zero();
    if config.fee_bps > 0 {
        fee_amount = Uint128::zero();
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
        .add_attribute("market_type", "fixed-odds")
        .add_attribute("action", "score_market")
        .add_attribute("sender", info.sender)
        .add_attribute("status", Status::CLOSED.to_string())
        .add_attribute("result", result.to_string())
        .add_attribute("fee_collected", fee_amount))
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
