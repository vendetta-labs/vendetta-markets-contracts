#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::{
    error::ContractError,
    execute::{
        execute_cancel, execute_claim_winnings, execute_place_bet, execute_score, execute_update,
    },
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, UpdateParams},
    queries::{
        query_bets, query_bets_by_address, query_config, query_estimate_winnings, query_market,
    },
    state::{Config, Market, Status, CONFIG, MARKET, TOTAL_AWAY, TOTAL_DRAW, TOTAL_HOME},
    validation::validate_fee_bps,
};

pub const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    validate_fee_bps(msg.fee_bps)?;

    set_contract_version(
        deps.storage,
        format!("crates.io:{CONTRACT_NAME}"),
        CONTRACT_VERSION,
    )?;

    let state = Config {
        admin_addr: msg.admin_addr.clone(),
        treasury_addr: msg.treasury_addr.clone(),
        fee_bps: msg.fee_bps,
        denom: msg.denom.clone(),
        denom_precision: msg.denom_precision,
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
        .add_attribute("admin_addr", msg.admin_addr)
        .add_attribute("treasury_addr", msg.treasury_addr)
        .add_attribute("denom", msg.denom)
        .add_attribute("denom_precision", msg.denom_precision.to_string())
        .add_attribute("fee_bps", msg.fee_bps.to_string())
        .add_attribute("id", market.id)
        .add_attribute("label", market.label)
        .add_attribute("home_team", market.home_team)
        .add_attribute("away_team", market.away_team)
        .add_attribute("start_timestamp", market.start_timestamp.to_string())
        .add_attribute("is_drawable", msg.is_drawable.to_string())
        .add_attribute("status", Status::ACTIVE.to_string()))
}

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
        ExecuteMsg::Update {
            admin_addr,
            treasury_addr,
            start_timestamp,
            fee_bps,
        } => execute_update(
            deps,
            info,
            UpdateParams {
                admin_addr,
                treasury_addr,
                start_timestamp,
                fee_bps,
            },
        ),
        ExecuteMsg::Score { result } => execute_score(deps, env, info, result),
        ExecuteMsg::Cancel {} => execute_cancel(deps, info),
    }
}
