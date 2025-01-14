use cosmwasm_std::{Addr, Decimal, Deps, Env, StdResult, Uint128};

use crate::{
    logic::calculate_max_bet,
    msg::{
        AllBets, BetRecordWithOdds, BetsByAddressResponse, BetsResponse, ConfigResponse,
        MarketResponse, MaxBetsResponse, PotentialPayouts, TotalAmounts,
    },
    state::{
        Status, ADDR_BETS_AWAY, ADDR_BETS_HOME, CONFIG, MARKET, POTENTIAL_PAYOUT_AWAY,
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

/// Returns the market max bet for each result
pub fn query_max_bets(deps: Deps, env: Env) -> StdResult<MaxBetsResponse> {
    let config = CONFIG.load(deps.storage)?;
    let market = MARKET.load(deps.storage)?;

    if market.status != Status::ACTIVE {
        return Ok(MaxBetsResponse { home: 0, away: 0 });
    }

    let market_balance = deps
        .querier
        .query_balance(env.contract.address, &config.denom)?
        .amount;

    let potential_payout_home = POTENTIAL_PAYOUT_HOME.load(deps.storage)?;
    let potential_payout_away = POTENTIAL_PAYOUT_AWAY.load(deps.storage)?;

    let home_max_bet = calculate_max_bet(
        &config,
        market_balance,
        Uint128::from(potential_payout_home),
        market.home_odds,
    );
    let away_max_bet = calculate_max_bet(
        &config,
        market_balance,
        Uint128::from(potential_payout_away),
        market.away_odds,
    );

    Ok(MaxBetsResponse {
        home: home_max_bet.into(),
        away: away_max_bet.into(),
    })
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
    let config = CONFIG.load(deps.storage)?;

    let (home_bet_amount, home_payout) = ADDR_BETS_HOME
        .may_load(deps.storage, address.clone())?
        .unwrap_or((0, 0));
    let home_odds = if home_bet_amount.gt(&0_u128) {
        Decimal::from_atomics(home_payout, config.denom_precision).unwrap()
            / Decimal::from_atomics(home_bet_amount, config.denom_precision).unwrap()
    } else {
        Decimal::zero()
    };
    let (away_bet_amount, away_payout) = ADDR_BETS_AWAY
        .may_load(deps.storage, address.clone())?
        .unwrap_or((0, 0));
    let away_odds = if away_bet_amount.gt(&0_u128) {
        Decimal::from_atomics(away_payout, config.denom_precision).unwrap()
            / Decimal::from_atomics(away_bet_amount, config.denom_precision).unwrap()
    } else {
        Decimal::zero()
    };

    let all_bets = AllBets {
        home: BetRecordWithOdds {
            bet_amount: home_bet_amount,
            payout: home_payout,
            odds: home_odds,
        },
        away: BetRecordWithOdds {
            bet_amount: away_bet_amount,
            payout: away_payout,
            odds: away_odds,
        },
    };

    Ok(BetsByAddressResponse { address, all_bets })
}
