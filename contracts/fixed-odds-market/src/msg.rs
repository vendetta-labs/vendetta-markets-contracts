use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal};

use crate::state::{BetAmount, Config, Market, MarketResult};

#[cw_serde]
pub struct InstantiateMsg {
    pub denom: String,
    pub denom_precision: u32,
    pub id: String,
    pub label: String,
    pub home_team: String,
    pub away_team: String,
    pub fee_spread_odds: Decimal,     // Fee spread in percentage points
    pub max_bet_risk_factor: Decimal, // Max bet risk factor in multiplier, ex: 1.5x
    pub seed_liquidity_amplifier: Decimal, // Seed liquidity amplifier in multiplier, ex: 3x
    pub initial_odds_home: Decimal,
    pub initial_odds_away: Decimal,
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
        initial_odds_home: Option<Decimal>,
        initial_odds_away: Option<Decimal>,
        start_timestamp: Option<u64>,
    },
    Score {
        result: MarketResult,
    },
    Cancel {},
}

#[cw_serde]
pub struct UpdateParams {
    pub fee_spread_odds: Option<Decimal>, // Fee spread in percentage points
    pub max_bet_risk_factor: Option<Decimal>, // Max bet risk factor in multiplier, ex: 1.5x
    pub seed_liquidity_amplifier: Option<Decimal>, // Seed liquidity amplifier in multiplier, ex: 3x
    pub initial_odds_home: Option<Decimal>,
    pub initial_odds_away: Option<Decimal>,
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
pub struct TotalAmounts {
    pub home: BetAmount,
    pub away: BetAmount,
}

#[cw_serde]
pub struct PotentialPayouts {
    pub home: u128,
    pub away: u128,
}

#[cw_serde]
pub struct BetsResponse {
    pub total_amounts: TotalAmounts,
    pub potential_payouts: PotentialPayouts,
}

#[cw_serde]
pub struct BetRecordWithOdds {
    pub bet_amount: BetAmount,
    pub payout: u128,
    pub odds: Decimal,
}

#[cw_serde]
pub struct AllBets {
    pub home: BetRecordWithOdds,
    pub away: BetRecordWithOdds,
}

#[cw_serde]
pub struct BetsByAddressResponse {
    pub address: Addr,
    pub all_bets: AllBets,
}
