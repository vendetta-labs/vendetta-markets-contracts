use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

use crate::state::{Config, Market, MarketResult, Odd};

#[cw_serde]
pub struct InstantiateMsg {
    pub fee_bps: u64,       // Fee in basis points
    pub max_bet_ratio: u64, // Max bet ratio in basis points
    pub denom: String,
    pub id: String,
    pub label: String,
    pub home_team: String,
    pub home_odds: Odd,
    pub away_team: String,
    pub away_odds: Odd,
    pub start_timestamp: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    PlaceBet {
        result: MarketResult,
        min_odds: Odd,
        receiver: Option<Addr>,
    },
    ClaimWinnings {
        receiver: Option<Addr>,
    },
    // Admin
    Update {
        max_bet_ratio: Option<u64>,
        home_odds: Option<Odd>,
        away_odds: Option<Odd>,
        start_timestamp: Option<u64>,
    },
    Score {
        result: MarketResult,
    },
    Cancel {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(MarketResponse)]
    Market {},
    #[returns(BetsByAddressResponse)]
    BetsByAddress { address: Addr },
    #[returns(EstimateWinningsResponse)]
    EstimateWinnings { address: Addr, result: MarketResult },
}

#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
}

#[cw_serde]
pub struct MarketResponse {
    pub market: Market,
}

#[cw_serde]
pub struct BetsByAddressResponse {
    pub address: Addr,
}

#[cw_serde]
pub struct EstimateWinningsResponse {
    pub estimate: u128,
}
