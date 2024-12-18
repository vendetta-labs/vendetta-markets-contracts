use cosmwasm_std::{Addr, Deps, StdResult};

use crate::{
    msg::{BetsByAddressResponse, ConfigResponse, EstimateWinningsResponse, MarketResponse},
    state::{MarketResult, CONFIG, MARKET},
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

/// Retruns the total bets for a specific address
pub fn query_bets_by_address(_deps: Deps, address: Addr) -> StdResult<BetsByAddressResponse> {
    Ok(BetsByAddressResponse { address })
}

/// Returns the estimated winnings for a specific address and result
pub fn query_estimate_winnings(
    _deps: Deps,
    _address: Addr,
    _result: MarketResult,
) -> StdResult<EstimateWinningsResponse> {
    let estimate = 0;

    Ok(EstimateWinningsResponse { estimate })
}
