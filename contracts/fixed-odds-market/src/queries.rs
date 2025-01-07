use cosmwasm_std::{Addr, Deps, StdResult};

use crate::{
    msg::{
        BetsByAddressResponse, BetsResponse, ConfigResponse, EstimateWinningsResponse,
        MarketResponse, PotentialPayouts, TotalBets,
    },
    state::{
        MarketResult, CONFIG, MARKET, POTENTIAL_PAYOUT_AWAY, POTENTIAL_PAYOUT_HOME,
        TOTAL_BETS_AWAY, TOTAL_BETS_HOME,
    },
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
pub fn query_bets(deps: Deps) -> StdResult<BetsResponse> {
    let totals = TotalBets {
        home: TOTAL_BETS_HOME.load(deps.storage)?,
        away: TOTAL_BETS_AWAY.load(deps.storage)?,
    };

    let potential_payouts = PotentialPayouts {
        home: POTENTIAL_PAYOUT_HOME.load(deps.storage)?,
        away: POTENTIAL_PAYOUT_AWAY.load(deps.storage)?,
    };

    Ok(BetsResponse {
        totals,
        potential_payouts,
    })
}

/// Retruns the total bets for a specific address
pub fn query_bets_by_address(_deps: Deps, address: Addr) -> StdResult<BetsByAddressResponse> {
    Ok(BetsByAddressResponse {
        address,
        totals: TotalBets { home: 0, away: 0 },
        potential_payouts: PotentialPayouts { home: 0, away: 0 },
    })
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
