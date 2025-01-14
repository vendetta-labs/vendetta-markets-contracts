use cosmwasm_std::{coin, coins, Addr, Decimal, Timestamp, Uint128};
use cw_multi_test::MockApiBech32;
use fixed_odds_market::{
    contract::{ADMIN_ADDRESS, TREASURY_ADDRESS},
    error::ContractError,
    msg::InstantiateMsg,
    state::{MarketResult, Status},
};
use helpers::setup_blockchain_and_contract;
use std::time::{SystemTime, UNIX_EPOCH};

const NATIVE_DENOM: &str = "denom";
const NATIVE_DENOM_PRECISION: u32 = 6;
const FAKE_DENOM: &str = "fakedenom";
const OTHER: &str = "OTHER";
const ANYONE: &str = "ANYONE";
const USER_A: &str = "USER_A";
const USER_B: &str = "USER_B";
const USER_C: &str = "USER_C";
const INITIAL_BALANCE: u128 = 1_000_000_000_000;

mod helpers;

mod create_market {
    use super::*;

    #[test]
    fn it_properly_creates_a_market() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 5; // 5 minutes from now

        let blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![(
                Addr::unchecked(ADMIN_ADDRESS),
                coins(INITIAL_BALANCE, NATIVE_DENOM),
            )],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                fee_spread_odds: Decimal::from_atomics(15_u128, 2).unwrap(), // 0.15
                max_bet_risk_factor: Decimal::from_atomics(15_u128, 1).unwrap(), // 1.5
                seed_liquidity_amplifier: Decimal::from_atomics(3_u128, 0).unwrap(), // 3
                initial_odds_home: Decimal::from_atomics(2_2_u128, 1).unwrap(), // 2.2
                initial_odds_away: Decimal::from_atomics(1_8_u128, 1).unwrap(), // 1.8
                start_timestamp,
            },
            coins(100_000_000, NATIVE_DENOM),
        )
        .unwrap();

        let query_config = blockchain_contract.query_config().unwrap();
        assert_eq!(ADMIN_ADDRESS, query_config.config.admin_addr.as_str());
        assert_eq!(TREASURY_ADDRESS, query_config.config.treasury_addr.as_str());
        assert_eq!(NATIVE_DENOM, query_config.config.denom.as_str());
        assert_eq!(
            Decimal::from_atomics(15_u128, 2).unwrap(),
            query_config.config.fee_spread_odds
        );
        assert_eq!(
            Decimal::from_atomics(15_u128, 1).unwrap(),
            query_config.config.max_bet_risk_factor
        );
        assert_eq!(
            Decimal::from_atomics(3_u128, 0).unwrap(),
            query_config.config.seed_liquidity_amplifier
        );
        assert_eq!(
            Decimal::from_atomics(22_u128, 1).unwrap(),
            query_config.config.initial_odds_home
        );
        assert_eq!(
            Decimal::from_atomics(18_u128, 1).unwrap(),
            query_config.config.initial_odds_away
        );

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
            Decimal::from_atomics(1_91_u128, 2).unwrap(),
            query_market.market.home_odds
        );
        assert_eq!(
            Decimal::from_atomics(1_56_u128, 2).unwrap(),
            query_market.market.away_odds
        );

        let query_max_bets = blockchain_contract.query_max_bets().unwrap();
        assert_eq!(34_904_013, query_max_bets.home);
        assert_eq!(42_735_042, query_max_bets.away);
    }

    #[test]
    fn unauthorized() {
        let err = setup_blockchain_and_contract(
            Addr::unchecked(USER_A),
            vec![(
                Addr::unchecked(USER_A),
                coins(INITIAL_BALANCE, NATIVE_DENOM),
            )],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                fee_spread_odds: Decimal::from_atomics(15_u128, 2).unwrap(), // 0.15
                max_bet_risk_factor: Decimal::from_atomics(15_u128, 1).unwrap(), // 1.5
                seed_liquidity_amplifier: Decimal::from_atomics(3_u128, 0).unwrap(), // 3
                initial_odds_home: Decimal::from_atomics(2_2_u128, 1).unwrap(), // 2.2
                initial_odds_away: Decimal::from_atomics(1_8_u128, 1).unwrap(), // 1.8
                start_timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs()
                    + 60 * 5, // 5 minutes from now
            },
            coins(100_000_000, NATIVE_DENOM),
        )
        .unwrap_err();

        assert_eq!(
            ContractError::Unauthorized {},
            err.downcast::<ContractError>().unwrap()
        );
    }

    #[test]
    fn market_not_initially_funded() {
        let err = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![(
                Addr::unchecked(ADMIN_ADDRESS),
                coins(INITIAL_BALANCE, NATIVE_DENOM),
            )],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                fee_spread_odds: Decimal::from_atomics(15_u128, 2).unwrap(), // 0.15
                max_bet_risk_factor: Decimal::from_atomics(15_u128, 1).unwrap(), // 1.5
                seed_liquidity_amplifier: Decimal::from_atomics(3_u128, 0).unwrap(), // 3
                initial_odds_home: Decimal::from_atomics(2_2_u128, 1).unwrap(), // 2.2
                initial_odds_away: Decimal::from_atomics(1_8_u128, 1).unwrap(), // 1.8
                start_timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs()
                    + 60 * 5, // 5 minutes from now
            },
            vec![],
        )
        .unwrap_err();

        assert_eq!(
            ContractError::MarketNotInitiallyFunded {},
            err.downcast::<ContractError>().unwrap()
        );
    }
}

mod place_bet {
    use super::*;

