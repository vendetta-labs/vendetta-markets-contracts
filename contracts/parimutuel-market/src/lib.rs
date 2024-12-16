use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod contract;
pub mod error;
pub mod execute;
pub mod helpers;
#[cfg(test)]
mod integration_tests;
pub mod msg;
pub mod queries;
pub mod state;

pub use crate::error::ContractError;

#[derive(Serialize, Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct TotalBets {
    pub total_home: u128,
    pub total_away: u128,
    pub total_draw: u128,
}

fn calculate_parimutuel_winnings(total_bets: u128, total_team_bets: u128, total_bet: u128) -> u128 {
    if total_bet == 0 || total_team_bets == 0 || total_bets == 0 {
        return 0;
    }

    Uint128::from(total_bets)
        .multiply_ratio(total_bet, total_team_bets)
        .u128()
}
