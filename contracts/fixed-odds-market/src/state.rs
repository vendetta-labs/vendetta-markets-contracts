use std::fmt;

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const CONFIG: Item<Config> = Item::new("config");
pub const MARKET: Item<Market> = Item::new("market");
pub const CLAIMS: Map<Addr, bool> = Map::new("claims");

pub const HOME_TOTAL_PAYOUT: Item<Uint128> = Item::new("home_total_payout");
pub const HOME_BETS: Map<Addr, Bet> = Map::new("home_bets");
pub const AWAY_TOTAL_PAYOUT: Item<Uint128> = Item::new("away_total_payout");
pub const AWAY_BETS: Map<Addr, Bet> = Map::new("away_bets");

pub type Odd = Uint128;
pub type BetAmount = Uint128;
pub type Bet = (Odd, BetAmount);

#[derive(Serialize, Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct Config {
    pub admin_addr: Addr,
    pub treasury_addr: Addr,
    pub fee_bps: u64,
    pub max_bet_ratio: u64,
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
}

impl fmt::Display for MarketResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MarketResult::HOME => write!(f, "HOME"),
            MarketResult::AWAY => write!(f, "AWAY"),
        }
    }
}

#[derive(Serialize, Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct Market {
    pub id: String,
    pub label: String,
    pub home_team: String,
    pub home_odds: Odd,
    pub away_team: String,
    pub away_odds: Odd,
    pub start_timestamp: u64,
    pub status: Status,
    pub result: Option<MarketResult>,
}
