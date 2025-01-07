use std::time::{SystemTime, UNIX_EPOCH};

use cosmwasm_std::{coins, Addr, Decimal};
use fixed_odds_market::{
    contract::{ADMIN_ADDRESS, TREASURY_ADDRESS},
    msg::InstantiateMsg,
    state::Status,
};
use helpers::setup_blockchain_and_contract;

const NATIVE_DENOM: &str = "denom";
// const FAKE_DENOM: &str = "fakedenom";
const USER_A: &str = "USER_A";
const USER_B: &str = "USER_B";
const USER_C: &str = "USER_C";
const INITIAL_BALANCE: u128 = 1_000_000_000_000;

mod helpers;

#[test]
fn it_creates_a_market() {
    let start_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
        + 60 * 5; // 5 minutes from now
    let blockchain_contract = setup_blockchain_and_contract(
        Addr::unchecked(ADMIN_ADDRESS),
        vec![
            (
                Addr::unchecked(ADMIN_ADDRESS),
                coins(INITIAL_BALANCE, NATIVE_DENOM),
            ),
            (
                Addr::unchecked(USER_A),
                coins(INITIAL_BALANCE, NATIVE_DENOM),
            ),
            (
                Addr::unchecked(USER_B),
                coins(INITIAL_BALANCE, NATIVE_DENOM),
            ),
            (
                Addr::unchecked(USER_C),
                coins(INITIAL_BALANCE, NATIVE_DENOM),
            ),
        ],
        InstantiateMsg {
            denom: NATIVE_DENOM.to_string(),
            id: "game-cs2-test-league".to_string(),
            label: "CS2 - Test League - Team A vs Team B".to_string(),
            home_team: "Team A".to_string(),
            away_team: "Team B".to_string(),
            fee_spread_odds: Decimal::from_atomics(15_u128, 2).unwrap(), // 0.15
            max_bet_risk_factor: Decimal::from_atomics(15_u128, 1).unwrap(), // 1.5
            seed_liquidity_amplifier: Decimal::from_atomics(3_u128, 0).unwrap(), // 3
            initial_odds_home: Decimal::from_atomics(22_u128, 1).unwrap(), // 2.2
            initial_odds_away: Decimal::from_atomics(18_u128, 1).unwrap(), // 1.8
            start_timestamp,                                             // 5 minutes from now
        },
        coins(100_000_000, NATIVE_DENOM),
    );

    let query_config = blockchain_contract.query_config().unwrap();
    assert_eq!(ADMIN_ADDRESS, query_config.config.admin_addr.as_str());
    assert_eq!(TREASURY_ADDRESS, query_config.config.treasury_addr.as_str());
    assert_eq!(NATIVE_DENOM, query_config.config.denom.as_str());

    let query_market = blockchain_contract.query_market().unwrap();
    assert_eq!("game-cs2-test-league", query_market.market.id);
    assert_eq!(
        "CS2 - Test League - Team A vs Team B",
        query_market.market.label
    );
    assert_eq!("Team A", query_market.market.home_team);
    assert_eq!("Team B", query_market.market.away_team);
    assert_eq!(start_timestamp, query_market.market.start_timestamp);
    assert_eq!(Status::ACTIVE, query_market.market.status);
    assert_eq!(None, query_market.market.result);
    assert_eq!(
        Decimal::from_atomics(191_u128, 2).unwrap(),
        query_market.market.home_odds
    );
    assert_eq!(
        Decimal::from_atomics(156_u128, 2).unwrap(),
        query_market.market.away_odds
    );
}