    #[test]
    fn it_properly_accepts_bets_calculates_the_winnings_for_the_market_and_collects_fees() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 5; // 5 minutes from now

        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![
                (
                    Addr::unchecked(ADMIN_ADDRESS),
                    coins(INITIAL_BALANCE, NATIVE_DENOM),
                ),
                (
                    MockApiBech32::new("neutron").addr_make(USER_A),
                    coins(INITIAL_BALANCE, NATIVE_DENOM),
                ),
                (
                    MockApiBech32::new("neutron").addr_make(USER_B),
                    coins(INITIAL_BALANCE, NATIVE_DENOM),
                ),
                (
                    MockApiBech32::new("neutron").addr_make(USER_C),
                    coins(INITIAL_BALANCE, NATIVE_DENOM),
                ),
            ],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                fee_spread_odds: Decimal::from_atomics(15_u128, 2).unwrap(), // 0.15
                max_bet_risk_factor: Decimal::from_atomics(15_u128, 1).unwrap(), // 1.5
                seed_liquidity_amplifier: Decimal::from_atomics(3_u128, 0).unwrap(), // 3
                initial_odds_home: Decimal::from_atomics(22_u128, 1).unwrap(), // 2.2
                initial_odds_away: Decimal::from_atomics(18_u128, 1).unwrap(), // 1.8
                start_timestamp,
            },
            coins(100_000_000, NATIVE_DENOM),
        )
        .unwrap();

        let query_max_bets = blockchain_contract.query_max_bets().unwrap();
        assert_eq!(34_904_013, query_max_bets.home);
        assert_eq!(42_735_042, query_max_bets.away);

        let user_a = blockchain_contract.blockchain.api().addr_make(USER_A);

        blockchain_contract
            .place_bet(
                &user_a,
                MarketResult::HOME,
                Decimal::from_atomics(1_91_u128, 2).unwrap(),
                None,
                &coins(10_000_000, NATIVE_DENOM),
            )
            .unwrap();

        let query_max_bets = blockchain_contract.query_max_bets().unwrap();
        assert_eq!(32_934_782, query_max_bets.home);
        assert_eq!(45_548_654, query_max_bets.away);

        let user_b = blockchain_contract.blockchain.api().addr_make(USER_B);

        blockchain_contract
            .place_bet(
                &user_b,
                MarketResult::AWAY,
                Decimal::from_atomics(1_61_u128, 2).unwrap(),
                None,
                &coins(10_000_000, NATIVE_DENOM),
            )
            .unwrap();

        let query_max_bets = blockchain_contract.query_max_bets().unwrap();
        assert_eq!(35_403_508, query_max_bets.home);
        assert_eq!(44_118_895, query_max_bets.away);

        let user_c = blockchain_contract.blockchain.api().addr_make(USER_C);

        blockchain_contract
            .place_bet(
                &user_c,
                MarketResult::AWAY,
                Decimal::from_atomics(1_57_u128, 2).unwrap(),
                None,
                &coins(40_000_000, NATIVE_DENOM),
            )
            .unwrap();

        blockchain_contract
            .place_bet(
                &user_c,
                MarketResult::HOME,
                Decimal::from_atomics(2_13_u128, 2).unwrap(),
                None,
                &coins(20_000_000, NATIVE_DENOM),
            )
            .unwrap();

        let query_max_bets = blockchain_contract.query_max_bets().unwrap();
        assert_eq!(39_831_649, query_max_bets.home);
        assert_eq!(44_342_105, query_max_bets.away);

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(30_000_000, query_bets.total_amounts.home);
        assert_eq!(50_000_000, query_bets.total_amounts.away);
        assert_eq!(61_700_000, query_bets.potential_payouts.home);
        assert_eq!(78_900_000, query_bets.potential_payouts.away);

        blockchain_contract.blockchain.update_block(|block| {
            block.time = Timestamp::from_seconds(
                start_timestamp + 60 * 30, // 30 minutes after the start timestamp
            );
        });

        let treasury_balance = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(Addr::unchecked(TREASURY_ADDRESS), NATIVE_DENOM)
            .unwrap();
        assert_eq!(0_u128, treasury_balance.amount.into());

        let market_balance = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(blockchain_contract.addr().clone(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(180_000_000_u128, market_balance.amount.into());

        blockchain_contract
            .score_market(&Addr::unchecked(ADMIN_ADDRESS), MarketResult::AWAY)
            .unwrap();

        let query_market = blockchain_contract.query_market().unwrap();
        assert_eq!(Status::CLOSED, query_market.market.status);
        assert_eq!(MarketResult::AWAY, query_market.market.result.unwrap());

        let query_max_bets = blockchain_contract.query_max_bets().unwrap();
        assert_eq!(0, query_max_bets.home);
        assert_eq!(0, query_max_bets.away);

        let treasury_balance = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(Addr::unchecked(TREASURY_ADDRESS), NATIVE_DENOM)
            .unwrap();
        assert_eq!(
            180_000_000_u128 - 78_900_000_u128,
            treasury_balance.amount.into()
        );

        let market_balance_before = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(blockchain_contract.addr().clone(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(78_900_000_u128, market_balance_before.amount.into());

        let user_a_balance_before = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(user_a.clone(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(
            INITIAL_BALANCE - 10_000_000_u128,
            user_a_balance_before.amount.into()
        );

        let err = blockchain_contract
            .claim_winnings(&user_a, None)
            .unwrap_err();
        assert_eq!(
            ContractError::NoWinnings {},
            err.downcast::<ContractError>().unwrap()
        );

        let user_a_balance_after = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(user_a.clone(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(user_a_balance_before.amount, user_a_balance_after.amount);

        let user_b_balance_before = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(user_b.clone(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(
            INITIAL_BALANCE - 10_000_000_u128,
            user_b_balance_before.amount.into()
        );

        blockchain_contract.claim_winnings(&user_b, None).unwrap();

        let user_b_balance_after = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(user_b.clone(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(
            user_b_balance_before.amount + Uint128::new(16_100_000_u128),
            user_b_balance_after.amount
        );

        let user_c_balance_before = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(user_c.clone(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(
            INITIAL_BALANCE - 60_000_000_u128,
            user_c_balance_before.amount.into()
        );

        blockchain_contract.claim_winnings(&user_c, None).unwrap();

        let user_c_balance_after = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(user_c.clone(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(
            user_c_balance_before.amount + Uint128::new(62_800_000),
            user_c_balance_after.amount
        );

        let market_balance_after = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(blockchain_contract.addr().clone(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(0_u128, market_balance_after.amount.into());
    }

    #[test]
    fn it_properly_averages_bets_when_there_are_multiple_bets_from_the_same_address() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 5; // 5 minutes from now

        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![
                (
                    Addr::unchecked(ADMIN_ADDRESS),
                    coins(INITIAL_BALANCE, NATIVE_DENOM),
                ),
                (
                    MockApiBech32::new("neutron").addr_make(USER_A),
                    coins(INITIAL_BALANCE, NATIVE_DENOM),
                ),
            ],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                fee_spread_odds: Decimal::from_atomics(15_u128, 2).unwrap(), // 0.15
                max_bet_risk_factor: Decimal::from_atomics(15_u128, 1).unwrap(), // 1.5
                seed_liquidity_amplifier: Decimal::from_atomics(3_u128, 0).unwrap(), // 3
                initial_odds_home: Decimal::from_atomics(22_u128, 1).unwrap(), // 2.2
                initial_odds_away: Decimal::from_atomics(18_u128, 1).unwrap(), // 1.8
                start_timestamp,
            },
            coins(100_000_000, NATIVE_DENOM),
        )
        .unwrap();

        let user_a = blockchain_contract.blockchain.api().addr_make(USER_A);

        blockchain_contract
            .place_bet(
                &user_a,
                MarketResult::HOME,
                Decimal::from_atomics(1_91_u128, 2).unwrap(),
                None,
                &coins(20_000_000, NATIVE_DENOM),
            )
            .unwrap();

        blockchain_contract
            .place_bet(
                &user_a,
                MarketResult::HOME,
                Decimal::from_atomics(1_77_u128, 2).unwrap(),
                None,
                &coins(15_000_000, NATIVE_DENOM),
            )
            .unwrap();

        blockchain_contract
            .place_bet(
                &user_a,
                MarketResult::HOME,
                Decimal::from_atomics(1_69_u128, 2).unwrap(),
                None,
                &coins(24_560_000, NATIVE_DENOM),
            )
            .unwrap();

        let user_a_payout = 38_200_000_u128 + 26_550_000_u128 + 41_506_400_u128;

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(59_560_000, query_bets.total_amounts.home);
        assert_eq!(0, query_bets.total_amounts.away);
        assert_eq!(user_a_payout, query_bets.potential_payouts.home);
        assert_eq!(0, query_bets.potential_payouts.away);

        let query_user_a_bets = blockchain_contract.query_bets_by_address(&user_a).unwrap();
        let home_bet_record = query_user_a_bets.all_bets.home;
        assert_eq!(
            Decimal::from_atomics(1_78402283411685695_u128, 17).unwrap(),
            home_bet_record.odds
        );
        assert_eq!(59_560_000_u128, home_bet_record.bet_amount);
        assert_eq!(user_a_payout, home_bet_record.payout);
        let away_bet_record = query_user_a_bets.all_bets.away;
        assert_eq!(Decimal::zero(), away_bet_record.odds);
        assert_eq!(0_u128, away_bet_record.bet_amount);
        assert_eq!(0_u128, away_bet_record.payout);

        blockchain_contract.blockchain.update_block(|block| {
            block.time = Timestamp::from_seconds(
                start_timestamp + 60 * 30, // 30 minutes after the start timestamp
            );
        });

        blockchain_contract
            .score_market(&Addr::unchecked(ADMIN_ADDRESS), MarketResult::HOME)
            .unwrap();

        let query_market = blockchain_contract.query_market().unwrap();
        assert_eq!(Status::CLOSED, query_market.market.status);
        assert_eq!(MarketResult::HOME, query_market.market.result.unwrap());

        let user_a_balance_before = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(user_a.clone(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(
            INITIAL_BALANCE - 59_560_000_u128,
            user_a_balance_before.amount.into()
        );

        blockchain_contract.claim_winnings(&user_a, None).unwrap();

        let user_a_balance_after = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(user_a.clone(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(
            INITIAL_BALANCE + user_a_payout - 59_560_000_u128,
            user_a_balance_after.amount.into()
        );
    }

    #[test]
    fn the_receiver_will_be_the_beneficiary_when_defined() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 5; // 5 minutes from now

        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![
                (
                    Addr::unchecked(ADMIN_ADDRESS),
                    coins(INITIAL_BALANCE, NATIVE_DENOM),
                ),
                (
                    MockApiBech32::new("neutron").addr_make(USER_A),
                    coins(INITIAL_BALANCE, NATIVE_DENOM),
                ),
            ],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                fee_spread_odds: Decimal::from_atomics(7_u128, 2).unwrap(), // 0.07
                max_bet_risk_factor: Decimal::from_atomics(15_u128, 1).unwrap(), // 1.5
                seed_liquidity_amplifier: Decimal::from_atomics(3_u128, 0).unwrap(), // 3
                initial_odds_home: Decimal::from_atomics(22_u128, 1).unwrap(), // 2.2
                initial_odds_away: Decimal::from_atomics(18_u128, 1).unwrap(), // 1.8
                start_timestamp,
            },
            coins(100_000_000, NATIVE_DENOM),
        )
        .unwrap();

        let user_a = blockchain_contract.blockchain.api().addr_make(USER_A);
        let user_b = blockchain_contract.blockchain.api().addr_make(USER_B);

        blockchain_contract
            .place_bet(
                &user_a,
                MarketResult::AWAY,
                Decimal::from_atomics(1_68_u128, 2).unwrap(),
                Some(user_b.clone()),
                &coins(10_000_000, NATIVE_DENOM),
            )
            .unwrap();

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(0, query_bets.total_amounts.home);
        assert_eq!(10_000_000, query_bets.total_amounts.away);

        let query_user_a_bets = blockchain_contract.query_bets_by_address(&user_a).unwrap();
        let home_bet_record = query_user_a_bets.all_bets.home;
        assert_eq!(Decimal::zero(), home_bet_record.odds);
        assert_eq!(0_u128, home_bet_record.bet_amount);
        assert_eq!(0_u128, home_bet_record.payout);
        let away_bet_record = query_user_a_bets.all_bets.away;
        assert_eq!(Decimal::zero(), away_bet_record.odds);
        assert_eq!(0_u128, away_bet_record.bet_amount);
        assert_eq!(0_u128, away_bet_record.payout);

        let query_user_b_bets = blockchain_contract.query_bets_by_address(&user_b).unwrap();
        let home_bet_record = query_user_b_bets.all_bets.home;
        assert_eq!(Decimal::zero(), home_bet_record.odds);
        assert_eq!(0_u128, home_bet_record.bet_amount);
        assert_eq!(0_u128, home_bet_record.payout);
        let away_bet_record = query_user_b_bets.all_bets.away;
        assert_eq!(
            Decimal::from_atomics(1_68_u128, 2).unwrap(),
            away_bet_record.odds
        );
        assert_eq!(10_000_000_u128, away_bet_record.bet_amount);
        assert_eq!(16_800_000_u128, away_bet_record.payout);

        blockchain_contract.blockchain.update_block(|block| {
            block.time = Timestamp::from_seconds(
                start_timestamp + 60 * 30, // 30 minutes after the start timestamp
            );
        });

        blockchain_contract
            .score_market(&Addr::unchecked(ADMIN_ADDRESS), MarketResult::AWAY)
            .unwrap();

        let query_market = blockchain_contract.query_market().unwrap();
        assert_eq!(Status::CLOSED, query_market.market.status);
        assert_eq!(MarketResult::AWAY, query_market.market.result.unwrap());

        let user_a_balance_before = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(user_a.clone(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(
            INITIAL_BALANCE - 10_000_000_u128,
            user_a_balance_before.amount.into()
        );

        let err = blockchain_contract
            .claim_winnings(&user_a, None)
            .unwrap_err();
        assert_eq!(
            ContractError::NoWinnings {},
            err.downcast::<ContractError>().unwrap()
        );

        let user_a_balance_after = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(user_a.clone(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(user_a_balance_before.amount, user_a_balance_after.amount);

        let user_b_balance_before = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(user_b.clone(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(0_u128, user_b_balance_before.amount.into());

        blockchain_contract.claim_winnings(&user_b, None).unwrap();

        let user_b_balance_after = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(user_b.clone(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(
            user_b_balance_before.amount + Uint128::new(16_800_000_u128),
            user_b_balance_after.amount
        );
    }

    #[test]
    fn it_cant_place_bet_if_market_isnt_active() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 5; // 5 minutes from now

        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![
                (
                    Addr::unchecked(ADMIN_ADDRESS),
                    coins(INITIAL_BALANCE, NATIVE_DENOM),
                ),
                (
                    MockApiBech32::new("neutron").addr_make(USER_A),
                    coins(INITIAL_BALANCE, NATIVE_DENOM),
                ),
            ],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                fee_spread_odds: Decimal::from_atomics(15_u128, 2).unwrap(), // 0.15
                max_bet_risk_factor: Decimal::from_atomics(15_u128, 1).unwrap(), // 1.5
                seed_liquidity_amplifier: Decimal::from_atomics(3_u128, 0).unwrap(), // 3
                initial_odds_home: Decimal::from_atomics(22_u128, 1).unwrap(), // 2.2
                initial_odds_away: Decimal::from_atomics(18_u128, 1).unwrap(), // 1.8
                start_timestamp,
            },
            coins(100_000_000, NATIVE_DENOM),
        )
        .unwrap();

        blockchain_contract
            .cancel_market(&Addr::unchecked(ADMIN_ADDRESS))
            .unwrap();

        let query_market = blockchain_contract.query_market().unwrap();
        assert_ne!(Status::ACTIVE, query_market.market.status);

        let user_a = blockchain_contract.blockchain.api().addr_make(USER_A);

        let err = blockchain_contract
            .place_bet(
                &user_a,
                MarketResult::HOME,
                Decimal::from_atomics(1_91_u128, 2).unwrap(),
                None,
                &coins(10_000_000, NATIVE_DENOM),
            )
            .unwrap_err();
        assert_eq!(
            ContractError::MarketNotActive {},
            err.downcast::<ContractError>().unwrap()
        );

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(0, query_bets.total_amounts.home);
        assert_eq!(0, query_bets.total_amounts.away);

        let query_user_a_bets = blockchain_contract.query_bets_by_address(&user_a).unwrap();
        let home_bet_record = query_user_a_bets.all_bets.home;
        assert_eq!(Decimal::zero(), home_bet_record.odds);
        assert_eq!(0_u128, home_bet_record.bet_amount);
        assert_eq!(0_u128, home_bet_record.payout);
        let away_bet_record = query_user_a_bets.all_bets.away;
        assert_eq!(Decimal::zero(), away_bet_record.odds);
        assert_eq!(0_u128, away_bet_record.bet_amount);
        assert_eq!(0_u128, away_bet_record.payout);
    }

    #[test]
    fn it_can_only_place_bets_up_until_5_minutes_before_market_start_timestamp() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 5; // 5 minutes from now

        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![
                (
                    Addr::unchecked(ADMIN_ADDRESS),
                    coins(INITIAL_BALANCE, NATIVE_DENOM),
                ),
                (
                    MockApiBech32::new("neutron").addr_make(USER_A),
                    coins(INITIAL_BALANCE, NATIVE_DENOM),
                ),
            ],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                fee_spread_odds: Decimal::from_atomics(15_u128, 2).unwrap(), // 0.15
                max_bet_risk_factor: Decimal::from_atomics(15_u128, 1).unwrap(), // 1.5
                seed_liquidity_amplifier: Decimal::from_atomics(3_u128, 0).unwrap(), // 3
                initial_odds_home: Decimal::from_atomics(22_u128, 1).unwrap(), // 2.2
                initial_odds_away: Decimal::from_atomics(18_u128, 1).unwrap(), // 1.8
                start_timestamp,
            },
            coins(100_000_000, NATIVE_DENOM),
        )
        .unwrap();

        blockchain_contract.blockchain.update_block(|block| {
            block.time = Timestamp::from_seconds(start_timestamp - 5 * 60 + 1); // 4 minutes 59 seconds before start timestamp
        });

        let query_market = blockchain_contract.query_market().unwrap();
        assert_eq!(Status::ACTIVE, query_market.market.status);

        let user_a = blockchain_contract.blockchain.api().addr_make(USER_A);

        let err = blockchain_contract
            .place_bet(
                &user_a,
                MarketResult::HOME,
                Decimal::from_atomics(1_91_u128, 2).unwrap(),
                None,
                &coins(10_000_000, NATIVE_DENOM),
            )
            .unwrap_err();
        assert_eq!(
            ContractError::BetsNotAccepted {},
            err.downcast::<ContractError>().unwrap()
        );

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(0, query_bets.total_amounts.home);
        assert_eq!(0, query_bets.total_amounts.away);

        let query_user_a_bets = blockchain_contract.query_bets_by_address(&user_a).unwrap();
        let home_bet_record = query_user_a_bets.all_bets.home;
        assert_eq!(Decimal::zero(), home_bet_record.odds);
        assert_eq!(0_u128, home_bet_record.bet_amount);
        assert_eq!(0_u128, home_bet_record.payout);
        let away_bet_record = query_user_a_bets.all_bets.away;
        assert_eq!(Decimal::zero(), away_bet_record.odds);
        assert_eq!(0_u128, away_bet_record.bet_amount);
        assert_eq!(0_u128, away_bet_record.payout);
    }

    #[test]
    fn it_cant_place_bet_without_sending_funds_in_the_market_denom() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 5; // 5 minutes from now

        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![
                (
                    Addr::unchecked(ADMIN_ADDRESS),
                    coins(INITIAL_BALANCE, NATIVE_DENOM),
                ),
                (
                    MockApiBech32::new("neutron").addr_make(USER_A),
                    vec![
                        coin(INITIAL_BALANCE, NATIVE_DENOM),
                        coin(INITIAL_BALANCE, FAKE_DENOM),
                    ],
                ),
            ],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                fee_spread_odds: Decimal::from_atomics(15_u128, 2).unwrap(), // 0.15
                max_bet_risk_factor: Decimal::from_atomics(15_u128, 1).unwrap(), // 1.5
                seed_liquidity_amplifier: Decimal::from_atomics(3_u128, 0).unwrap(), // 3
                initial_odds_home: Decimal::from_atomics(22_u128, 1).unwrap(), // 2.2
                initial_odds_away: Decimal::from_atomics(18_u128, 1).unwrap(), // 1.8
                start_timestamp,
            },
            coins(100_000_000, NATIVE_DENOM),
        )
        .unwrap();

        let user_a = blockchain_contract.blockchain.api().addr_make(USER_A);

        let err = blockchain_contract
            .place_bet(
                &user_a,
                MarketResult::HOME,
                Decimal::from_atomics(1_91_u128, 2).unwrap(),
                None,
                &[],
            )
            .unwrap_err();
        assert_eq!(
            ContractError::PaymentError {},
            err.downcast::<ContractError>().unwrap()
        );

        let err = blockchain_contract
            .place_bet(
                &user_a,
                MarketResult::HOME,
                Decimal::from_atomics(1_91_u128, 2).unwrap(),
                None,
                &coins(10_000_000, FAKE_DENOM),
            )
            .unwrap_err();
        assert_eq!(
            ContractError::PaymentError {},
            err.downcast::<ContractError>().unwrap()
        );

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(0, query_bets.total_amounts.home);
        assert_eq!(0, query_bets.total_amounts.away);

        let query_user_a_bets = blockchain_contract.query_bets_by_address(&user_a).unwrap();
        let home_bet_record = query_user_a_bets.all_bets.home;
        assert_eq!(Decimal::zero(), home_bet_record.odds);
        assert_eq!(0_u128, home_bet_record.bet_amount);
        assert_eq!(0_u128, home_bet_record.payout);
        let away_bet_record = query_user_a_bets.all_bets.away;
        assert_eq!(Decimal::zero(), away_bet_record.odds);
        assert_eq!(0_u128, away_bet_record.bet_amount);
        assert_eq!(0_u128, away_bet_record.payout);
    }

    #[test]
    fn it_cant_place_bet_if_the_min_odds_requirement_is_not_met() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 5; // 5 minutes from now

        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![
                (
                    Addr::unchecked(ADMIN_ADDRESS),
                    coins(INITIAL_BALANCE, NATIVE_DENOM),
                ),
                (
                    MockApiBech32::new("neutron").addr_make(USER_A),
                    coins(INITIAL_BALANCE, NATIVE_DENOM),
                ),
            ],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                fee_spread_odds: Decimal::from_atomics(15_u128, 2).unwrap(), // 0.15
                max_bet_risk_factor: Decimal::from_atomics(15_u128, 1).unwrap(), // 1.5
                seed_liquidity_amplifier: Decimal::from_atomics(3_u128, 0).unwrap(), // 3
                initial_odds_home: Decimal::from_atomics(22_u128, 1).unwrap(), // 2.2
                initial_odds_away: Decimal::from_atomics(18_u128, 1).unwrap(), // 1.8
                start_timestamp,
            },
            coins(100_000_000, NATIVE_DENOM),
        )
        .unwrap();

        let user_a = blockchain_contract.blockchain.api().addr_make(USER_A);

        let err = blockchain_contract
            .place_bet(
                &user_a,
                MarketResult::HOME,
                Decimal::from_atomics(192_u128, 2).unwrap(),
                None,
                &coins(10_000_000, NATIVE_DENOM),
            )
            .unwrap_err();
        assert_eq!(
            ContractError::MinimumOddsNotKept {},
            err.downcast::<ContractError>().unwrap()
        );

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(0, query_bets.total_amounts.home);
        assert_eq!(0, query_bets.total_amounts.away);

        let query_user_a_bets = blockchain_contract.query_bets_by_address(&user_a).unwrap();
        let home_bet_record = query_user_a_bets.all_bets.home;
        assert_eq!(Decimal::zero(), home_bet_record.odds);
        assert_eq!(0_u128, home_bet_record.bet_amount);
        assert_eq!(0_u128, home_bet_record.payout);
        let away_bet_record = query_user_a_bets.all_bets.away;
        assert_eq!(Decimal::zero(), away_bet_record.odds);
        assert_eq!(0_u128, away_bet_record.bet_amount);
        assert_eq!(0_u128, away_bet_record.payout);
    }

    #[test]
    fn it_cant_place_bet_with_amount_higher_than_the_max_allowed_bet() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 5; // 5 minutes from now

        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![
                (
                    Addr::unchecked(ADMIN_ADDRESS),
                    coins(INITIAL_BALANCE, NATIVE_DENOM),
                ),
                (
                    MockApiBech32::new("neutron").addr_make(USER_A),
                    coins(INITIAL_BALANCE, NATIVE_DENOM),
                ),
            ],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                fee_spread_odds: Decimal::from_atomics(15_u128, 2).unwrap(), // 0.15
                max_bet_risk_factor: Decimal::from_atomics(15_u128, 1).unwrap(), // 1.5
                seed_liquidity_amplifier: Decimal::from_atomics(3_u128, 0).unwrap(), // 3
                initial_odds_home: Decimal::from_atomics(22_u128, 1).unwrap(), // 2.2
                initial_odds_away: Decimal::from_atomics(18_u128, 1).unwrap(), // 1.8
                start_timestamp,
            },
            coins(100_000_000, NATIVE_DENOM),
        )
        .unwrap();

        let user_a = blockchain_contract.blockchain.api().addr_make(USER_A);

        let err = blockchain_contract
            .place_bet(
                &user_a,
                MarketResult::HOME,
                Decimal::from_atomics(1_91_u128, 2).unwrap(),
                None,
                &coins(34_910_000, NATIVE_DENOM),
            )
            .unwrap_err();
        assert_eq!(
            ContractError::MaxBetExceeded {},
            err.downcast::<ContractError>().unwrap()
        );

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(0, query_bets.total_amounts.home);
        assert_eq!(0, query_bets.total_amounts.away);

        let query_user_a_bets = blockchain_contract.query_bets_by_address(&user_a).unwrap();
        let home_bet_record = query_user_a_bets.all_bets.home;
        assert_eq!(Decimal::zero(), home_bet_record.odds);
        assert_eq!(0_u128, home_bet_record.bet_amount);
        assert_eq!(0_u128, home_bet_record.payout);
        let away_bet_record = query_user_a_bets.all_bets.away;
        assert_eq!(Decimal::zero(), away_bet_record.odds);
        assert_eq!(0_u128, away_bet_record.bet_amount);
        assert_eq!(0_u128, away_bet_record.payout);
    }
}

mod claim_winnings {
    use super::*;

    #[test]
    fn it_properly_claims_winnings() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 5; // 5 minutes from now

        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![
                (
                    Addr::unchecked(ADMIN_ADDRESS),
                    coins(INITIAL_BALANCE, NATIVE_DENOM),
                ),
                (
                    MockApiBech32::new("neutron").addr_make(USER_A),
                    coins(INITIAL_BALANCE, NATIVE_DENOM),
                ),
                (
                    MockApiBech32::new("neutron").addr_make(OTHER),
                    coins(INITIAL_BALANCE, NATIVE_DENOM),
                ),
            ],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                fee_spread_odds: Decimal::from_atomics(15_u128, 2).unwrap(), // 0.15
                max_bet_risk_factor: Decimal::from_atomics(15_u128, 1).unwrap(), // 1.5
                seed_liquidity_amplifier: Decimal::from_atomics(3_u128, 0).unwrap(), // 3
                initial_odds_home: Decimal::from_atomics(22_u128, 1).unwrap(), // 2.2
                initial_odds_away: Decimal::from_atomics(18_u128, 1).unwrap(), // 1.8
                start_timestamp,
            },
            coins(100_000_000, NATIVE_DENOM),
        )
        .unwrap();

        let user_a = blockchain_contract.blockchain.api().addr_make(USER_A);
        blockchain_contract
            .place_bet(
                &user_a,
                MarketResult::HOME,
                Decimal::from_atomics(1_91_u128, 2).unwrap(),
                None,
                &coins(10_000_000, NATIVE_DENOM),
            )
            .unwrap();

        let other = blockchain_contract.blockchain.api().addr_make(OTHER);
        blockchain_contract
            .place_bet(
                &other,
                MarketResult::AWAY,
                Decimal::from_atomics(1_61_u128, 2).unwrap(),
                None,
                &coins(5_000_000, NATIVE_DENOM),
            )
            .unwrap();

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(10_000_000, query_bets.total_amounts.home);
        assert_eq!(5_000_000, query_bets.total_amounts.away);

        blockchain_contract.blockchain.update_block(|block| {
            block.time = Timestamp::from_seconds(
                start_timestamp + 60 * 30, // 30 minutes after the start timestamp
            );
        });

        blockchain_contract
            .score_market(&Addr::unchecked(ADMIN_ADDRESS), MarketResult::HOME)
            .unwrap();

        let query_market = blockchain_contract.query_market().unwrap();
        assert_eq!(Status::CLOSED, query_market.market.status);
        assert_eq!(MarketResult::HOME, query_market.market.result.unwrap());

        let user_a_balance = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(&user_a, NATIVE_DENOM)
            .unwrap();
        assert_eq!(
            INITIAL_BALANCE - 10_000_000_u128,
            user_a_balance.amount.into()
        );

        blockchain_contract.claim_winnings(&user_a, None).unwrap();

        let user_a_balance = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(&user_a, NATIVE_DENOM)
            .unwrap();
        assert_eq!(
            INITIAL_BALANCE + 9_100_000_u128,
            user_a_balance.amount.into()
        );
    }

    #[test]
    fn it_can_claim_on_behalf_of_the_receiver_when_defined() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 5; // 5 minutes from now

        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![
                (
                    Addr::unchecked(ADMIN_ADDRESS),
                    coins(INITIAL_BALANCE, NATIVE_DENOM),
                ),
                (
                    MockApiBech32::new("neutron").addr_make(USER_A),
                    coins(INITIAL_BALANCE, NATIVE_DENOM),
                ),
            ],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                fee_spread_odds: Decimal::from_atomics(15_u128, 2).unwrap(), // 0.15
                max_bet_risk_factor: Decimal::from_atomics(15_u128, 1).unwrap(), // 1.5
                seed_liquidity_amplifier: Decimal::from_atomics(3_u128, 0).unwrap(), // 3
                initial_odds_home: Decimal::from_atomics(22_u128, 1).unwrap(), // 2.2
                initial_odds_away: Decimal::from_atomics(18_u128, 1).unwrap(), // 1.8
                start_timestamp,
            },
            coins(100_000_000, NATIVE_DENOM),
        )
        .unwrap();

        let user_a = blockchain_contract.blockchain.api().addr_make(USER_A);
        blockchain_contract
            .place_bet(
                &user_a,
                MarketResult::HOME,
                Decimal::from_atomics(1_91_u128, 2).unwrap(),
                None,
                &coins(10_000_000, NATIVE_DENOM),
            )
            .unwrap();

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(10_000_000, query_bets.total_amounts.home);
        assert_eq!(0, query_bets.total_amounts.away);

        blockchain_contract.blockchain.update_block(|block| {
            block.time = Timestamp::from_seconds(
                start_timestamp + 60 * 30, // 30 minutes after the start timestamp
            );
        });

        blockchain_contract
            .score_market(&Addr::unchecked(ADMIN_ADDRESS), MarketResult::HOME)
            .unwrap();

        let query_market = blockchain_contract.query_market().unwrap();
        assert_eq!(Status::CLOSED, query_market.market.status);
        assert_eq!(MarketResult::HOME, query_market.market.result.unwrap());

        let user_a_balance = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(&user_a, NATIVE_DENOM)
            .unwrap();
        assert_eq!(
            INITIAL_BALANCE - 10_000_000_u128,
            user_a_balance.amount.into()
        );

        let anyone = blockchain_contract.blockchain.api().addr_make(ANYONE);
        blockchain_contract
            .claim_winnings(&anyone, Some(user_a.clone()))
            .unwrap();

        let user_a_balance = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(&user_a, NATIVE_DENOM)
            .unwrap();
        assert_eq!(
            INITIAL_BALANCE + 9_100_000_u128,
            user_a_balance.amount.into()
        );
    }

    #[test]
    fn it_will_return_all_bets_made_if_market_was_cancelled() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 5; // 5 minutes from now

        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![
                (
                    Addr::unchecked(ADMIN_ADDRESS),
                    coins(INITIAL_BALANCE, NATIVE_DENOM),
                ),
                (
                    MockApiBech32::new("neutron").addr_make(USER_A),
                    coins(INITIAL_BALANCE, NATIVE_DENOM),
                ),
                (
                    MockApiBech32::new("neutron").addr_make(OTHER),
                    coins(INITIAL_BALANCE, NATIVE_DENOM),
                ),
            ],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                fee_spread_odds: Decimal::from_atomics(15_u128, 2).unwrap(), // 0.15
                max_bet_risk_factor: Decimal::from_atomics(15_u128, 1).unwrap(), // 1.5
                seed_liquidity_amplifier: Decimal::from_atomics(3_u128, 0).unwrap(), // 3
                initial_odds_home: Decimal::from_atomics(22_u128, 1).unwrap(), // 2.2
                initial_odds_away: Decimal::from_atomics(18_u128, 1).unwrap(), // 1.8
                start_timestamp,
            },
            coins(100_000_000, NATIVE_DENOM),
        )
        .unwrap();

        let user_a = blockchain_contract.blockchain.api().addr_make(USER_A);
        blockchain_contract
            .place_bet(
                &user_a,
                MarketResult::HOME,
                Decimal::from_atomics(1_91_u128, 2).unwrap(),
                None,
                &coins(10_000_000, NATIVE_DENOM),
            )
            .unwrap();

        let other = blockchain_contract.blockchain.api().addr_make(OTHER);
        blockchain_contract
            .place_bet(
                &other,
                MarketResult::AWAY,
                Decimal::from_atomics(1_61_u128, 2).unwrap(),
                None,
                &coins(5_000_000, NATIVE_DENOM),
            )
            .unwrap();

        let user_a = blockchain_contract.blockchain.api().addr_make(USER_A);
        blockchain_contract
            .place_bet(
                &user_a,
                MarketResult::AWAY,
                Decimal::from_atomics(159_u128, 2).unwrap(),
                None,
                &coins(5_000_000, NATIVE_DENOM),
            )
            .unwrap();

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(10_000_000, query_bets.total_amounts.home);
        assert_eq!(10_000_000, query_bets.total_amounts.away);

        blockchain_contract
            .cancel_market(&Addr::unchecked(ADMIN_ADDRESS))
            .unwrap();

        let query_market = blockchain_contract.query_market().unwrap();
        assert_eq!(Status::CANCELLED, query_market.market.status);

        let user_a_balance = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(&user_a, NATIVE_DENOM)
            .unwrap();
        assert_eq!(
            INITIAL_BALANCE - 15_000_000_u128,
            user_a_balance.amount.into()
        );

        blockchain_contract.claim_winnings(&user_a, None).unwrap();

        let user_a_balance = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(&user_a, NATIVE_DENOM)
            .unwrap();
        assert_eq!(INITIAL_BALANCE, user_a_balance.amount.into());

        let other_balance = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(&other, NATIVE_DENOM)
            .unwrap();
        assert_eq!(
            INITIAL_BALANCE - 5_000_000_u128,
            other_balance.amount.into()
        );

        blockchain_contract.claim_winnings(&other, None).unwrap();

        let other_balance = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(&other, NATIVE_DENOM)
            .unwrap();
        assert_eq!(INITIAL_BALANCE, other_balance.amount.into());

        let treasury_balance = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(Addr::unchecked(TREASURY_ADDRESS), NATIVE_DENOM)
            .unwrap();
        assert_eq!(0_u128, treasury_balance.amount.into());
    }

    #[test]
    fn it_cant_claim_winnings_while_market_is_active() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 5; // 5 minutes from now

        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![
                (
                    Addr::unchecked(ADMIN_ADDRESS),
                    coins(INITIAL_BALANCE, NATIVE_DENOM),
                ),
                (
                    MockApiBech32::new("neutron").addr_make(USER_A),
                    coins(INITIAL_BALANCE, NATIVE_DENOM),
                ),
            ],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                fee_spread_odds: Decimal::from_atomics(15_u128, 2).unwrap(), // 0.15
                max_bet_risk_factor: Decimal::from_atomics(15_u128, 1).unwrap(), // 1.5
                seed_liquidity_amplifier: Decimal::from_atomics(3_u128, 0).unwrap(), // 3
                initial_odds_home: Decimal::from_atomics(22_u128, 1).unwrap(), // 2.2
                initial_odds_away: Decimal::from_atomics(18_u128, 1).unwrap(), // 1.8
                start_timestamp,
            },
            coins(100_000_000, NATIVE_DENOM),
        )
        .unwrap();

        let user_a = blockchain_contract.blockchain.api().addr_make(USER_A);
        blockchain_contract
            .place_bet(
                &user_a,
                MarketResult::HOME,
                Decimal::from_atomics(1_91_u128, 2).unwrap(),
                None,
                &coins(10_000_000, NATIVE_DENOM),
            )
            .unwrap();

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(10_000_000, query_bets.total_amounts.home);
        assert_eq!(0, query_bets.total_amounts.away);

        let query_market = blockchain_contract.query_market().unwrap();
        assert_eq!(Status::ACTIVE, query_market.market.status);
        assert_eq!(None, query_market.market.result);

        let user_a_balance = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(&user_a, NATIVE_DENOM)
            .unwrap();
        assert_eq!(
            INITIAL_BALANCE - 10_000_000_u128,
            user_a_balance.amount.into()
        );

        let err = blockchain_contract
            .claim_winnings(&user_a, None)
            .unwrap_err();
        assert_eq!(
            ContractError::MarketNotClosed {},
            err.downcast::<ContractError>().unwrap()
        );

        let user_a_balance = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(&user_a, NATIVE_DENOM)
            .unwrap();
        assert_eq!(
            INITIAL_BALANCE - 10_000_000_u128,
            user_a_balance.amount.into()
        );
    }

    #[test]
    fn it_cant_claim_winnings_twice() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 5; // 5 minutes from now

        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![
                (
                    Addr::unchecked(ADMIN_ADDRESS),
                    coins(INITIAL_BALANCE, NATIVE_DENOM),
                ),
                (
                    MockApiBech32::new("neutron").addr_make(USER_A),
                    coins(INITIAL_BALANCE, NATIVE_DENOM),
                ),
            ],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                fee_spread_odds: Decimal::from_atomics(15_u128, 2).unwrap(), // 0.15
                max_bet_risk_factor: Decimal::from_atomics(15_u128, 1).unwrap(), // 1.5
                seed_liquidity_amplifier: Decimal::from_atomics(3_u128, 0).unwrap(), // 3
                initial_odds_home: Decimal::from_atomics(22_u128, 1).unwrap(), // 2.2
                initial_odds_away: Decimal::from_atomics(18_u128, 1).unwrap(), // 1.8
                start_timestamp,
            },
            coins(100_000_000, NATIVE_DENOM),
        )
        .unwrap();

        let user_a = blockchain_contract.blockchain.api().addr_make(USER_A);
        blockchain_contract
            .place_bet(
                &user_a,
                MarketResult::HOME,
                Decimal::from_atomics(1_91_u128, 2).unwrap(),
                None,
                &coins(10_000_000, NATIVE_DENOM),
            )
            .unwrap();

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(10_000_000, query_bets.total_amounts.home);
        assert_eq!(0, query_bets.total_amounts.away);

        blockchain_contract.blockchain.update_block(|block| {
            block.time = Timestamp::from_seconds(
                start_timestamp + 60 * 30, // 30 minutes after the start timestamp
            );
        });

        blockchain_contract
            .score_market(&Addr::unchecked(ADMIN_ADDRESS), MarketResult::HOME)
            .unwrap();

        let query_market = blockchain_contract.query_market().unwrap();
        assert_eq!(Status::CLOSED, query_market.market.status);
        assert_eq!(MarketResult::HOME, query_market.market.result.unwrap());

        let user_a_balance = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(&user_a, NATIVE_DENOM)
            .unwrap();
        assert_eq!(
            INITIAL_BALANCE - 10_000_000_u128,
            user_a_balance.amount.into()
        );

        blockchain_contract.claim_winnings(&user_a, None).unwrap();

        let err = blockchain_contract
            .claim_winnings(&user_a, None)
            .unwrap_err();
        assert_eq!(
            ContractError::ClaimAlreadyMade {},
            err.downcast::<ContractError>().unwrap()
        );

        let user_a_balance = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(&user_a, NATIVE_DENOM)
            .unwrap();
        assert_eq!(
            INITIAL_BALANCE + 9_100_000_u128,
            user_a_balance.amount.into()
        );
    }

    #[test]
    fn it_cant_claim_winnings_when_there_is_nothing_to_claim() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 5; // 5 minutes from now

        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![
                (
                    Addr::unchecked(ADMIN_ADDRESS),
                    coins(INITIAL_BALANCE, NATIVE_DENOM),
                ),
                (
                    MockApiBech32::new("neutron").addr_make(USER_A),
                    coins(INITIAL_BALANCE, NATIVE_DENOM),
                ),
            ],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                fee_spread_odds: Decimal::from_atomics(15_u128, 2).unwrap(), // 0.15
                max_bet_risk_factor: Decimal::from_atomics(15_u128, 1).unwrap(), // 1.5
                seed_liquidity_amplifier: Decimal::from_atomics(3_u128, 0).unwrap(), // 3
                initial_odds_home: Decimal::from_atomics(22_u128, 1).unwrap(), // 2.2
                initial_odds_away: Decimal::from_atomics(18_u128, 1).unwrap(), // 1.8
                start_timestamp,
            },
            coins(100_000_000, NATIVE_DENOM),
        )
        .unwrap();

        let user_a = blockchain_contract.blockchain.api().addr_make(USER_A);

        blockchain_contract
            .place_bet(
                &user_a,
                MarketResult::HOME,
                Decimal::from_atomics(1_91_u128, 2).unwrap(),
                None,
                &coins(10_000_000, NATIVE_DENOM),
            )
            .unwrap();

        blockchain_contract.blockchain.update_block(|block| {
            block.time = Timestamp::from_seconds(
                start_timestamp + 60 * 30, // 30 minutes after the start timestamp
            );
        });

        blockchain_contract
            .score_market(&Addr::unchecked(ADMIN_ADDRESS), MarketResult::AWAY)
            .unwrap();

        let query_market = blockchain_contract.query_market().unwrap();
        assert_eq!(Status::CLOSED, query_market.market.status);
        assert_eq!(MarketResult::AWAY, query_market.market.result.unwrap());

        let other_balance = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(&user_a, NATIVE_DENOM)
            .unwrap();
        assert_eq!(
            INITIAL_BALANCE - 10_000_000_u128,
            other_balance.amount.into()
        );

        let err = blockchain_contract
            .claim_winnings(&user_a, None)
            .unwrap_err();
        assert_eq!(
            ContractError::NoWinnings {},
            err.downcast::<ContractError>().unwrap()
        );

        let other_balance = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(&user_a, NATIVE_DENOM)
            .unwrap();
        assert_eq!(
            INITIAL_BALANCE - 10_000_000_u128,
            other_balance.amount.into()
        );
    }
}

