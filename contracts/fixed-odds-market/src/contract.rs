#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::{
    error::ContractError,
    execute::{
        execute_cancel, execute_claim_winnings, execute_place_bet, execute_score, execute_update,
    },
    logic::{calculate_max_bet, calculate_odds},
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, UpdateParams},
    queries::{query_bets, query_bets_by_address, query_config, query_market, query_max_bets},
    state::{
        Config, Market, Status, CONFIG, MARKET, POTENTIAL_PAYOUT_AWAY, POTENTIAL_PAYOUT_HOME,
        TOTAL_BETS_AWAY, TOTAL_BETS_HOME,
    },
};

pub const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const ADMIN_ADDRESS: &str = "neutron15yhlj25av4fkw6s8qwnzerp490pkxmn9094g7r";
pub const TREASURY_ADDRESS: &str = "neutron12v9pqx602k3rzm5hf4jewepl8na4x89ja4td24";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    if Addr::unchecked(ADMIN_ADDRESS) != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let market_balance = deps
        .querier
        .query_balance(&env.contract.address, &msg.denom)?
        .amount;

    if market_balance.is_zero() {
        return Err(ContractError::MarketNotInitiallyFunded {});
    }

    set_contract_version(
        deps.storage,
        format!("crates.io:{CONTRACT_NAME}"),
        CONTRACT_VERSION,
    )?;

    let config = Config {
        admin_addr: Addr::unchecked(ADMIN_ADDRESS),
        treasury_addr: Addr::unchecked(TREASURY_ADDRESS),
        denom: msg.denom.clone(),
        denom_precision: msg.denom_precision,
        fee_spread_odds: msg.fee_spread_odds,
        max_bet_risk_factor: msg.max_bet_risk_factor,
        seed_liquidity_amplifier: msg.seed_liquidity_amplifier,
        initial_odds_home: msg.initial_odds_home,
        initial_odds_away: msg.initial_odds_away,
    };
    CONFIG.save(deps.storage, &config)?;

    TOTAL_BETS_HOME.save(deps.storage, &0)?;
    POTENTIAL_PAYOUT_HOME.save(deps.storage, &0)?;
    TOTAL_BETS_AWAY.save(deps.storage, &0)?;
    POTENTIAL_PAYOUT_AWAY.save(deps.storage, &0)?;

    let (home_odds, away_odds) =
        calculate_odds(&config, market_balance, Uint128::zero(), Uint128::zero());

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

    let home_max_bet =
        calculate_max_bet(&config, market_balance, Uint128::zero(), market.home_odds);
    let away_max_bet =
        calculate_max_bet(&config, market_balance, Uint128::zero(), market.away_odds);

    Ok(Response::new()
        .add_attribute("protocol", "vendetta-markets")
        .add_attribute("market_type", "fixed-odds")
        .add_attribute("action", "create_market")
        .add_attribute("sender", info.sender)
        .add_attribute("admin_addr", ADMIN_ADDRESS)
        .add_attribute("treasury_addr", TREASURY_ADDRESS)
        .add_attribute("denom", msg.denom)
        .add_attribute("fee_spread_odds", msg.fee_spread_odds.to_string())
        .add_attribute("max_bet_risk_factor", msg.max_bet_risk_factor.to_string())
        .add_attribute(
            "seed_liquidity_amplifier",
            msg.seed_liquidity_amplifier.to_string(),
        )
        .add_attribute("initial_odds_home", msg.initial_odds_home.to_string())
        .add_attribute("initial_odds_away", msg.initial_odds_away.to_string())
        .add_attribute("id", market.id)
        .add_attribute("label", market.label)
        .add_attribute("home_team", market.home_team)
        .add_attribute("home_odds", market.home_odds.to_string())
        .add_attribute("home_max_bet", home_max_bet.to_string())
        .add_attribute("away_team", market.away_team)
        .add_attribute("away_odds", market.away_odds.to_string())
        .add_attribute("away_max_bet", away_max_bet.to_string())
        .add_attribute("start_timestamp", market.start_timestamp.to_string())
        .add_attribute("status", Status::ACTIVE.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Market {} => to_json_binary(&query_market(deps)?),
        QueryMsg::MaxBets {} => to_json_binary(&query_max_bets(deps, env)?),
        QueryMsg::Bets {} => to_json_binary(&query_bets(deps)?),
        QueryMsg::BetsByAddress { address } => {
            to_json_binary(&query_bets_by_address(deps, address)?)
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
            initial_odds_home,
            initial_odds_away,
            start_timestamp,
        } => execute_update(
            deps,
            env,
            info,
            UpdateParams {
                fee_spread_odds,
                max_bet_risk_factor,
                seed_liquidity_amplifier,
                initial_odds_home,
                initial_odds_away,
                start_timestamp,
            },
        ),
        ExecuteMsg::Score { result } => execute_score(deps, env, info, result),
        ExecuteMsg::Cancel {} => execute_cancel(deps, env, info),
    }
}
