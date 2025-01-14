use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Invalid chain prefix: {0}")]
    InvalidChainPrefix(String),

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