mod update_market {
    use fixed_odds_market::msg::UpdateParams;

    use super::*;

    #[test]
    fn it_properly_updates_market_start_timestamp() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            - 60 * 5; // 5 minutes ago

        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![(
                Addr::unchecked(ADMIN_ADDRESS),
                coins(INITIAL_BALANCE, NATIVE_DENOM),
            )],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                fee_spread_odds: Decimal::from_atomics(15_u128, 2).unwrap(), // 0.15
                max_bet_risk_factor: Decimal::from_atomics(15_u128, 1).unwrap(), // 1.5
                seed_liquidity_amplifier: Decimal::from_atomics(3_u128, 0).unwrap(), // 3
                initial_odds_home: Decimal::from_atomics(22_u128, 1).unwrap(), // 2.2
                initial_odds_away: Decimal::from_atomics(18_u128, 1).unwrap(), // 1.8
                start_timestamp,
            },
            coins(100_000_000, NATIVE_DENOM),
        )
        .unwrap();

        let query_market = blockchain_contract.query_market().unwrap();
        assert_eq!(start_timestamp, query_market.market.start_timestamp);

        let new_start_timestamp = start_timestamp - 60 * 30; // 30 minutes ago

        blockchain_contract
            .update_market(
                &Addr::unchecked(ADMIN_ADDRESS),
                UpdateParams {
                    start_timestamp: Some(new_start_timestamp),
                    fee_spread_odds: None,
                    max_bet_risk_factor: None,
                    seed_liquidity_amplifier: None,
                    initial_odds_home: None,
                    initial_odds_away: None,
                },
            )
            .unwrap();

        let query_config = blockchain_contract.query_config().unwrap();
        assert_eq!(ADMIN_ADDRESS, query_config.config.admin_addr.as_str());
        assert_eq!(TREASURY_ADDRESS, query_config.config.treasury_addr.as_str());
        assert_eq!(NATIVE_DENOM, query_config.config.denom.as_str());
        assert_eq!(
            Decimal::from_atomics(15_u128, 2).unwrap(),
            query_config.config.fee_spread_odds
        );
        assert_eq!(
            Decimal::from_atomics(15_u128, 1).unwrap(),
            query_config.config.max_bet_risk_factor
        );
        assert_eq!(
            Decimal::from_atomics(3_u128, 0).unwrap(),
            query_config.config.seed_liquidity_amplifier
        );
        assert_eq!(
            Decimal::from_atomics(22_u128, 1).unwrap(),
            query_config.config.initial_odds_home
        );
        assert_eq!(
            Decimal::from_atomics(18_u128, 1).unwrap(),
            query_config.config.initial_odds_away
        );

        let query_market = blockchain_contract.query_market().unwrap();
        assert_eq!("game-cs2-test-league", query_market.market.id);
        assert_eq!(
            "CS2 - Test League - Team A vs Team B",
            query_market.market.label
        );
        assert_eq!("Team A", query_market.market.home_team);
        assert_eq!("Team B", query_market.market.away_team);
        assert_eq!(new_start_timestamp, query_market.market.start_timestamp);
        assert_eq!(Status::ACTIVE, query_market.market.status);
        assert_eq!(None, query_market.market.result);
        assert_eq!(
            Decimal::from_atomics(1_91_u128, 2).unwrap(),
            query_market.market.home_odds
        );
        assert_eq!(
            Decimal::from_atomics(1_56_u128, 2).unwrap(),
            query_market.market.away_odds
        );
    }

    #[test]
    fn it_properly_updates_market_fee_spread_odds() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            - 60 * 5; // 5 minutes ago

        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![(
                Addr::unchecked(ADMIN_ADDRESS),
                coins(INITIAL_BALANCE, NATIVE_DENOM),
            )],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                fee_spread_odds: Decimal::from_atomics(15_u128, 2).unwrap(), // 0.15
                max_bet_risk_factor: Decimal::from_atomics(15_u128, 1).unwrap(), // 1.5
                seed_liquidity_amplifier: Decimal::from_atomics(3_u128, 0).unwrap(), // 3
                initial_odds_home: Decimal::from_atomics(22_u128, 1).unwrap(), // 2.2
                initial_odds_away: Decimal::from_atomics(18_u128, 1).unwrap(), // 1.8
                start_timestamp,
            },
            coins(100_000_000, NATIVE_DENOM),
        )
        .unwrap();

        let query_config = blockchain_contract.query_config().unwrap();
        assert_eq!(
            Decimal::from_atomics(15_u128, 2).unwrap(),
            query_config.config.fee_spread_odds
        );

        let new_fee_spread_odds = Decimal::from_atomics(25_u128, 2).unwrap(); // 0.25

        blockchain_contract
            .update_market(
                &Addr::unchecked(ADMIN_ADDRESS),
                UpdateParams {
                    start_timestamp: None,
                    fee_spread_odds: Some(new_fee_spread_odds),
                    max_bet_risk_factor: None,
                    seed_liquidity_amplifier: None,
                    initial_odds_home: None,
                    initial_odds_away: None,
                },
            )
            .unwrap();

        let query_config = blockchain_contract.query_config().unwrap();
        assert_eq!(ADMIN_ADDRESS, query_config.config.admin_addr.as_str());
        assert_eq!(TREASURY_ADDRESS, query_config.config.treasury_addr.as_str());
        assert_eq!(NATIVE_DENOM, query_config.config.denom.as_str());
        assert_eq!(new_fee_spread_odds, query_config.config.fee_spread_odds);
        assert_eq!(
            Decimal::from_atomics(15_u128, 1).unwrap(),
            query_config.config.max_bet_risk_factor
        );
        assert_eq!(
            Decimal::from_atomics(3_u128, 0).unwrap(),
            query_config.config.seed_liquidity_amplifier
        );
        assert_eq!(
            Decimal::from_atomics(22_u128, 1).unwrap(),
            query_config.config.initial_odds_home
        );
        assert_eq!(
            Decimal::from_atomics(18_u128, 1).unwrap(),
            query_config.config.initial_odds_away
        );

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
            Decimal::from_atomics(1_76_u128, 2).unwrap(),
            query_market.market.home_odds
        );
        assert_eq!(
            Decimal::from_atomics(1_44_u128, 2).unwrap(),
            query_market.market.away_odds
        );
    }

    #[test]
    fn it_properly_updates_market_max_bet_risk_factor() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            - 60 * 5; // 5 minutes ago

        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![
                (
                    Addr::unchecked(ADMIN_ADDRESS),
                    coins(INITIAL_BALANCE, NATIVE_DENOM),
                ),
                (
                    MockApiBech32::new("neutron").addr_make(USER_A),
                    coins(INITIAL_BALANCE, NATIVE_DENOM),
                ),
            ],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                fee_spread_odds: Decimal::from_atomics(15_u128, 2).unwrap(), // 0.15
                max_bet_risk_factor: Decimal::from_atomics(35_u128, 1).unwrap(), // 3.5
                seed_liquidity_amplifier: Decimal::from_atomics(3_u128, 0).unwrap(), // 3
                initial_odds_home: Decimal::from_atomics(22_u128, 1).unwrap(), // 2.2
                initial_odds_away: Decimal::from_atomics(18_u128, 1).unwrap(), // 1.8
                start_timestamp,
            },
            coins(100_000_000, NATIVE_DENOM),
        )
        .unwrap();

        let query_config = blockchain_contract.query_config().unwrap();
        assert_eq!(
            Decimal::from_atomics(35_u128, 1).unwrap(),
            query_config.config.max_bet_risk_factor
        );

        let user_a = blockchain_contract.blockchain.api().addr_make(USER_A);

        let err = blockchain_contract
            .place_bet(
                &user_a,
                MarketResult::HOME,
                Decimal::from_atomics(1_91_u128, 2).unwrap(),
                None,
                &coins(15_000_000, NATIVE_DENOM),
            )
            .unwrap_err();
        assert_eq!(
            ContractError::MaxBetExceeded {},
            err.downcast::<ContractError>().unwrap()
        );

        let new_max_bet_risk_factor = Decimal::from_atomics(15_u128, 1).unwrap(); // 1.5

        blockchain_contract
            .update_market(
                &Addr::unchecked(ADMIN_ADDRESS),
                UpdateParams {
                    start_timestamp: None,
                    fee_spread_odds: None,
                    max_bet_risk_factor: Some(new_max_bet_risk_factor),
                    seed_liquidity_amplifier: None,
                    initial_odds_home: None,
                    initial_odds_away: None,
                },
            )
            .unwrap();

        let err = blockchain_contract
            .place_bet(
                &user_a,
                MarketResult::HOME,
                Decimal::from_atomics(1_91_u128, 2).unwrap(),
                None,
                &coins(34_910_000, NATIVE_DENOM),
            )
            .unwrap_err();
        assert_eq!(
            ContractError::MaxBetExceeded {},
            err.downcast::<ContractError>().unwrap()
        );

        blockchain_contract
            .place_bet(
                &user_a,
                MarketResult::HOME,
                Decimal::from_atomics(1_91_u128, 2).unwrap(),
                None,
                &coins(15_000_000, NATIVE_DENOM),
            )
            .unwrap();

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(15_000_000, query_bets.total_amounts.home);
        assert_eq!(0, query_bets.total_amounts.away);

        let query_config = blockchain_contract.query_config().unwrap();
        assert_eq!(ADMIN_ADDRESS, query_config.config.admin_addr.as_str());
        assert_eq!(TREASURY_ADDRESS, query_config.config.treasury_addr.as_str());
        assert_eq!(NATIVE_DENOM, query_config.config.denom.as_str());
        assert_eq!(
            Decimal::from_atomics(15_u128, 2).unwrap(),
            query_config.config.fee_spread_odds
        );
        assert_eq!(
            new_max_bet_risk_factor,
            query_config.config.max_bet_risk_factor
        );
        assert_eq!(
            Decimal::from_atomics(3_u128, 0).unwrap(),
            query_config.config.seed_liquidity_amplifier
        );
        assert_eq!(
            Decimal::from_atomics(22_u128, 1).unwrap(),
            query_config.config.initial_odds_home
        );
        assert_eq!(
            Decimal::from_atomics(18_u128, 1).unwrap(),
            query_config.config.initial_odds_away
        );

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
            Decimal::from_atomics(18_u128, 1).unwrap(),
            query_market.market.home_odds
        );
        assert_eq!(
            Decimal::from_atomics(1_64_u128, 2).unwrap(),
            query_market.market.away_odds
        );
    }

    #[test]
    fn it_properly_updates_market_seed_liquidity_amplifier() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            - 60 * 5; // 5 minutes ago

        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![
                (
                    Addr::unchecked(ADMIN_ADDRESS),
                    coins(INITIAL_BALANCE, NATIVE_DENOM),
                ),
                (
                    MockApiBech32::new("neutron").addr_make(USER_A),
                    coins(INITIAL_BALANCE, NATIVE_DENOM),
                ),
            ],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                fee_spread_odds: Decimal::from_atomics(15_u128, 2).unwrap(), // 0.15
                max_bet_risk_factor: Decimal::from_atomics(15_u128, 1).unwrap(), // 1.5
                seed_liquidity_amplifier: Decimal::from_atomics(3_u128, 0).unwrap(), // 3
                initial_odds_home: Decimal::from_atomics(22_u128, 1).unwrap(), // 2.2
                initial_odds_away: Decimal::from_atomics(18_u128, 1).unwrap(), // 1.8
                start_timestamp,
            },
            coins(100_000_000, NATIVE_DENOM),
        )
        .unwrap();

        let query_config = blockchain_contract.query_config().unwrap();
        assert_eq!(
            Decimal::from_atomics(3_u128, 0).unwrap(), // 3
            query_config.config.seed_liquidity_amplifier
        );

        let user_a = blockchain_contract.blockchain.api().addr_make(USER_A);

        blockchain_contract
            .place_bet(
                &user_a,
                MarketResult::HOME,
                Decimal::from_atomics(1_91_u128, 2).unwrap(),
                None,
                &coins(15_000_000, NATIVE_DENOM),
            )
            .unwrap();

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(15_000_000, query_bets.total_amounts.home);
        assert_eq!(0, query_bets.total_amounts.away);

        let new_seed_liquidity_amplifier = Decimal::from_atomics(5_u128, 0).unwrap(); // 5

        blockchain_contract
            .update_market(
                &Addr::unchecked(ADMIN_ADDRESS),
                UpdateParams {
                    start_timestamp: None,
                    fee_spread_odds: None,
                    max_bet_risk_factor: None,
                    seed_liquidity_amplifier: Some(new_seed_liquidity_amplifier),
                    initial_odds_home: None,
                    initial_odds_away: None,
                },
            )
            .unwrap();

        let query_config = blockchain_contract.query_config().unwrap();
        assert_eq!(ADMIN_ADDRESS, query_config.config.admin_addr.as_str());
        assert_eq!(TREASURY_ADDRESS, query_config.config.treasury_addr.as_str());
        assert_eq!(NATIVE_DENOM, query_config.config.denom.as_str());
        assert_eq!(
            Decimal::from_atomics(15_u128, 2).unwrap(),
            query_config.config.fee_spread_odds
        );
        assert_eq!(
            Decimal::from_atomics(15_u128, 1).unwrap(),
            query_config.config.max_bet_risk_factor
        );
        assert_eq!(
            new_seed_liquidity_amplifier,
            query_config.config.seed_liquidity_amplifier
        );
        assert_eq!(
            Decimal::from_atomics(22_u128, 1).unwrap(),
            query_config.config.initial_odds_home
        );
        assert_eq!(
            Decimal::from_atomics(18_u128, 1).unwrap(),
            query_config.config.initial_odds_away
        );

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
            Decimal::from_atomics(1_84_u128, 2).unwrap(),
            query_market.market.home_odds
        );
        assert_eq!(
            Decimal::from_atomics(1_61_u128, 2).unwrap(),
            query_market.market.away_odds
        );
    }

    #[test]
    fn it_properly_updates_market_initial_odds() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            - 60 * 5; // 5 minutes ago

        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![(
                Addr::unchecked(ADMIN_ADDRESS),
                coins(INITIAL_BALANCE, NATIVE_DENOM),
            )],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                fee_spread_odds: Decimal::from_atomics(15_u128, 2).unwrap(), // 0.15
                max_bet_risk_factor: Decimal::from_atomics(15_u128, 1).unwrap(), // 1.5
                seed_liquidity_amplifier: Decimal::from_atomics(3_u128, 0).unwrap(), // 3
                initial_odds_home: Decimal::from_atomics(22_u128, 1).unwrap(), // 2.2
                initial_odds_away: Decimal::from_atomics(18_u128, 1).unwrap(), // 1.8
                start_timestamp,
            },
            coins(100_000_000, NATIVE_DENOM),
        )
        .unwrap();

        let query_config = blockchain_contract.query_config().unwrap();
        assert_eq!(
            Decimal::from_atomics(22_u128, 1).unwrap(),
            query_config.config.initial_odds_home
        );
        assert_eq!(
            Decimal::from_atomics(18_u128, 1).unwrap(),
            query_config.config.initial_odds_away
        );

        let new_initial_odds_home = Decimal::from_atomics(17_u128, 1).unwrap(); // 1.7
        let new_initial_odds_away = Decimal::from_atomics(23_u128, 1).unwrap(); // 2.3

        blockchain_contract
            .update_market(
                &Addr::unchecked(ADMIN_ADDRESS),
                UpdateParams {
                    start_timestamp: None,
                    fee_spread_odds: None,
                    max_bet_risk_factor: None,
                    seed_liquidity_amplifier: None,
                    initial_odds_home: Some(new_initial_odds_home),
                    initial_odds_away: Some(new_initial_odds_away),
                },
            )
            .unwrap();

        let query_config = blockchain_contract.query_config().unwrap();
        assert_eq!(ADMIN_ADDRESS, query_config.config.admin_addr.as_str());
        assert_eq!(TREASURY_ADDRESS, query_config.config.treasury_addr.as_str());
        assert_eq!(NATIVE_DENOM, query_config.config.denom.as_str());
        assert_eq!(
            Decimal::from_atomics(15_u128, 2).unwrap(),
            query_config.config.fee_spread_odds
        );
        assert_eq!(
            Decimal::from_atomics(15_u128, 1).unwrap(),
            query_config.config.max_bet_risk_factor
        );
        assert_eq!(
            Decimal::from_atomics(3_u128, 0).unwrap(),
            query_config.config.seed_liquidity_amplifier
        );
        assert_eq!(new_initial_odds_home, query_config.config.initial_odds_home);
        assert_eq!(new_initial_odds_away, query_config.config.initial_odds_away);

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
            Decimal::from_atomics(1_47_u128, 2).unwrap(),
            query_market.market.home_odds
        );
        assert_eq!(
            Decimal::from_atomics(2_u128, 0).unwrap(),
            query_market.market.away_odds
        );
    }

    #[test]
    fn unauthorized() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            - 60 * 5; // 5 minutes ago

        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![(
                Addr::unchecked(ADMIN_ADDRESS),
                coins(INITIAL_BALANCE, NATIVE_DENOM),
            )],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                fee_spread_odds: Decimal::from_atomics(15_u128, 2).unwrap(), // 0.15
                max_bet_risk_factor: Decimal::from_atomics(15_u128, 1).unwrap(), // 1.5
                seed_liquidity_amplifier: Decimal::from_atomics(3_u128, 0).unwrap(), // 3
                initial_odds_home: Decimal::from_atomics(22_u128, 1).unwrap(), // 2.2
                initial_odds_away: Decimal::from_atomics(18_u128, 1).unwrap(), // 1.8
                start_timestamp,
            },
            coins(100_000_000, NATIVE_DENOM),
        )
        .unwrap();

        let query_market = blockchain_contract.query_market().unwrap();
        assert_eq!(start_timestamp, query_market.market.start_timestamp);

        let anyone = blockchain_contract.blockchain.api().addr_make(ANYONE);

        let err = blockchain_contract
            .update_market(
                &anyone,
                UpdateParams {
                    start_timestamp: Some(start_timestamp - 60 * 30), // 30 minutes ago
                    fee_spread_odds: None,
                    max_bet_risk_factor: None,
                    seed_liquidity_amplifier: None,
                    initial_odds_home: None,
                    initial_odds_away: None,
                },
            )
            .unwrap_err();
        assert_eq!(
            ContractError::Unauthorized {},
            err.downcast::<ContractError>().unwrap()
        );

        let query_market = blockchain_contract.query_market().unwrap();
        assert_eq!(start_timestamp, query_market.market.start_timestamp);
    }

    #[test]
    fn market_not_active() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            - 60 * 10; // 10 minutes ago

        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![(
                Addr::unchecked(ADMIN_ADDRESS),
                coins(INITIAL_BALANCE, NATIVE_DENOM),
            )],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                fee_spread_odds: Decimal::from_atomics(15_u128, 2).unwrap(), // 0.15
                max_bet_risk_factor: Decimal::from_atomics(15_u128, 1).unwrap(), // 1.5
                seed_liquidity_amplifier: Decimal::from_atomics(3_u128, 0).unwrap(), // 3
                initial_odds_home: Decimal::from_atomics(22_u128, 1).unwrap(), // 2.2
                initial_odds_away: Decimal::from_atomics(18_u128, 1).unwrap(), // 1.8
                start_timestamp,
            },
            coins(100_000_000, NATIVE_DENOM),
        )
        .unwrap();

        blockchain_contract
            .cancel_market(&Addr::unchecked(ADMIN_ADDRESS))
            .unwrap();

        let query_market = blockchain_contract.query_market().unwrap();
        assert_ne!(Status::ACTIVE, query_market.market.status);

        let err = blockchain_contract
            .update_market(
                &Addr::unchecked(ADMIN_ADDRESS),
                UpdateParams {
                    start_timestamp: Some(start_timestamp - 60 * 30), // 30 minutes ago
                    fee_spread_odds: None,
                    max_bet_risk_factor: None,
                    seed_liquidity_amplifier: None,
                    initial_odds_home: None,
                    initial_odds_away: None,
                },
            )
            .unwrap_err();
        assert_eq!(
            ContractError::MarketNotActive {},
            err.downcast::<ContractError>().unwrap()
        );

        let query_market = blockchain_contract.query_market().unwrap();
        assert_eq!(start_timestamp, query_market.market.start_timestamp);
    }
}

