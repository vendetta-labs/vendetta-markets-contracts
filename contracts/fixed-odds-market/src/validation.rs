use cosmwasm_std::Decimal;

use crate::error::ContractError;

pub fn validate_odd(odd: Decimal) -> Result<(), ContractError> {
    if odd < Decimal::one() {
        return Err(ContractError::InvalidOdd(odd));
    }

    Ok(())
}

pub fn validate_fee_spread_odds(fee_spread_odds: Decimal) -> Result<(), ContractError> {
    if fee_spread_odds < Decimal::zero()
        || fee_spread_odds > Decimal::from_atomics(25_u128, 2).unwrap()
    {
        return Err(ContractError::InvalidFeeSpreadOdds(fee_spread_odds));
    }

    Ok(())
}

pub fn validate_max_bet_risk_factor(max_bet_risk_factor: Decimal) -> Result<(), ContractError> {
    if max_bet_risk_factor < Decimal::one()
        || max_bet_risk_factor > Decimal::from_atomics(10_u128, 0).unwrap()
    {
        return Err(ContractError::InvalidMaxBetRiskFactor(max_bet_risk_factor));
    }

    Ok(())
}

pub fn validate_seed_liquidity_amplifier(
    seed_liquidity_amplifier: Decimal,
) -> Result<(), ContractError> {
    if seed_liquidity_amplifier < Decimal::one()
        || seed_liquidity_amplifier > Decimal::from_atomics(10_u128, 0).unwrap()
    {
        return Err(ContractError::InvalidSeedLiquidityAmplifier(
            seed_liquidity_amplifier,
        ));
    }

    Ok(())
}
