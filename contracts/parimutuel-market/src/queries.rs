use cosmwasm_std::{Addr, Deps, StdResult};

use crate::{
    calculate_parimutuel_winnings,
    msg::{
        BetsByAddressResponse, BetsResponse, ConfigResponse, EstimateWinningsResponse,
        MarketResponse,
    },
    state::{
        MarketResult, CONFIG, MARKET, POOL_AWAY, POOL_DRAW, POOL_HOME, TOTAL_AWAY, TOTAL_DRAW,
        TOTAL_HOME,
    },
    TotalBets,
};

/// Returns the current config of the market
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { config })
}

/// Returns the current state and data of the market
pub fn query_market(deps: Deps) -> StdResult<MarketResponse> {
    let market = MARKET.load(deps.storage)?;
    Ok(MarketResponse { market })
}

/// Returns the total bets of the market
///
/// This includes the total bet amounts in each pool:
/// TOTAL_HOME, TOTAL_AWAY, TOTAL_DRAW
pub fn query_bets(deps: Deps) -> StdResult<BetsResponse> {
    let totals = TotalBets {
        total_home: TOTAL_HOME.load(deps.storage)?,
        total_away: TOTAL_AWAY.load(deps.storage)?,
        total_draw: TOTAL_DRAW.load(deps.storage)?,
    };
    Ok(BetsResponse { totals })
}

/// Retruns the total bets for a specific address
///
/// This includes the total bet amounts in each pool:
/// TOTAL_HOME, TOTAL_AWAY, TOTAL_DRAW
pub fn query_bets_by_address(deps: Deps, address: Addr) -> StdResult<BetsByAddressResponse> {
    let total_home = if POOL_HOME.has(deps.storage, address.clone()) {
        POOL_HOME.load(deps.storage, address.clone())?
    } else {
        0
    };

    let total_away = if POOL_AWAY.has(deps.storage, address.clone()) {
        POOL_AWAY.load(deps.storage, address.clone())?
    } else {
        0
    };

    let total_draw = if POOL_DRAW.has(deps.storage, address.clone()) {
        POOL_DRAW.load(deps.storage, address.clone())?
    } else {
        0
    };

    let totals = TotalBets {
        total_home,
        total_away,
        total_draw,
    };

    Ok(BetsByAddressResponse { address, totals })
}

/// Returns the estimated winnings for a specific address and result
///
/// Based on the total bets of the opposing pools and the bet amount of the address on the result pool.
/// The result is calculated as follows:
///
/// `winnings = opposing_total_bets * result_address_bet_amount / result_total_bets`
///
/// This does not take into account the fees.
pub fn query_estimate_winnings(
    deps: Deps,
    address: Addr,
    result: MarketResult,
) -> StdResult<EstimateWinningsResponse> {
    let total_home = TOTAL_HOME.load(deps.storage)?;
    let total_away = TOTAL_AWAY.load(deps.storage)?;
    let total_draw = TOTAL_DRAW.load(deps.storage)?;

    let addr_pool_home = if POOL_HOME.has(deps.storage, address.clone()) {
        POOL_HOME.load(deps.storage, address.clone())?
    } else {
        0
    };

    let addr_pool_away = if POOL_AWAY.has(deps.storage, address.clone()) {
        POOL_AWAY.load(deps.storage, address.clone())?
    } else {
        0
    };

    let addr_pool_draw = if POOL_DRAW.has(deps.storage, address.clone()) {
        POOL_DRAW.load(deps.storage, address)?
    } else {
        0
    };

    let addr_bets = match result {
        MarketResult::HOME => addr_pool_home,
        MarketResult::AWAY => addr_pool_away,
        MarketResult::DRAW => addr_pool_draw,
    };

    let team_bets = match result {
        MarketResult::HOME => total_home,
        MarketResult::AWAY => total_away,
        MarketResult::DRAW => total_draw,
    };

    let estimate =
        calculate_parimutuel_winnings(total_home + total_away + total_draw, team_bets, addr_bets);

    Ok(EstimateWinningsResponse { estimate })
}
