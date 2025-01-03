#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::{
    calculate_odds,
    execute::{
        execute_cancel, execute_claim_winnings, execute_place_bet, execute_score, execute_update,
    },
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    queries::{query_bets_by_address, query_config, query_estimate_winnings, query_market},
    state::{
        Config, Market, Status, AWAY_TOTAL_BETS, AWAY_TOTAL_PAYOUT, CONFIG, HOME_TOTAL_BETS,
        HOME_TOTAL_PAYOUT, MARKET,
    },
    ContractError,
};

pub const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const ADMIN_ADDRESS: &str = "neutron15yhlj25av4fkw6s8qwnzerp490pkxmn9094g7r";
pub const TREASURY_ADDRESS: &str = "neutron12v9pqx602k3rzm5hf4jewepl8na4x89ja4td24";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    if Addr::unchecked(ADMIN_ADDRESS) != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let initial_fund = info.funds.first().unwrap();
    let market_balance = if initial_fund.denom == msg.denom {
        initial_fund.amount
    } else {
        Uint128::zero()
    };

    if market_balance.is_zero() {
        return Err(ContractError::MarketNotInitiallyFunded {});
    }

    set_contract_version(
        deps.storage,
        format!("crates.io:{CONTRACT_NAME}"),
        CONTRACT_VERSION,
    )?;

    let state = Config {
        admin_addr: Addr::unchecked(ADMIN_ADDRESS),
        treasury_addr: Addr::unchecked(TREASURY_ADDRESS),
        denom: msg.denom,
        fee_spread_odds: msg.fee_spread_odds,
        max_bet_risk_factor: msg.max_bet_risk_factor,
        seed_liquidity_amplifier: msg.seed_liquidity_amplifier,
        initial_home_odds: msg.initial_home_odds,
        initial_away_odds: msg.initial_away_odds,
    };
    CONFIG.save(deps.storage, &state)?;

    HOME_TOTAL_PAYOUT.save(deps.storage, &Uint128::zero())?;
    AWAY_TOTAL_PAYOUT.save(deps.storage, &Uint128::zero())?;
    HOME_TOTAL_BETS.save(deps.storage, &Uint128::zero())?;
    AWAY_TOTAL_BETS.save(deps.storage, &Uint128::zero())?;

    let (home_odds, away_odds) =
        calculate_odds(&state, market_balance, Uint128::zero(), Uint128::zero());

    let market = Market {
        id: msg.id,
        label: msg.label,
        home_team: msg.home_team,
        home_odds,
        away_team: msg.away_team,
        away_odds,
        start_timestamp: msg.start_timestamp,
        status: Status::ACTIVE,
        result: None,
    };
    MARKET.save(deps.storage, &market)?;

    Ok(Response::new()
        .add_attribute("protocol", "vendetta-markets")
        .add_attribute("market_type", "fixed-odds")
        .add_attribute("action", "create_market")
        .add_attribute("sender", info.sender)
        .add_attribute("id", market.id)
        .add_attribute("label", market.label)
        .add_attribute("home_team", market.home_team)
        .add_attribute("home_odds", market.home_odds.to_string())
        .add_attribute("away_team", market.away_team)
        .add_attribute("away_odds", market.away_odds.to_string())
        .add_attribute("start_timestamp", market.start_timestamp.to_string())
        .add_attribute("status", Status::ACTIVE.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Market {} => to_json_binary(&query_market(deps)?),
        QueryMsg::BetsByAddress { address } => {
            to_json_binary(&query_bets_by_address(deps, address)?)
        }
        QueryMsg::EstimateWinnings { address, result } => {
            to_json_binary(&query_estimate_winnings(deps, address, result)?)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::PlaceBet {
            result,
            min_odds,
            receiver,
        } => execute_place_bet(deps, env, info, result, min_odds, receiver),
        ExecuteMsg::ClaimWinnings { receiver } => execute_claim_winnings(deps, info, receiver),
        ExecuteMsg::Update {
            fee_spread_odds,
            max_bet_risk_factor,
            seed_liquidity_amplifier,
            initial_home_odds,
            initial_away_odds,
            start_timestamp,
        } => execute_update(
            deps,
            info,
            fee_spread_odds,
            max_bet_risk_factor,
            seed_liquidity_amplifier,
            initial_home_odds,
            initial_away_odds,
            start_timestamp,
        ),
        ExecuteMsg::Score { result } => execute_score(deps, env, info, result),
        ExecuteMsg::Cancel {} => execute_cancel(deps, info),
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::msg::{ConfigResponse, MarketResponse};

    use super::*;
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
    use cosmwasm_std::{coin, from_json, Decimal};

    const NATIVE_DENOM: &str = "denom";

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 5; // 5 minutes from now
        let msg = InstantiateMsg {
            denom: NATIVE_DENOM.to_string(),
            id: "game-cs2-test-league".to_string(),
            label: "CS2 - Test League - Team A vs Team B".to_string(),
            home_team: "Team A".to_string(),
            away_team: "Team B".to_string(),
            fee_spread_odds: Decimal::from_atomics(15_u128, 2).unwrap(), // 0.15
            max_bet_risk_factor: Decimal::from_atomics(15_u128, 1).unwrap(), // 1.5
            seed_liquidity_amplifier: Decimal::from_atomics(3_u128, 0).unwrap(), // 3
            initial_home_odds: Decimal::from_atomics(22_u128, 1).unwrap(), // 2.1
            initial_away_odds: Decimal::from_atomics(18_u128, 1).unwrap(), // 1.8
            start_timestamp,
        };
        let info = message_info(
            &Addr::unchecked(ADMIN_ADDRESS),
            &[coin(1_000, NATIVE_DENOM)],
        );

        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let config_query = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
        let value: ConfigResponse = from_json(&config_query).unwrap();
        assert_eq!(ADMIN_ADDRESS, value.config.admin_addr.as_str());
        assert_eq!(TREASURY_ADDRESS, value.config.treasury_addr.as_str());
        assert_eq!(NATIVE_DENOM, value.config.denom.as_str());

        let market_query = query(deps.as_ref(), mock_env(), QueryMsg::Market {}).unwrap();
        let value: MarketResponse = from_json(&market_query).unwrap();
        assert_eq!("game-cs2-test-league", value.market.id);
        assert_eq!("CS2 - Test League - Team A vs Team B", value.market.label);
        assert_eq!("Team A", value.market.home_team);
        assert_eq!("Team B", value.market.away_team);
        assert_eq!(start_timestamp, value.market.start_timestamp);
        assert_eq!(Status::ACTIVE, value.market.status);
        assert_eq!(None, value.market.result);
        assert_eq!(
            Decimal::from_atomics(191_u128, 2).unwrap(),
            value.market.home_odds
        );
        assert_eq!(
            Decimal::from_atomics(156_u128, 2).unwrap(),
            value.market.away_odds
        );
        println!("{:?}", value.market);
    }
}
