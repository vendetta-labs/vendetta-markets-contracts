use cosmwasm_std::{Decimal, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Invalid chain prefix: {0}")]
    InvalidChainPrefix(String),

    #[error("Invalid odd: {0}")]
    InvalidOdd(Decimal),

    #[error("Invalid odds combination")]
    InvalidOddsCombination,

    #[error("Invalid fee spread odds: {0}")]
    InvalidFeeSpreadOdds(Decimal),

    #[error("Invalid max bet risk factor: {0}")]
    InvalidMaxBetRiskFactor(Decimal),

    #[error("Invalid seed liquidity amplifier: {0}")]
    InvalidSeedLiquidityAmplifier(Decimal),

    #[error("Market not initially funded")]
    MarketNotInitiallyFunded {},

    #[error("Market not active")]
    MarketNotActive {},

    #[error("Market not closed")]
    MarketNotClosed {},

    #[error("Market not scoreable")]
    MarketNotScoreable {},

    #[error("Bets no longer accepted")]
    BetsNotAccepted {},

    #[error("Payment error")]
    PaymentError {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Claim already made")]
    ClaimAlreadyMade {},

    #[error("No winnings")]
    NoWinnings {},

    #[error("Minimum odds not kept")]
    MinimumOddsNotKept {},

    #[error("Max bet exceeded")]
    MaxBetExceeded {},
}
