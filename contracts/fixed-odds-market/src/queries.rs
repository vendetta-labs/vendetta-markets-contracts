use cosmwasm_std::{Addr, Decimal, Deps, StdResult};

use crate::{
    msg::{
        AllBets, BetsByAddressResponse, BetsResponse, ConfigResponse, MarketResponse,
        PotentialPayouts, TotalAmounts,
    },
    state::{
        ADDR_BETS_AWAY, ADDR_BETS_HOME, CONFIG, MARKET, POTENTIAL_PAYOUT_AWAY,
        POTENTIAL_PAYOUT_HOME, TOTAL_BETS_AWAY, TOTAL_BETS_HOME,
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

/// Returns the total bets and potential payouts of the market
pub fn query_bets(deps: Deps) -> StdResult<BetsResponse> {
    let total_amounts = TotalAmounts {
        home: TOTAL_BETS_HOME.load(deps.storage)?,
        away: TOTAL_BETS_AWAY.load(deps.storage)?,
    };

    let potential_payouts = PotentialPayouts {
        home: POTENTIAL_PAYOUT_HOME.load(deps.storage)?,
        away: POTENTIAL_PAYOUT_AWAY.load(deps.storage)?,
    };

    Ok(BetsResponse {
        total_amounts,
        potential_payouts,
    })
}

/// Retruns the average bets and potential payouts for a specific address
pub fn query_bets_by_address(deps: Deps, address: Addr) -> StdResult<BetsByAddressResponse> {
    let all_bets = AllBets {
        home: ADDR_BETS_HOME
            .may_load(deps.storage, address.clone())?
            .unwrap_or((Decimal::zero(), 0)),
        away: ADDR_BETS_AWAY
            .may_load(deps.storage, address.clone())?
            .unwrap_or((Decimal::zero(), 0)),
    };

    Ok(BetsByAddressResponse {
        address,
        all_bets,
        potential_payouts: PotentialPayouts { home: 0, away: 0 },
    })
}
