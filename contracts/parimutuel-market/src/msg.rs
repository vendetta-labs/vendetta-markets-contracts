use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

use crate::{
    state::{Config, Market, MarketResult},
    TotalBets,
};
#[cw_serde]
pub struct InstantiateMsg {
    pub fee_bps: u64, // Fee in basis points
    pub denom: String,
    pub id: String,
    pub label: String,
    pub home_team: String,
    pub away_team: String,
    pub start_timestamp: u64,
    pub is_drawable: bool,
}
#[cw_serde]
pub enum ExecuteMsg {
    PlaceBet {
        result: MarketResult,
        receiver: Option<Addr>,
    },
    ClaimWinnings {
        receiver: Option<Addr>,
    },
    // Admin
    Update {
        start_timestamp: u64,
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
    #[returns(BetsResponse)]
    Bets {},
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
pub struct BetsResponse {
    pub totals: TotalBets,
}

#[cw_serde]
pub struct BetsByAddressResponse {
    pub address: Addr,
    pub totals: TotalBets,
}

#[cw_serde]
pub struct EstimateWinningsResponse {
    pub estimate: u128,
}
