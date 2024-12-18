use cosmwasm_std::{
    coin, Addr, BankMsg, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};

use crate::{
    calculate_parimutuel_winnings,
    state::{
        MarketResult, Status, CLAIMS, CONFIG, MARKET, POOL_AWAY, POOL_DRAW, POOL_HOME, TOTAL_AWAY,
        TOTAL_DRAW, TOTAL_HOME,
    },
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
        .add_attribute("market_type", "parimutuel")
        .add_attribute("action", "update_market")
        .add_attribute("sender", info.sender)
        .add_attribute("start_timestamp", start_timestamp.to_string())
        .add_attribute("total_home", TOTAL_HOME.load(deps.storage)?.to_string())
        .add_attribute("total_away", TOTAL_AWAY.load(deps.storage)?.to_string())
        .add_attribute("total_draw", TOTAL_DRAW.load(deps.storage)?.to_string()))
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

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::contract::{execute, instantiate, query, ADMIN_ADDRESS, TREASURY_ADDRESS};
    use crate::msg::{BetsResponse, ExecuteMsg, InstantiateMsg, MarketResponse, QueryMsg};

    use super::*;
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
    use cosmwasm_std::{from_json, Timestamp};

    const NATIVE_DENOM: &str = "denom";

    mod place_bet {
        use super::*;

        #[test]
        fn it_properly_accepts_bets() {
            let mut deps = mock_dependencies();
            let block_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs();
            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(block_timestamp);

            let start_timestamp = block_timestamp + 60 * 5; // 5 minutes from now
            let msg = InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                fee_bps: 250,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            };

            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

            let anyone = deps.api.addr_make("ANYONE");
            let info = message_info(&anyone, &[coin(1_000, NATIVE_DENOM)]);
            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::AWAY,
                receiver: None,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
            let value: BetsResponse = from_json(&res).unwrap();
            assert_eq!(0, value.totals.total_draw);
            assert_eq!(1_000, value.totals.total_away);
            assert_eq!(0, value.totals.total_home);

            let res = query(
                deps.as_ref(),
                env.clone(),
                QueryMsg::BetsByAddress {
                    address: anyone.clone(),
                },
            )
            .unwrap();
            let value: BetsResponse = from_json(&res).unwrap();
            assert_eq!(0, value.totals.total_draw);
            assert_eq!(1_000, value.totals.total_away);
            assert_eq!(0, value.totals.total_home);
        }

        #[test]
        fn the_receiver_will_be_the_beneficiary_when_defined() {
            let mut deps = mock_dependencies();
            let block_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs();
            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(block_timestamp);

            let start_timestamp = block_timestamp + 60 * 5; // 5 minutes from now
            let msg = InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                fee_bps: 250,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            };

            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

            let anyone = deps.api.addr_make("ANYONE");
            let other = deps.api.addr_make("OTHER");
            let info = message_info(&anyone, &[coin(1_000, NATIVE_DENOM)]);
            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::AWAY,
                receiver: Some(other.clone()),
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
            let value: BetsResponse = from_json(&res).unwrap();
            assert_eq!(1_000, value.totals.total_away);

            let res = query(
                deps.as_ref(),
                env.clone(),
                QueryMsg::BetsByAddress {
                    address: anyone.clone(),
                },
            )
            .unwrap();
            let value: BetsResponse = from_json(&res).unwrap();
            assert_eq!(0, value.totals.total_away);

            let res = query(
                deps.as_ref(),
                env.clone(),
                QueryMsg::BetsByAddress {
                    address: other.clone(),
                },
            )
            .unwrap();
            let value: BetsResponse = from_json(&res).unwrap();
            assert_eq!(1_000, value.totals.total_away);
        }

        #[test]
        fn it_cant_place_bet_on_draw_when_market_isnt_drawable() {
            let mut deps = mock_dependencies();
            let block_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs();
            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(block_timestamp);

            let start_timestamp = block_timestamp + 60 * 5; // 5 minutes from now
            let msg = InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                fee_bps: 250,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: false,
            };

            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

            let anyone = deps.api.addr_make("ANYONE");
            let info = message_info(&anyone, &[coin(1_000, NATIVE_DENOM)]);
            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::DRAW,
                receiver: None,
            };
            let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
            assert_eq!(ContractError::MarketNotDrawable {}, res);

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
            let value: BetsResponse = from_json(&res).unwrap();
            assert_eq!(0, value.totals.total_draw);
            assert_eq!(0, value.totals.total_away);
            assert_eq!(0, value.totals.total_home);

            let res = query(
                deps.as_ref(),
                env.clone(),
                QueryMsg::BetsByAddress {
                    address: anyone.clone(),
                },
            )
            .unwrap();
            let value: BetsResponse = from_json(&res).unwrap();
            assert_eq!(0, value.totals.total_draw);
            assert_eq!(0, value.totals.total_away);
            assert_eq!(0, value.totals.total_home);
        }

        #[test]
        fn it_cant_place_bet_if_market_isnt_active() {
            let mut deps = mock_dependencies();
            let block_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs();
            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(block_timestamp);

            let start_timestamp = block_timestamp + 60 * 5; // 5 minutes from now
            let msg = InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                fee_bps: 250,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            };

            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
            let msg = ExecuteMsg::Cancel {};
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
            let value: MarketResponse = from_json(&res).unwrap();
            assert_ne!(Status::ACTIVE, value.market.status);

            let anyone = deps.api.addr_make("ANYONE");
            let info = message_info(&anyone, &[coin(1_000, NATIVE_DENOM)]);
            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::DRAW,
                receiver: None,
            };
            let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
            assert_eq!(ContractError::MarketNotActive {}, res);

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
            let value: BetsResponse = from_json(&res).unwrap();
            assert_eq!(0, value.totals.total_draw);
            assert_eq!(0, value.totals.total_away);
            assert_eq!(0, value.totals.total_home);

            let res = query(
                deps.as_ref(),
                env.clone(),
                QueryMsg::BetsByAddress {
                    address: anyone.clone(),
                },
            )
            .unwrap();
            let value: BetsResponse = from_json(&res).unwrap();
            assert_eq!(0, value.totals.total_draw);
            assert_eq!(0, value.totals.total_away);
            assert_eq!(0, value.totals.total_home);
        }

        #[test]
        fn it_can_only_place_bets_up_until_5_minutes_before_market_start_timestamp() {
            let mut deps = mock_dependencies();
            let block_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs();
            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(block_timestamp);

            let start_timestamp = block_timestamp + 60 * 5 - 1; // 4 minutes and 59 seconds from now
            let msg = InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                fee_bps: 250,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            };

            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

            let anyone = deps.api.addr_make("ANYONE");
            let info = message_info(&anyone, &[coin(1_000, NATIVE_DENOM)]);
            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::AWAY,
                receiver: None,
            };
            let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
            assert_eq!(ContractError::BetsNotAccepted {}, res);

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
            let value: BetsResponse = from_json(&res).unwrap();
            assert_eq!(0, value.totals.total_draw);
            assert_eq!(0, value.totals.total_away);
            assert_eq!(0, value.totals.total_home);

            let res = query(
                deps.as_ref(),
                env.clone(),
                QueryMsg::BetsByAddress {
                    address: anyone.clone(),
                },
            )
            .unwrap();
            let value: BetsResponse = from_json(&res).unwrap();
            assert_eq!(0, value.totals.total_draw);
            assert_eq!(0, value.totals.total_away);
            assert_eq!(0, value.totals.total_home);
        }

        #[test]
        fn it_cant_place_bet_without_sending_funds_in_the_market_denom() {
            let mut deps = mock_dependencies();
            let block_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs();
            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(block_timestamp);

            let start_timestamp = block_timestamp + 60 * 5; // 5 minutes from now
            let msg = InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                fee_bps: 250,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            };

            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

            let anyone = deps.api.addr_make("ANYONE");
            let info = message_info(&anyone, &[coin(1_000, "otherdenom")]);
            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::AWAY,
                receiver: None,
            };
            let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
            assert_eq!(ContractError::PaymentError {}, res);

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
            let value: BetsResponse = from_json(&res).unwrap();
            assert_eq!(0, value.totals.total_draw);
            assert_eq!(0, value.totals.total_away);
            assert_eq!(0, value.totals.total_home);

            let res = query(
                deps.as_ref(),
                env.clone(),
                QueryMsg::BetsByAddress {
                    address: anyone.clone(),
                },
            )
            .unwrap();
            let value: BetsResponse = from_json(&res).unwrap();
            assert_eq!(0, value.totals.total_draw);
            assert_eq!(0, value.totals.total_away);
            assert_eq!(0, value.totals.total_home);
        }
    }

    mod claim_winnings {
        use super::*;

        #[test]
        fn it_properly_claims_winnings() {
            let mut deps = mock_dependencies();
            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs(),
            );

            let start_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 10; // 10 minutes from now
            let msg = InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                fee_bps: 250,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            };
            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

            let anyone = deps.api.addr_make("ANYONE");
            let info = message_info(&anyone, &[coin(1_000, NATIVE_DENOM)]);
            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::DRAW,
                receiver: None,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let other = deps.api.addr_make("OTHER");
            let info = message_info(&other, &[coin(1_000, NATIVE_DENOM)]);
            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::AWAY,
                receiver: None,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
            let value: BetsResponse = from_json(&res).unwrap();
            assert_eq!(1_000, value.totals.total_draw);
            assert_eq!(1_000, value.totals.total_away);
            assert_eq!(0, value.totals.total_home);

            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs()
                    + 60 * 45, // 45 minutes in the future
            );

            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            let msg = ExecuteMsg::Score {
                result: MarketResult::DRAW,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
            let value: MarketResponse = from_json(&res).unwrap();
            assert_eq!(Status::CLOSED, value.market.status);
            assert_eq!(MarketResult::DRAW, value.market.result.unwrap());

            let info = message_info(&anyone, &[]);
            let msg = ExecuteMsg::ClaimWinnings { receiver: None };
            let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
            assert_eq!(1, res.messages.len());
            let send: CosmosMsg = res.messages.first().unwrap().msg.clone();
            match send {
                CosmosMsg::Bank(bank_msg) => match bank_msg {
                    BankMsg::Send { to_address, amount } => {
                        assert_eq!(to_address, anyone.to_string());
                        assert_eq!(amount, vec![coin(1_950, NATIVE_DENOM)]);
                    }
                    _ => panic!("Unexpected message: {:?}", bank_msg),
                },
                _ => panic!("Unexpected message: {:?}", send),
            }
        }

        #[test]
        fn it_can_claim_on_behalf_of_the_receiver_when_defined() {
            let mut deps = mock_dependencies();
            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs(),
            );

            let start_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 10; // 10 minutes from now
            let msg = InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                fee_bps: 250,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            };
            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

            let anyone = deps.api.addr_make("ANYONE");
            let info = message_info(&anyone, &[coin(1_000, NATIVE_DENOM)]);
            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::DRAW,
                receiver: None,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let other = deps.api.addr_make("OTHER");
            let info = message_info(&other, &[coin(1_000, NATIVE_DENOM)]);
            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::AWAY,
                receiver: None,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
            let value: BetsResponse = from_json(&res).unwrap();
            assert_eq!(1_000, value.totals.total_draw);
            assert_eq!(1_000, value.totals.total_away);
            assert_eq!(0, value.totals.total_home);

            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs()
                    + 60 * 45, // 45 minutes in the future
            );

            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            let msg = ExecuteMsg::Score {
                result: MarketResult::DRAW,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
            let value: MarketResponse = from_json(&res).unwrap();
            assert_eq!(Status::CLOSED, value.market.status);
            assert_eq!(MarketResult::DRAW, value.market.result.unwrap());

            let info = message_info(&other, &[]);
            let msg = ExecuteMsg::ClaimWinnings {
                receiver: Some(anyone.clone()),
            };
            let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
            assert_eq!(1, res.messages.len());
            let send: CosmosMsg = res.messages.first().unwrap().msg.clone();
            match send {
                CosmosMsg::Bank(bank_msg) => match bank_msg {
                    BankMsg::Send { to_address, amount } => {
                        assert_eq!(to_address, anyone.to_string());
                        assert_eq!(amount, vec![coin(1_950, NATIVE_DENOM)]);
                    }
                    _ => panic!("Unexpected message: {:?}", bank_msg),
                },
                _ => panic!("Unexpected message: {:?}", send),
            }
        }

        #[test]
        fn it_will_return_all_bets_made_if_market_was_cancelled() {
            let mut deps = mock_dependencies();
            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs(),
            );

            let start_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 10; // 10 minutes from now
            let msg = InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                fee_bps: 250,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            };
            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

            let anyone = deps.api.addr_make("ANYONE");
            let info = message_info(&anyone, &[coin(1_000, NATIVE_DENOM)]);
            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::DRAW,
                receiver: None,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let other = deps.api.addr_make("OTHER");
            let info = message_info(&other, &[coin(750, NATIVE_DENOM)]);
            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::AWAY,
                receiver: None,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
            let value: BetsResponse = from_json(&res).unwrap();
            assert_eq!(1_000, value.totals.total_draw);
            assert_eq!(750, value.totals.total_away);
            assert_eq!(0, value.totals.total_home);

            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs()
                    + 60 * 45, // 45 minutes in the future
            );

            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            let msg = ExecuteMsg::Cancel {};
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
            let value: MarketResponse = from_json(&res).unwrap();
            assert_eq!(Status::CANCELLED, value.market.status);
            assert_eq!(None, value.market.result);

            let info = message_info(&anyone, &[]);
            let msg = ExecuteMsg::ClaimWinnings { receiver: None };
            let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
            assert_eq!(1, res.messages.len());
            let send: CosmosMsg = res.messages.first().unwrap().msg.clone();
            match send {
                CosmosMsg::Bank(bank_msg) => match bank_msg {
                    BankMsg::Send { to_address, amount } => {
                        assert_eq!(to_address, anyone.to_string());
                        assert_eq!(amount, vec![coin(1_000, NATIVE_DENOM)]);
                    }
                    _ => panic!("Unexpected message: {:?}", bank_msg),
                },
                _ => panic!("Unexpected message: {:?}", send),
            }

            let info = message_info(&other, &[]);
            let msg = ExecuteMsg::ClaimWinnings { receiver: None };
            let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
            assert_eq!(1, res.messages.len());
            let send: CosmosMsg = res.messages.first().unwrap().msg.clone();
            match send {
                CosmosMsg::Bank(bank_msg) => match bank_msg {
                    BankMsg::Send { to_address, amount } => {
                        assert_eq!(to_address, other.to_string());
                        assert_eq!(amount, vec![coin(750, NATIVE_DENOM)]);
                    }
                    _ => panic!("Unexpected message: {:?}", bank_msg),
                },
                _ => panic!("Unexpected message: {:?}", send),
            }
        }

        #[test]
        fn it_cant_claim_winnings_while_market_is_active() {
            let mut deps = mock_dependencies();
            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs(),
            );

            let start_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 10; // 10 minutes from now
            let msg = InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                fee_bps: 250,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            };
            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

            let anyone = deps.api.addr_make("ANYONE");
            let info = message_info(&anyone, &[coin(1_000, NATIVE_DENOM)]);
            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::DRAW,
                receiver: None,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let other = deps.api.addr_make("OTHER");
            let info = message_info(&other, &[coin(1_000, NATIVE_DENOM)]);
            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::AWAY,
                receiver: None,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
            let value: BetsResponse = from_json(&res).unwrap();
            assert_eq!(1_000, value.totals.total_draw);
            assert_eq!(1_000, value.totals.total_away);
            assert_eq!(0, value.totals.total_home);

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
            let value: MarketResponse = from_json(&res).unwrap();
            assert_eq!(Status::ACTIVE, value.market.status);
            assert_eq!(None, value.market.result);

            let info = message_info(&anyone, &[]);
            let msg = ExecuteMsg::ClaimWinnings { receiver: None };
            let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
            assert_eq!(ContractError::MarketNotClosed {}, res);
        }

        #[test]
        fn it_cant_claim_winnings_twice() {
            let mut deps = mock_dependencies();
            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs(),
            );

            let start_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 10; // 10 minutes from now
            let msg = InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                fee_bps: 250,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            };
            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

            let anyone = deps.api.addr_make("ANYONE");
            let info = message_info(&anyone, &[coin(1_000, NATIVE_DENOM)]);
            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::DRAW,
                receiver: None,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let other = deps.api.addr_make("OTHER");
            let info = message_info(&other, &[coin(1_000, NATIVE_DENOM)]);
            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::AWAY,
                receiver: None,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
            let value: BetsResponse = from_json(&res).unwrap();
            assert_eq!(1_000, value.totals.total_draw);
            assert_eq!(1_000, value.totals.total_away);
            assert_eq!(0, value.totals.total_home);

            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs()
                    + 60 * 45, // 45 minutes in the future
            );

            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            let msg = ExecuteMsg::Score {
                result: MarketResult::DRAW,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
            let value: MarketResponse = from_json(&res).unwrap();
            assert_eq!(Status::CLOSED, value.market.status);
            assert_eq!(MarketResult::DRAW, value.market.result.unwrap());

            let info = message_info(&anyone, &[]);
            let msg = ExecuteMsg::ClaimWinnings { receiver: None };
            let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
            assert_eq!(1, res.messages.len());
            let send: CosmosMsg = res.messages.first().unwrap().msg.clone();
            match send {
                CosmosMsg::Bank(bank_msg) => match bank_msg {
                    BankMsg::Send { to_address, amount } => {
                        assert_eq!(to_address, anyone.to_string());
                        assert_eq!(amount, vec![coin(1_950, NATIVE_DENOM)]);
                    }
                    _ => panic!("Unexpected message: {:?}", bank_msg),
                },
                _ => panic!("Unexpected message: {:?}", send),
            }

            let info = message_info(&anyone, &[]);
            let msg = ExecuteMsg::ClaimWinnings { receiver: None };
            let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
            assert_eq!(ContractError::ClaimAlreadyMade {}, res);
        }

        #[test]
        fn it_cant_claim_winnings_when_there_is_nothing_to_claim() {
            let mut deps = mock_dependencies();
            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs(),
            );

            let start_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 10; // 10 minutes from now
            let msg = InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                fee_bps: 250,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            };
            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

            let anyone = deps.api.addr_make("ANYONE");
            let info = message_info(&anyone, &[coin(1_000, NATIVE_DENOM)]);
            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::DRAW,
                receiver: None,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let other = deps.api.addr_make("OTHER");
            let info = message_info(&other, &[coin(1_000, NATIVE_DENOM)]);
            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::AWAY,
                receiver: None,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
            let value: BetsResponse = from_json(&res).unwrap();
            assert_eq!(1_000, value.totals.total_draw);
            assert_eq!(1_000, value.totals.total_away);
            assert_eq!(0, value.totals.total_home);

            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs()
                    + 60 * 45, // 45 minutes in the future
            );

            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            let msg = ExecuteMsg::Score {
                result: MarketResult::DRAW,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
            let value: MarketResponse = from_json(&res).unwrap();
            assert_eq!(Status::CLOSED, value.market.status);
            assert_eq!(MarketResult::DRAW, value.market.result.unwrap());

            let info = message_info(&other, &[]);
            let msg = ExecuteMsg::ClaimWinnings { receiver: None };
            let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
            assert_eq!(ContractError::NoWinnings {}, res);
        }
    }

    mod update_market {
        use super::*;

        #[test]
        fn proper_update_market() {
            let mut deps = mock_dependencies();
            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs(), // Now
            );

            let start_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                - 60 * 5; // 5 minutes ago
            let msg = InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                fee_bps: 250,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            };
            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
            let value: MarketResponse = from_json(&res).unwrap();
            assert_eq!(start_timestamp, value.market.start_timestamp);

            let new_start_timestamp = start_timestamp - 60 * 30; // 30 minutes ago
            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            let msg = ExecuteMsg::Update {
                start_timestamp: new_start_timestamp,
            };
            let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
            assert_eq!(0, res.messages.len());

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
            let value: MarketResponse = from_json(&res).unwrap();
            assert_eq!(new_start_timestamp, value.market.start_timestamp);
        }

        #[test]
        fn unauthorized() {
            let mut deps = mock_dependencies();
            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs(), // Now
            );

            let start_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 5; // 5 minutes from now
            let msg = InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                fee_bps: 250,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            };
            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
            let value: MarketResponse = from_json(&res).unwrap();
            assert_eq!(start_timestamp, value.market.start_timestamp);

            let anyone = deps.api.addr_make("ANYONE");
            let info = message_info(&anyone, &[]);
            let msg = ExecuteMsg::Update {
                start_timestamp: start_timestamp - 60 * 30, // 30 minutes ago
            };
            let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
            assert_eq!(ContractError::Unauthorized {}, res);

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
            let value: MarketResponse = from_json(&res).unwrap();
            assert_eq!(start_timestamp, value.market.start_timestamp);
        }

        #[test]
        fn market_not_active() {
            let mut deps = mock_dependencies();
            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs(),
            );

            let start_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 10; // 10 minutes from now
            let msg = InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                fee_bps: 250,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            };
            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

            let info = message_info(
                &Addr::unchecked(ADMIN_ADDRESS),
                &[coin(1_000, NATIVE_DENOM)],
            );
            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::DRAW,
                receiver: None,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let info = message_info(
                &Addr::unchecked(ADMIN_ADDRESS),
                &[coin(1_000, NATIVE_DENOM)],
            );
            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::AWAY,
                receiver: None,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
            let value: BetsResponse = from_json(&res).unwrap();
            assert_eq!(1_000, value.totals.total_draw);
            assert_eq!(1_000, value.totals.total_away);
            assert_eq!(0, value.totals.total_home);

            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs()
                    + 60 * 45, // 45 minutes in the future
            );

            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            let msg = ExecuteMsg::Score {
                result: MarketResult::DRAW,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
            let value: MarketResponse = from_json(&res).unwrap();
            assert_ne!(Status::ACTIVE, value.market.status);

            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            let msg = ExecuteMsg::Update {
                start_timestamp: start_timestamp - 60 * 30, // 30 minutes ago
            };
            let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
            assert_eq!(ContractError::MarketNotActive {}, res);

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
            let value: MarketResponse = from_json(&res).unwrap();
            assert_eq!(start_timestamp, value.market.start_timestamp);
        }
    }

    mod score_market {
        use super::*;

        #[test]
        fn proper_score_market_and_collect_fees() {
            let mut deps = mock_dependencies();
            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs(),
            );

            let start_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 10; // 10 minutes from now
            let msg = InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                fee_bps: 250,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            };
            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

            let info = message_info(
                &Addr::unchecked(ADMIN_ADDRESS),
                &[coin(1_000, NATIVE_DENOM)],
            );
            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::DRAW,
                receiver: None,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let info = message_info(
                &Addr::unchecked(ADMIN_ADDRESS),
                &[coin(1_000, NATIVE_DENOM)],
            );
            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::AWAY,
                receiver: None,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
            let value: BetsResponse = from_json(&res).unwrap();
            assert_eq!(1_000, value.totals.total_draw);
            assert_eq!(1_000, value.totals.total_away);
            assert_eq!(0, value.totals.total_home);

            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs()
                    + 60 * 45, // 45 minutes in the future
            );

            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            let msg = ExecuteMsg::Score {
                result: MarketResult::DRAW,
            };
            let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
            assert_eq!(1, res.messages.len());
            let send: CosmosMsg = res.messages.first().unwrap().msg.clone();
            match send {
                CosmosMsg::Bank(bank_msg) => match bank_msg {
                    BankMsg::Send { to_address, amount } => {
                        assert_eq!(to_address, TREASURY_ADDRESS);
                        assert_eq!(amount, vec![coin(50, NATIVE_DENOM)]);
                    }
                    _ => panic!("Unexpected message: {:?}", bank_msg),
                },
                _ => panic!("Unexpected message: {:?}", send),
            }

            let fee_event = res
                .attributes
                .iter()
                .find(|attribute| attribute.key == "fee_collected");
            assert_eq!("50", fee_event.unwrap().value);

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
            let value: MarketResponse = from_json(&res).unwrap();
            assert_eq!(Status::CLOSED, value.market.status);
            assert_eq!(MarketResult::DRAW, value.market.result.unwrap());
        }

        #[test]
        fn do_not_collect_fees_when_set_to_zero() {
            let mut deps = mock_dependencies();
            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs(),
            );

            let start_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 10; // 10 minutes from now
            let msg = InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                fee_bps: 0,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            };
            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

            let info = message_info(
                &Addr::unchecked(ADMIN_ADDRESS),
                &[coin(1_000, NATIVE_DENOM)],
            );
            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::DRAW,
                receiver: None,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let info = message_info(
                &Addr::unchecked(ADMIN_ADDRESS),
                &[coin(1_000, NATIVE_DENOM)],
            );
            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::AWAY,
                receiver: None,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
            let value: BetsResponse = from_json(&res).unwrap();
            assert_eq!(1_000, value.totals.total_draw);
            assert_eq!(1_000, value.totals.total_away);
            assert_eq!(0, value.totals.total_home);

            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs()
                    + 60 * 45, // 45 minutes in the future
            );

            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            let msg = ExecuteMsg::Score {
                result: MarketResult::DRAW,
            };
            let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
            assert_eq!(0, res.messages.len());
            let fee_event = res
                .attributes
                .iter()
                .find(|attribute| attribute.key == "fee_collected");
            assert_eq!("0", fee_event.unwrap().value);

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
            let value: MarketResponse = from_json(&res).unwrap();
            assert_eq!(Status::CLOSED, value.market.status);
            assert_eq!(MarketResult::DRAW, value.market.result.unwrap());
        }

        #[test]
        fn unauthorized() {
            let mut deps = mock_dependencies();
            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs(),
            );

            let start_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 10; // 10 minutes from now
            let msg = InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                fee_bps: 250,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            };
            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

            let info = message_info(
                &Addr::unchecked(ADMIN_ADDRESS),
                &[coin(1_000, NATIVE_DENOM)],
            );
            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::HOME,
                receiver: None,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let info = message_info(
                &Addr::unchecked(ADMIN_ADDRESS),
                &[coin(1_000, NATIVE_DENOM)],
            );
            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::DRAW,
                receiver: None,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
            let value: BetsResponse = from_json(&res).unwrap();
            assert_eq!(1_000, value.totals.total_draw);
            assert_eq!(0, value.totals.total_away);
            assert_eq!(1_000, value.totals.total_home);

            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs()
                    + 60 * 45, // 45 minutes in the future
            );

            let anyone = deps.api.addr_make("ANYONE");
            let info = message_info(&anyone, &[]);
            let msg = ExecuteMsg::Score {
                result: MarketResult::DRAW,
            };
            let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
            assert_eq!(ContractError::Unauthorized {}, res);

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
            let value: MarketResponse = from_json(&res).unwrap();
            assert_eq!(Status::ACTIVE, value.market.status);
            assert_eq!(None, value.market.result);
        }

        #[test]
        fn market_not_drawable() {
            let mut deps = mock_dependencies();
            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs(),
            );

            let start_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 10; // 10 minutes from now
            let msg = InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                fee_bps: 250,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: false,
            };
            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

            let info = message_info(
                &Addr::unchecked(ADMIN_ADDRESS),
                &[coin(1_000, NATIVE_DENOM)],
            );
            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::HOME,
                receiver: None,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let info = message_info(
                &Addr::unchecked(ADMIN_ADDRESS),
                &[coin(1_000, NATIVE_DENOM)],
            );
            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::AWAY,
                receiver: None,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
            let value: BetsResponse = from_json(&res).unwrap();
            assert_eq!(0, value.totals.total_draw);
            assert_eq!(1_000, value.totals.total_away);
            assert_eq!(1_000, value.totals.total_home);

            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs()
                    + 60 * 45, // 45 minutes in the future
            );

            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            let msg = ExecuteMsg::Score {
                result: MarketResult::DRAW,
            };
            let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
            assert_eq!(ContractError::MarketNotDrawable {}, res);

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
            let value: MarketResponse = from_json(&res).unwrap();
            assert_eq!(Status::ACTIVE, value.market.status);
            assert_eq!(None, value.market.result);
        }

        #[test]
        fn market_not_active() {
            let mut deps = mock_dependencies();
            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs(),
            );

            let start_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 10; // 10 minutes from now
            let msg = InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                fee_bps: 250,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            };
            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs()
                    + 60 * 45, // 45 minutes in the future
            );

            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            let msg = ExecuteMsg::Cancel {};
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
            let value: MarketResponse = from_json(&res).unwrap();
            assert_ne!(Status::ACTIVE, value.market.status);
            assert_eq!(None, value.market.result);

            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            let msg = ExecuteMsg::Score {
                result: MarketResult::DRAW,
            };
            let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
            assert_eq!(ContractError::MarketNotActive {}, res);

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
            let value: MarketResponse = from_json(&res).unwrap();
            assert_eq!(None, value.market.result);
        }

        #[test]
        fn market_not_scoreable() {
            let mut deps = mock_dependencies();
            let mut env = mock_env();
            let current_block_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs();
            env.block.time = Timestamp::from_seconds(current_block_time);

            let start_timestamp = current_block_time + 60 * 5; // 5 minutes from block time
            let msg = InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                fee_bps: 250,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            };
            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

            let info = message_info(
                &Addr::unchecked(ADMIN_ADDRESS),
                &[coin(1_000, NATIVE_DENOM)],
            );
            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::DRAW,
                receiver: None,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let info = message_info(
                &Addr::unchecked(ADMIN_ADDRESS),
                &[coin(1_000, NATIVE_DENOM)],
            );
            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::HOME,
                receiver: None,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
            let value: BetsResponse = from_json(&res).unwrap();
            assert_eq!(1_000, value.totals.total_draw);
            assert_eq!(0, value.totals.total_away);
            assert_eq!(1_000, value.totals.total_home);

            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                start_timestamp + 60 * 30 - 1, // 29 minutes and 59 seconds from start timestamp
            );
            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            let msg = ExecuteMsg::Score {
                result: MarketResult::DRAW,
            };
            let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
            assert_eq!(ContractError::MarketNotScoreable {}, res);

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
            let value: MarketResponse = from_json(&res).unwrap();
            assert_eq!(Status::ACTIVE, value.market.status);
            assert_eq!(None, value.market.result);

            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                start_timestamp + 60 * 30, // 30 minutes from start timestamp
            );
            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            let msg = ExecuteMsg::Score {
                result: MarketResult::HOME,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
            let value: MarketResponse = from_json(&res).unwrap();
            assert_eq!(Status::CLOSED, value.market.status);
            assert_eq!(MarketResult::HOME, value.market.result.unwrap());
        }

        #[test]
        fn market_has_no_winnings() {
            let mut deps = mock_dependencies();
            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs(),
            );

            let start_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 10; // 10 minutes from now
            let msg = InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                fee_bps: 250,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            };
            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

            let info = message_info(
                &Addr::unchecked(ADMIN_ADDRESS),
                &[coin(1_000, NATIVE_DENOM)],
            );
            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::DRAW,
                receiver: None,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
            let value: BetsResponse = from_json(&res).unwrap();
            assert_eq!(1_000, value.totals.total_draw);
            assert_eq!(0, value.totals.total_away);
            assert_eq!(0, value.totals.total_home);

            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs()
                    + 60 * 45, // 45 minutes in the future
            );

            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            let msg = ExecuteMsg::Score {
                result: MarketResult::DRAW,
            };
            let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
            assert_eq!(ContractError::NoWinnings {}, res);

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
            let value: MarketResponse = from_json(&res).unwrap();
            assert_eq!(Status::ACTIVE, value.market.status);
            assert_eq!(None, value.market.result);
        }

        #[test]
        fn market_has_no_winners() {
            let mut deps = mock_dependencies();
            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs(),
            );

            let start_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 10; // 10 minutes from now
            let msg = InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                fee_bps: 250,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            };
            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

            let info = message_info(
                &Addr::unchecked(ADMIN_ADDRESS),
                &[coin(1_000, NATIVE_DENOM)],
            );
            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::DRAW,
                receiver: None,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let info = message_info(
                &Addr::unchecked(ADMIN_ADDRESS),
                &[coin(1_000, NATIVE_DENOM)],
            );
            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::AWAY,
                receiver: None,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
            let value: BetsResponse = from_json(&res).unwrap();
            assert_eq!(1_000, value.totals.total_draw);
            assert_eq!(1_000, value.totals.total_away);
            assert_eq!(0, value.totals.total_home);

            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs()
                    + 60 * 45, // 45 minutes in the future
            );

            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            let msg = ExecuteMsg::Score {
                result: MarketResult::HOME,
            };
            let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
            assert_eq!(ContractError::NoWinnings {}, res);

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
            let value: MarketResponse = from_json(&res).unwrap();
            assert_eq!(Status::ACTIVE, value.market.status);
            assert_eq!(None, value.market.result);
        }
    }

    mod cancel_market {
        use super::*;

        #[test]
        fn proper_cancel_market() {
            let mut deps = mock_dependencies();
            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs()
                    + 60 * 45, // 45 minutes in the future
            );

            let start_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 10; // 10 minutes from now
            let msg = InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                fee_bps: 250,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            };
            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
            let value: MarketResponse = from_json(&res).unwrap();
            assert_eq!(Status::ACTIVE, value.market.status);

            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            let msg = ExecuteMsg::Cancel {};
            let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
            assert_eq!(0, res.messages.len());

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
            let value: MarketResponse = from_json(&res).unwrap();
            assert_eq!(Status::CANCELLED, value.market.status);
        }

        #[test]
        fn unauthorized() {
            let mut deps = mock_dependencies();
            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs()
                    + 60 * 45, // 45 minutes in the future
            );

            let start_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 10; // 10 minutes from now
            let msg = InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                fee_bps: 250,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            };
            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
            let value: MarketResponse = from_json(&res).unwrap();
            assert_eq!(Status::ACTIVE, value.market.status);

            let anyone = deps.api.addr_make("ANYONE");
            let info = message_info(&anyone, &[]);
            let msg = ExecuteMsg::Cancel {};
            let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
            assert_eq!(ContractError::Unauthorized {}, res);

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
            let value: MarketResponse = from_json(&res).unwrap();
            assert_ne!(Status::CANCELLED, value.market.status);
        }

        #[test]
        fn market_not_active() {
            let mut deps = mock_dependencies();
            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs(),
            );

            let start_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 10; // 10 minutes from now
            let msg = InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                fee_bps: 250,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            };
            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

            let info = message_info(
                &Addr::unchecked(ADMIN_ADDRESS),
                &[coin(1_000, NATIVE_DENOM)],
            );
            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::DRAW,
                receiver: None,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let info = message_info(
                &Addr::unchecked(ADMIN_ADDRESS),
                &[coin(1_000, NATIVE_DENOM)],
            );
            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::AWAY,
                receiver: None,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
            let value: BetsResponse = from_json(&res).unwrap();
            assert_eq!(1_000, value.totals.total_draw);
            assert_eq!(1_000, value.totals.total_away);
            assert_eq!(0, value.totals.total_home);

            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs()
                    + 60 * 45, // 45 minutes in the future
            );

            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            let msg = ExecuteMsg::Score {
                result: MarketResult::DRAW,
            };
            execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
            let value: MarketResponse = from_json(&res).unwrap();
            assert_ne!(Status::ACTIVE, value.market.status);

            let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
            let msg = ExecuteMsg::Cancel {};
            let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
            assert_eq!(ContractError::MarketNotActive {}, res);

            let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
            let value: MarketResponse = from_json(&res).unwrap();
            assert_ne!(Status::CANCELLED, value.market.status);
        }
    }
}
