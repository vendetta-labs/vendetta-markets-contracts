use cosmwasm_std::{Decimal, Uint128};
use std::cmp::Ordering;

use crate::state::Config;

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
    let initial_home_probability = Decimal::one() / config.initial_odds_home;
    let initial_away_probability = Decimal::one() / config.initial_odds_away;

    let total_bets = Decimal::from_atomics(home_total_bets, config.denom_precision).unwrap()
        + Decimal::from_atomics(away_total_bets, config.denom_precision).unwrap();

    let derived_home_probability = if total_bets != Decimal::zero() {
        Decimal::from_atomics(home_total_bets, config.denom_precision).unwrap() / total_bets
    } else {
        Decimal::zero()
    };

    let derived_away_probability = if total_bets != Decimal::zero() {
        Decimal::from_atomics(away_total_bets, config.denom_precision).unwrap() / total_bets
    } else {
        Decimal::zero()
    };

    let market_probabilities_weight = total_bets
        / (total_bets
            + Decimal::from_atomics(market_seed_balance, config.denom_precision).unwrap()
                * config.seed_liquidity_amplifier);

    let new_home_probability = ((derived_home_probability * market_probabilities_weight)
        + (initial_home_probability * (Decimal::one() - market_probabilities_weight)))
        * (Decimal::one() + config.fee_spread_odds);

    let new_away_probability = ((derived_away_probability * market_probabilities_weight)
        + (initial_away_probability * (Decimal::one() - market_probabilities_weight)))
        * (Decimal::one() + config.fee_spread_odds);

    (
        truncate_decimal(Decimal::one() / new_home_probability, 2),
        truncate_decimal(Decimal::one() / new_away_probability, 2),
    )
}

/// Truncates the decimal places
///
/// The function takes a decimal and truncates the decimal places to the specified number of decimals.
///
/// For example, if the decimal is 1.2345 and the decimals is 2, the function will return 1.23.
fn truncate_decimal(decimal: Decimal, decimals: u32) -> Decimal {
    let mut atomics = decimal.atomics();

    let decimal_places = decimal.decimal_places() as i32;
    let decimal_places_difference: i32 = decimal_places - decimals as i32;

    match decimal_places_difference.cmp(&0) {
        Ordering::Greater => {
            atomics /= Uint128::from(10_u128.pow(decimal_places_difference as u32));
        }
        Ordering::Less => {
            atomics *= Uint128::from(10_u128.pow((-decimal_places_difference) as u32));
        }
        Ordering::Equal => {}
    }

    Decimal::from_atomics(atomics, decimals).unwrap()
}

/// Calculates the maximum bet amount for a market
/// based on the following formula:
///
/// ```ignore
/// max_available_payout = market_balance - total_payout
///
/// max_bet_amount = max_available_payout / odds / max_bet_risk_factor
/// ```
///
/// The function returns the maximum bet amount as a Uint128.
pub fn calculate_max_bet(
    config: &Config,
    market_balance: Uint128,
    total_payout: Uint128,
    odds: Decimal,
) -> Uint128 {
    println!(
        "market_balance: {:?}",
        Decimal::from_atomics(market_balance, config.denom_precision).unwrap()
    );
    println!(
        "total_payout: {:?}",
        Decimal::from_atomics(total_payout, config.denom_precision).unwrap()
    );
    let max_available_payout = Decimal::from_atomics(market_balance, config.denom_precision)
        .unwrap()
        - Decimal::from_atomics(total_payout, config.denom_precision).unwrap();

    let max_bet_amount = max_available_payout / odds / config.max_bet_risk_factor;

    convert_from_decimal_to_uint128(max_bet_amount, config.denom_precision)
}

/// Truncates the decimal places and converts it to a Uint128
///
/// The function takes a decimal and truncates the decimal places to the specified number of decimals,
/// then converts it to a Uint128.
///
/// For example, if the decimal is 1.2345 and the decimals is 2, the function will return 123.
fn convert_from_decimal_to_uint128(decimal: Decimal, decimals: u32) -> Uint128 {
    let mut atomics = decimal.atomics();

    let decimal_places = decimal.decimal_places() as i32;
    let decimal_places_difference: i32 = decimal_places - decimals as i32;

    match decimal_places_difference.cmp(&0) {
        Ordering::Greater => {
            atomics /= Uint128::from(10_u128.pow(decimal_places_difference as u32));
        }
        Ordering::Less => {
            atomics *= Uint128::from(10_u128.pow((-decimal_places_difference) as u32));
        }
        Ordering::Equal => {}
    }

    atomics
}

#[cfg(test)]
mod tests {
    use crate::logic::{convert_from_decimal_to_uint128, truncate_decimal};
    use cosmwasm_std::Decimal;
    use cosmwasm_std::Uint128;

    mod truncate_decimal {
        use super::*;

        #[test]
        fn it_truncates_decimal() {
            let decimal = Decimal::from_atomics(123_456789_u128, 6).unwrap();
            let truncated_decimal = truncate_decimal(decimal, 2);
            assert_eq!(
                Decimal::from_atomics(12_345_u128, 2).unwrap(),
                truncated_decimal
            );

            let decimal = Decimal::from_atomics(1_2_u128, 1).unwrap();
            let truncated_decimal = truncate_decimal(decimal, 2);
            assert_eq!(
                Decimal::from_atomics(1_20_u128, 2).unwrap(),
                truncated_decimal
            );

            let decimal = Decimal::from_atomics(2_3129321362139427_u128, 16).unwrap();
            let truncated_decimal = truncate_decimal(decimal, 2);
            assert_eq!(
                Decimal::from_atomics(2_31_u128, 2).unwrap(),
                truncated_decimal
            );
        }
    }

    mod convert_from_decimal_to_uint128 {
        use super::*;

        #[test]
        fn it_converts_decimal_to_uint128() {
            let decimal = Decimal::from_atomics(123_456789_u128, 6).unwrap();
            let uint128 = convert_from_decimal_to_uint128(decimal, 6);
            assert_eq!(Uint128::from(123_456789_u128), uint128);

            let decimal = Decimal::from_atomics(1_2_u128, 1).unwrap();
            let uint128 = convert_from_decimal_to_uint128(decimal, 1);
            assert_eq!(Uint128::from(1_2_u128), uint128);

            let decimal = Decimal::from_atomics(2_3129321362139427_u128, 16).unwrap();
            let uint128 = convert_from_decimal_to_uint128(decimal, 16);
            assert_eq!(Uint128::from(2_3129321362139427_u128), uint128);
        }
    }
}
