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

    #[error("Invalid fee bps: {0}")]
    InvalidFeeBps(u64),

    #[error("Market not active")]
    MarketNotActive {},

    #[error("Market not closed")]
    MarketNotClosed {},

    #[error("Market not drawable")]
    MarketNotDrawable {},

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
}
