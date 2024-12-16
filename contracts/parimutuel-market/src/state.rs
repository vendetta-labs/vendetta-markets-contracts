use std::fmt;

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const CONFIG: Item<Config> = Item::new("config");
pub const MARKET: Item<Market> = Item::new("market");
pub const POOL_HOME: Map<Addr, u128> = Map::new("pool_home");
pub const POOL_AWAY: Map<Addr, u128> = Map::new("pool_away");
pub const POOL_DRAW: Map<Addr, u128> = Map::new("pool_draw");
pub const TOTAL_HOME: Item<u128> = Item::new("total_home");
pub const TOTAL_AWAY: Item<u128> = Item::new("total_away");
pub const TOTAL_DRAW: Item<u128> = Item::new("total_draw");
pub const CLAIMS: Map<Addr, bool> = Map::new("claims");

#[derive(Serialize, Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct Config {
    pub admin_addr: Addr,
    pub treasury_addr: Addr,
    pub fee_bps: u64,
    pub denom: String,
}

#[derive(Serialize, Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub enum Status {
    ACTIVE,
    CLOSED,
    CANCELLED,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Status::ACTIVE => write!(f, "ACTIVE"),
            Status::CLOSED => write!(f, "CLOSED"),
            Status::CANCELLED => write!(f, "CANCELLED"),
        }
    }
}

#[derive(Serialize, Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub enum MarketResult {
    HOME,
    AWAY,
    DRAW,
}

impl fmt::Display for MarketResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MarketResult::HOME => write!(f, "HOME"),
            MarketResult::AWAY => write!(f, "AWAY"),
            MarketResult::DRAW => write!(f, "DRAW"),
        }
    }
}

#[derive(Serialize, Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct Market {
    pub id: String,
    pub label: String,
    pub home_team: String,
    pub away_team: String,
    pub start_timestamp: u64,
    pub status: Status,
    pub result: Option<MarketResult>,
    pub is_drawable: bool,
}
