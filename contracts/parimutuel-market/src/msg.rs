use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

use crate::state::{Config, Market, MarketResult};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin_addr: Addr,
    pub treasury_addr: Addr,
    pub fee_bps: u64, // Fee in basis points
    pub denom: String,
    pub denom_precision: u32,
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
        admin_addr: Option<Addr>,
        treasury_addr: Option<Addr>,
        fee_bps: Option<u64>,
        start_timestamp: Option<u64>,
    },
    Score {
        result: MarketResult,
    },
    Cancel {},
}

#[cw_serde]
pub struct UpdateParams {
    pub admin_addr: Option<Addr>,
    pub treasury_addr: Option<Addr>,
    pub fee_bps: Option<u64>, // Fee in basis points
    pub start_timestamp: Option<u64>,
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
pub struct TotalBets {
    pub home: u128,
    pub away: u128,
    pub draw: u128,
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
