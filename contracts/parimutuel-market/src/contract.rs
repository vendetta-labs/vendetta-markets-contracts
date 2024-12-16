#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use crate::{
    error::ContractError,
    execute::{
        execute_cancel, execute_claim_winnings, execute_place_bet, execute_score, execute_update,
    },
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    queries::{
        query_bets, query_bets_by_address, query_config, query_estimate_winnings, query_market,
    },
    state::{Config, Market, Status, CONFIG, MARKET, TOTAL_AWAY, TOTAL_DRAW, TOTAL_HOME},
};

pub const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const ADMIN_ADDRESS: &str = "neutron15yhlj25av4fkw6s8qwnzerp490pkxmn9094g7r";
pub const TREASURY_ADDRESS: &str = "neutron12v9pqx602k3rzm5hf4jewepl8na4x89ja4td24";

// INSTANTIATE

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

    set_contract_version(
        deps.storage,
        format!("crates.io:{CONTRACT_NAME}"),
        CONTRACT_VERSION,
    )?;

    let state = Config {
        admin_addr: Addr::unchecked(ADMIN_ADDRESS),
        treasury_addr: Addr::unchecked(TREASURY_ADDRESS),
        fee_bps: msg.fee_bps,
        denom: msg.denom,
    };
    CONFIG.save(deps.storage, &state)?;

    let market = Market {
        id: msg.id,
        label: msg.label,
        home_team: msg.home_team,
        away_team: msg.away_team,
        start_timestamp: msg.start_timestamp,
        status: Status::ACTIVE,
        result: None,
        is_drawable: msg.is_drawable,
    };
    MARKET.save(deps.storage, &market)?;

    TOTAL_HOME.save(deps.storage, &0)?;
    TOTAL_AWAY.save(deps.storage, &0)?;
    TOTAL_DRAW.save(deps.storage, &0)?;

    Ok(Response::new()
        .add_attribute("protocol", "vendetta-markets")
        .add_attribute("market_type", "parimutuel")
        .add_attribute("action", "create_market")
        .add_attribute("sender", info.sender)
        .add_attribute("id", market.id)
        .add_attribute("label", market.label)
        .add_attribute("home_team", market.home_team)
        .add_attribute("away_team", market.away_team)
        .add_attribute("start_timestamp", market.start_timestamp.to_string())
        .add_attribute("is_drawable", msg.is_drawable.to_string())
        .add_attribute("status", Status::ACTIVE.to_string()))
}

// QUERY

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Market {} => to_json_binary(&query_market(deps)?),
        QueryMsg::Bets {} => to_json_binary(&query_bets(deps)?),
        QueryMsg::BetsByAddress { address } => {
            to_json_binary(&query_bets_by_address(deps, address)?)
        }
        QueryMsg::EstimateWinnings { address, result } => {
            to_json_binary(&query_estimate_winnings(deps, address, result)?)
        }
    }
}

// EXECUTE

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::PlaceBet { result, receiver } => {
            execute_place_bet(deps, env, info, result, receiver)
        }
        ExecuteMsg::ClaimWinnings { receiver } => execute_claim_winnings(deps, info, receiver),
        ExecuteMsg::Update { start_timestamp } => execute_update(deps, info, start_timestamp),
        ExecuteMsg::Score { result } => execute_score(deps, env, info, result),
        ExecuteMsg::Cancel {} => execute_cancel(deps, info),
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::msg::{ConfigResponse, MarketResponse};

    use super::*;
    use cosmwasm_std::from_json;
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};

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
            fee_bps: 250,
            id: "game-cs2-test-league".to_string(),
            label: "CS2 - Test League - Team A vs Team B".to_string(),
            home_team: "Team A".to_string(),
            away_team: "Team B".to_string(),
            start_timestamp,
            is_drawable: true,
        };
        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);

        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let config_query = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
        let value: ConfigResponse = from_json(&config_query).unwrap();
        assert_eq!(250, value.config.fee_bps);
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
        assert_eq!(true, value.market.is_drawable);
        assert_eq!(Status::ACTIVE, value.market.status);
        assert_eq!(None, value.market.result);
    }

    // #[test]
    // fn increment() {
    //     let mut deps = mock_dependencies();

    //     let msg = InstantiateMsg {
    //         denom: NATIVE_DENOM.to_string(),
    //         fee_bps: 250,
    //         id: "game-cs2-test-league".to_string(),
    //         label: "CS2 - Test League - Team A vs Team B".to_string(),
    //         home_team: "Team A".to_string(),
    //         away_team: "Team B".to_string(),
    //         start_timestamp: SystemTime::now()
    //             .duration_since(UNIX_EPOCH)
    //             .expect("Time went backwards")
    //             .as_secs()
    //             + 60 * 5, // 5 minutes from now
    //         is_drawable: true,
    //     };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // beneficiary can release it
    //     let info = mock_info("anyone", &coins(2, "token"));
    //     let msg = ExecuteMsg::Increment {};
    //     let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // should increase counter by 1
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: GetCountResponse = from_json(&res).unwrap();
    //     assert_eq!(18, value.count);
    // }
}