mod score_market {
    use super::*;

    #[test]
    fn proper_score_market_and_collect_fees() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 5; // 5 minutes from now

        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![(
                Addr::unchecked(ADMIN_ADDRESS),
                coins(INITIAL_BALANCE, NATIVE_DENOM),
            )],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                fee_spread_odds: Decimal::from_atomics(15_u128, 2).unwrap(), // 0.15
                max_bet_risk_factor: Decimal::from_atomics(15_u128, 1).unwrap(), // 1.5
                seed_liquidity_amplifier: Decimal::from_atomics(3_u128, 0).unwrap(), // 3
                initial_odds_home: Decimal::from_atomics(22_u128, 1).unwrap(), // 2.2
                initial_odds_away: Decimal::from_atomics(18_u128, 1).unwrap(), // 1.8
                start_timestamp,
            },
            coins(100_000_000, NATIVE_DENOM),
        )
        .unwrap();

        blockchain_contract
            .place_bet(
                &Addr::unchecked(ADMIN_ADDRESS),
                MarketResult::HOME,
                Decimal::from_atomics(1_91_u128, 2).unwrap(),
                None,
                &coins(10_000_000, NATIVE_DENOM),
            )
            .unwrap();

        blockchain_contract
            .place_bet(
                &Addr::unchecked(ADMIN_ADDRESS),
                MarketResult::AWAY,
                Decimal::from_atomics(1_61_u128, 2).unwrap(),
                None,
                &coins(10_000_000, NATIVE_DENOM),
            )
            .unwrap();

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(10_000_000, query_bets.total_amounts.home);
        assert_eq!(10_000_000, query_bets.total_amounts.away);
        assert_eq!(19_100_000, query_bets.potential_payouts.home);
        assert_eq!(16_100_000, query_bets.potential_payouts.away);

        blockchain_contract.blockchain.update_block(|block| {
            block.time = Timestamp::from_seconds(
                start_timestamp + 60 * 30, // 30 minutes after the start timestamp
            );
        });

        let treasury_balance = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(Addr::unchecked(TREASURY_ADDRESS), NATIVE_DENOM)
            .unwrap();
        assert_eq!(0_u128, treasury_balance.amount.into());

        blockchain_contract
            .score_market(&Addr::unchecked(ADMIN_ADDRESS), MarketResult::AWAY)
            .unwrap();

        let treasury_balance = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(Addr::unchecked(TREASURY_ADDRESS), NATIVE_DENOM)
            .unwrap();
        assert_eq!(103_900_000_u128, treasury_balance.amount.into());

        let query_market = blockchain_contract.query_market().unwrap();
        assert_eq!(Status::CLOSED, query_market.market.status);
        assert_eq!(MarketResult::AWAY, query_market.market.result.unwrap());
    }

    #[test]
    fn unauthorized() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 5; // 5 minutes from now

        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![(
                Addr::unchecked(ADMIN_ADDRESS),
                coins(INITIAL_BALANCE, NATIVE_DENOM),
            )],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                fee_spread_odds: Decimal::from_atomics(15_u128, 2).unwrap(), // 0.15
                max_bet_risk_factor: Decimal::from_atomics(15_u128, 1).unwrap(), // 1.5
                seed_liquidity_amplifier: Decimal::from_atomics(3_u128, 0).unwrap(), // 3
                initial_odds_home: Decimal::from_atomics(22_u128, 1).unwrap(), // 2.2
                initial_odds_away: Decimal::from_atomics(18_u128, 1).unwrap(), // 1.8
                start_timestamp,
            },
            coins(100_000_000, NATIVE_DENOM),
        )
        .unwrap();

        blockchain_contract.blockchain.update_block(|block| {
            block.time = Timestamp::from_seconds(
                start_timestamp + 60 * 30, // 30 minutes after the start timestamp
            );
        });

        let err = blockchain_contract
            .score_market(
                &blockchain_contract.blockchain.api().addr_make(ANYONE),
                MarketResult::HOME,
            )
            .unwrap_err();
        assert_eq!(
            ContractError::Unauthorized {},
            err.downcast::<ContractError>().unwrap()
        );

        let query_market = blockchain_contract.query_market().unwrap();
        assert_eq!(Status::ACTIVE, query_market.market.status);
        assert_eq!(None, query_market.market.result);
    }

    #[test]
    fn market_not_active() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 5; // 5 minutes from now

        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![(
                Addr::unchecked(ADMIN_ADDRESS),
                coins(INITIAL_BALANCE, NATIVE_DENOM),
            )],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                fee_spread_odds: Decimal::from_atomics(15_u128, 2).unwrap(), // 0.15
                max_bet_risk_factor: Decimal::from_atomics(15_u128, 1).unwrap(), // 1.5
                seed_liquidity_amplifier: Decimal::from_atomics(3_u128, 0).unwrap(), // 3
                initial_odds_home: Decimal::from_atomics(22_u128, 1).unwrap(), // 2.2
                initial_odds_away: Decimal::from_atomics(18_u128, 1).unwrap(), // 1.8
                start_timestamp,
            },
            coins(100_000_000, NATIVE_DENOM),
        )
        .unwrap();

        blockchain_contract
            .cancel_market(&Addr::unchecked(ADMIN_ADDRESS))
            .unwrap();

        let query_market = blockchain_contract.query_market().unwrap();
        assert_ne!(Status::ACTIVE, query_market.market.status);
        assert_eq!(None, query_market.market.result);

        let err = blockchain_contract
            .score_market(&Addr::unchecked(ADMIN_ADDRESS), MarketResult::HOME)
            .unwrap_err();
        assert_eq!(
            ContractError::MarketNotActive {},
            err.downcast::<ContractError>().unwrap()
        );

        let query_market = blockchain_contract.query_market().unwrap();
        assert_eq!(None, query_market.market.result);
    }

    #[test]
    fn market_not_scoreable() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 5; // 5 minutes from now

        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![(
                Addr::unchecked(ADMIN_ADDRESS),
                coins(INITIAL_BALANCE, NATIVE_DENOM),
            )],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                fee_spread_odds: Decimal::from_atomics(15_u128, 2).unwrap(), // 0.15
                max_bet_risk_factor: Decimal::from_atomics(15_u128, 1).unwrap(), // 1.5
                seed_liquidity_amplifier: Decimal::from_atomics(3_u128, 0).unwrap(), // 3
                initial_odds_home: Decimal::from_atomics(22_u128, 1).unwrap(), // 2.2
                initial_odds_away: Decimal::from_atomics(18_u128, 1).unwrap(), // 1.8
                start_timestamp,
            },
            coins(100_000_000, NATIVE_DENOM),
        )
        .unwrap();

        blockchain_contract.blockchain.update_block(|block| {
            block.time = Timestamp::from_seconds(
                start_timestamp + 60 * 30 - 1, // 29 minutes and 59 seconds from start timestamp
            );
        });

        let err = blockchain_contract
            .score_market(&Addr::unchecked(ADMIN_ADDRESS), MarketResult::HOME)
            .unwrap_err();
        assert_eq!(
            ContractError::MarketNotScoreable {},
            err.downcast::<ContractError>().unwrap()
        );

        let query_market = blockchain_contract.query_market().unwrap();
        assert_eq!(Status::ACTIVE, query_market.market.status);
        assert_eq!(None, query_market.market.result);

        blockchain_contract.blockchain.update_block(|block| {
            block.time = Timestamp::from_seconds(
                start_timestamp + 60 * 30, // 30 minutes from start timestamp
            );
        });

        blockchain_contract
            .score_market(&Addr::unchecked(ADMIN_ADDRESS), MarketResult::HOME)
            .unwrap();

        let query_market = blockchain_contract.query_market().unwrap();
        assert_eq!(Status::CLOSED, query_market.market.status);
        assert_eq!(MarketResult::HOME, query_market.market.result.unwrap());
    }
}

