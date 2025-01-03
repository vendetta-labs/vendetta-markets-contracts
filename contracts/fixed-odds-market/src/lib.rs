pub mod contract;
pub mod error;
pub mod execute;
pub mod helpers;
#[cfg(test)]
mod integration_tests;
pub mod msg;
pub mod queries;
pub mod state;

use cosmwasm_std::{Decimal, Uint128};
use state::Config;

pub use crate::error::ContractError;

/// Calculates the new odds for a market
///
/// The new odds are calculated based on the following formula:
///
/// ```ignore
/// market_seed_balance = market_balance - home_total_bets - away_total_bets
///
/// initial_home_probability = 1 / initial_home_odds
/// initial_away_probability = 1 / initial_away_odds
///
/// derived_home_probability = home_total_bets / (home_total_bets + away_total_bets) || 0
/// derived_away_probability = away_total_bets / (home_total_bets + away_total_bets) || 0
///
/// market_probabilities_weight = (home_total_bets + away_total_bets) / (home_total_bets + away_total_bets + market_seed_balance * seed_amplifier)
///
/// new_home_probability = ((derived_home_probability * market_probabilities_weight) + (initial_home_probability * (1 - market_probabilities_weight))) * (1 + fee_spread_odds)
/// new_away_probability = ((derived_away_probability * market_probabilities_weight) + (initial_away_probability * (1 - market_probabilities_weight))) * (1 + fee_spread_odds)
///
/// new_home_odds = 1 / new_home_probability
/// new_away_odds = 1 / new_away_probability
/// ```
///
/// Returns the new home and away odds as a tuple `(new_home_odds, new_away_odds)`
pub fn calculate_odds(
    config: &Config,
    market_balance: Uint128,
    home_total_bets: Uint128,
    away_total_bets: Uint128,
) -> (Decimal, Decimal) {
    let market_seed_balance = market_balance - home_total_bets - away_total_bets;
    let initial_home_probability = Decimal::one() / config.initial_home_odds;
    let initial_away_probability = Decimal::one() / config.initial_away_odds;

    let total_bets = Decimal::from_atomics(home_total_bets, 6).unwrap()
        + Decimal::from_atomics(away_total_bets, 6).unwrap();

    let derived_home_probability = if total_bets != Decimal::zero() {
        Decimal::from_atomics(home_total_bets, 6).unwrap() / total_bets
    } else {
        Decimal::zero()
    };

    let derived_away_probability = if total_bets != Decimal::zero() {
        Decimal::from_atomics(away_total_bets, 6).unwrap() / total_bets
    } else {
        Decimal::zero()
    };

    let market_probabilities_weight = total_bets
        / (total_bets
            + Decimal::from_atomics(market_seed_balance, 6).unwrap()
                * config.seed_liquidity_amplifier);

    let new_home_probability = ((derived_home_probability * market_probabilities_weight)
        + (initial_home_probability * (Decimal::one() - market_probabilities_weight)))
        * (Decimal::one() + config.fee_spread_odds);

    let new_away_probability = ((derived_away_probability * market_probabilities_weight)
        + (initial_away_probability * (Decimal::one() - market_probabilities_weight)))
        * (Decimal::one() + config.fee_spread_odds);

    (
        truncate_odds(Decimal::one() / new_home_probability, 2),
        truncate_odds(Decimal::one() / new_away_probability, 2),
    )
}

/// Truncates the decimal places of an odds
///
/// The function takes a decimal and truncates the decimal places to the specified number of decimals.
///
/// For example, if the decimal is 1.2345 and the decimals is 2, the function will return 1.23.
fn truncate_odds(odds: Decimal, decimals: u32) -> Decimal {
    let mut atomics = odds.atomics();

    let decimal_places = odds.decimal_places() as i32;
    let decimal_places_difference = decimal_places - 2;

    if decimal_places_difference > 0 {
        atomics = atomics / Uint128::from(10_u128.pow(decimal_places_difference as u32));
    } else if decimal_places_difference < 0 {
        atomics = atomics * Uint128::from(10_u128.pow((decimal_places_difference * -1) as u32));
    }

    Decimal::from_atomics(atomics, decimals).unwrap()
}
