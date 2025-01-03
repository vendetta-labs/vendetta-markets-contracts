use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal};

use crate::state::{Config, Market, MarketResult};

#[cw_serde]
pub struct InstantiateMsg {
    pub denom: String,
    pub id: String,
    pub label: String,
    pub home_team: String,
    pub away_team: String,
    pub fee_spread_odds: Decimal,     // Fee spread in percentage points
    pub max_bet_risk_factor: Decimal, // Max bet risk factor in multiplier, ex: 1.5x
    pub seed_liquidity_amplifier: Decimal, // Seed liquidity amplifier in multiplier, ex: 3x
    pub initial_home_odds: Decimal,
    pub initial_away_odds: Decimal,
    pub start_timestamp: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    PlaceBet {
        result: MarketResult,
        min_odds: Decimal,
        receiver: Option<Addr>,
    },
    ClaimWinnings {
        receiver: Option<Addr>,
    },
    // Admin
    Update {
        fee_spread_odds: Option<Decimal>, // Fee spread in percentage points
        max_bet_risk_factor: Option<Decimal>, // Max bet risk factor in multiplier, ex: 1.5x
        seed_liquidity_amplifier: Option<Decimal>, // Seed liquidity amplifier in multiplier, ex: 3x
        initial_home_odds: Option<Decimal>,
        initial_away_odds: Option<Decimal>,
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