mod cancel_market {
    use super::*;

    #[test]
    fn proper_cancel_market() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 5; // 5 minutes from now

        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![(
                Addr::unchecked(ADMIN_ADDRESS),
                coins(INITIAL_BALANCE, NATIVE_DENOM),
            )],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                fee_spread_odds: Decimal::from_atomics(15_u128, 2).unwrap(), // 0.15
                max_bet_risk_factor: Decimal::from_atomics(15_u128, 1).unwrap(), // 1.5
                seed_liquidity_amplifier: Decimal::from_atomics(3_u128, 0).unwrap(), // 3
                initial_odds_home: Decimal::from_atomics(22_u128, 1).unwrap(), // 2.2
                initial_odds_away: Decimal::from_atomics(18_u128, 1).unwrap(), // 1.8
                start_timestamp,
            },
            coins(100_000_000, NATIVE_DENOM),
        )
        .unwrap();

        let query_market = blockchain_contract.query_market().unwrap();
        assert_eq!(Status::ACTIVE, query_market.market.status);

        blockchain_contract
            .cancel_market(&Addr::unchecked(ADMIN_ADDRESS))
            .unwrap();

        let query_market = blockchain_contract.query_market().unwrap();
        assert_eq!(Status::CANCELLED, query_market.market.status);
    }

    #[test]
    fn unauthorized() {
        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![(
                Addr::unchecked(ADMIN_ADDRESS),
                coins(INITIAL_BALANCE, NATIVE_DENOM),
            )],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                fee_spread_odds: Decimal::from_atomics(15_u128, 2).unwrap(), // 0.15
                max_bet_risk_factor: Decimal::from_atomics(15_u128, 1).unwrap(), // 1.5
                seed_liquidity_amplifier: Decimal::from_atomics(3_u128, 0).unwrap(), // 3
                initial_odds_home: Decimal::from_atomics(22_u128, 1).unwrap(), // 2.2
                initial_odds_away: Decimal::from_atomics(18_u128, 1).unwrap(), // 1.8
                start_timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs()
                    + 60 * 5, // 5 minutes from now
            },
            coins(100_000_000, NATIVE_DENOM),
        )
        .unwrap();

        let query_market = blockchain_contract.query_market().unwrap();
        assert_eq!(Status::ACTIVE, query_market.market.status);

        let anyone = blockchain_contract.blockchain.api().addr_make(ANYONE);
        let err = blockchain_contract.cancel_market(&anyone).unwrap_err();
        assert_eq!(
            ContractError::Unauthorized {},
            err.downcast::<ContractError>().unwrap()
        );

        let query_market = blockchain_contract.query_market().unwrap();
        assert_ne!(Status::CANCELLED, query_market.market.status);
    }

    #[test]
    fn market_not_active() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 5; // 5 minutes from now

        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![(
                Addr::unchecked(ADMIN_ADDRESS),
                coins(INITIAL_BALANCE, NATIVE_DENOM),
            )],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                fee_spread_odds: Decimal::from_atomics(15_u128, 2).unwrap(), // 0.15
                max_bet_risk_factor: Decimal::from_atomics(15_u128, 1).unwrap(), // 1.5
                seed_liquidity_amplifier: Decimal::from_atomics(3_u128, 0).unwrap(), // 3
                initial_odds_home: Decimal::from_atomics(22_u128, 1).unwrap(), // 2.2
                initial_odds_away: Decimal::from_atomics(18_u128, 1).unwrap(), // 1.8
                start_timestamp,
            },
            coins(100_000_000, NATIVE_DENOM),
        )
        .unwrap();

        blockchain_contract.blockchain.update_block(|block| {
            block.time = Timestamp::from_seconds(
                start_timestamp + 60 * 45, // 45 minutes after the start timestamp
            );
        });

        blockchain_contract
            .score_market(&Addr::unchecked(ADMIN_ADDRESS), MarketResult::HOME)
            .unwrap();

        let query_market = blockchain_contract.query_market().unwrap();
        assert_ne!(Status::ACTIVE, query_market.market.status);

        let err = blockchain_contract
            .cancel_market(&Addr::unchecked(ADMIN_ADDRESS))
            .unwrap_err();
        assert_eq!(
            ContractError::MarketNotActive {},
            err.downcast::<ContractError>().unwrap()
        );

        let query_market = blockchain_contract.query_market().unwrap();
        assert_ne!(Status::CANCELLED, query_market.market.status);
    }
}
