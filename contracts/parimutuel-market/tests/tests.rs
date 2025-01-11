use cosmwasm_std::{coin, coins, Addr, Timestamp, Uint128};
use cw_multi_test::MockApiBech32;
use helpers::setup_blockchain_and_contract;
use parimutuel_market::{
    contract::{ADMIN_ADDRESS, TREASURY_ADDRESS},
    error::ContractError,
    msg::InstantiateMsg,
    state::{MarketResult, Status},
};
use std::time::{SystemTime, UNIX_EPOCH};

mod helpers;

const NATIVE_DENOM: &str = "denom";
const NATIVE_DENOM_PRECISION: u32 = 6;
const FAKE_DENOM: &str = "fakedenom";
const OTHER: &str = "USER_OTHER";
const ANYONE: &str = "USER_ANYONE";
const USER_A: &str = "USER_A";
const USER_B: &str = "USER_B";
const USER_C: &str = "USER_C";
const DEFAULT_FEE_BPS: u64 = 250;
const INITIAL_BALANCE: u128 = 1_000_000_000;

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
                fee_bps: DEFAULT_FEE_BPS,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            },
            vec![],
        )
        .unwrap();

        let query_config = blockchain_contract.query_config().unwrap();
        assert_eq!(250, query_config.config.fee_bps);
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
        assert!(query_market.market.is_drawable);
        assert_eq!(Status::ACTIVE, query_market.market.status);
        assert_eq!(None, query_market.market.result);
    }

    #[test]
    fn unauthorized() {
        let blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(USER_A),
            vec![],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                fee_bps: DEFAULT_FEE_BPS,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs()
                    + 60 * 5, // 5 minutes from now
                is_drawable: true,
            },
            vec![],
        )
        .unwrap_err();

        assert_eq!(
            ContractError::Unauthorized {},
            blockchain_contract.downcast::<ContractError>().unwrap()
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
                    vec![
                        coin(INITIAL_BALANCE, NATIVE_DENOM),
                        coin(INITIAL_BALANCE, FAKE_DENOM),
                    ],
                ),
            ],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                fee_bps: DEFAULT_FEE_BPS,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            },
            vec![],
        )
        .unwrap();

        let user_a = blockchain_contract.blockchain.api().addr_make(USER_A);

        blockchain_contract
            .place_bet(
                &user_a,
                MarketResult::HOME,
                None,
                &coins(100_762_000, NATIVE_DENOM),
            )
            .unwrap();

        blockchain_contract
            .place_bet(
                &user_a,
                MarketResult::DRAW,
                None,
                &coins(340_228_000, NATIVE_DENOM),
            )
            .unwrap();

        let user_b = blockchain_contract.blockchain.api().addr_make(USER_B);

        blockchain_contract
            .place_bet(
                &user_b,
                MarketResult::AWAY,
                None,
                &coins(200_505_000, NATIVE_DENOM),
            )
            .unwrap();

        blockchain_contract
            .place_bet(
                &user_b,
                MarketResult::HOME,
                None,
                &coins(2_505_000, NATIVE_DENOM),
            )
            .unwrap();

        let user_c = blockchain_contract.blockchain.api().addr_make(USER_C);

        blockchain_contract
            .place_bet(
                &user_c,
                MarketResult::DRAW,
                None,
                &coins(300_029_300, NATIVE_DENOM),
            )
            .unwrap();

        let err = blockchain_contract
            .place_bet(
                &user_c,
                MarketResult::DRAW,
                None,
                &coins(300_029_300, FAKE_DENOM),
            )
            .unwrap_err();
        assert_eq!(
            ContractError::PaymentError {},
            err.downcast::<ContractError>().unwrap()
        );

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(query_bets.totals.home, 103_267_000);
        assert_eq!(query_bets.totals.away, 200_505_000);
        assert_eq!(query_bets.totals.draw, 640_257_300);

        let total_bets = 103_267_000 + 200_505_000 + 640_257_300;
        assert_eq!(
            total_bets,
            query_bets.totals.home + query_bets.totals.away + query_bets.totals.draw
        );

        let query_user_a_bets = blockchain_contract.query_bets_by_address(&user_a).unwrap();
        assert_eq!(query_user_a_bets.totals.home, 100_762_000);
        assert_eq!(query_user_a_bets.totals.away, 0);
        assert_eq!(query_user_a_bets.totals.draw, 340_228_000);

        let query_user_b_bets = blockchain_contract.query_bets_by_address(&user_b).unwrap();
        assert_eq!(query_user_b_bets.totals.home, 2_505_000);
        assert_eq!(query_user_b_bets.totals.away, 200_505_000);
        assert_eq!(query_user_b_bets.totals.draw, 0);

        let query_user_c_bets = blockchain_contract.query_bets_by_address(&user_c).unwrap();
        assert_eq!(query_user_c_bets.totals.home, 0);
        assert_eq!(query_user_c_bets.totals.away, 0);
        assert_eq!(query_user_c_bets.totals.draw, 300_029_300);

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
        assert_eq!(total_bets, market_balance.amount.into());

        blockchain_contract
            .score_market(&Addr::unchecked(ADMIN_ADDRESS), MarketResult::DRAW)
            .unwrap();

        let query_market = blockchain_contract.query_market().unwrap();
        assert_eq!(query_market.market.status, Status::CLOSED);
        assert_eq!(query_market.market.result.unwrap(), MarketResult::DRAW);

        let treasury_balance = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(Addr::unchecked(TREASURY_ADDRESS), NATIVE_DENOM)
            .unwrap();
        let fee_amount = Uint128::from(total_bets)
            .multiply_ratio(Uint128::from(DEFAULT_FEE_BPS), Uint128::from(10000_u128));
        assert_eq!(fee_amount, treasury_balance.amount);

        let market_balance_before = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(blockchain_contract.addr().clone(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(
            Uint128::new(total_bets) - fee_amount,
            market_balance_before.amount
        );

        let user_a_balance_before = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(user_a.clone(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(Uint128::new(559_010_000), user_a_balance_before.amount);

        blockchain_contract.claim_winnings(&user_a, None).unwrap();

        let user_a_balance_after = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(user_a.clone(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(
            user_a_balance_before.amount + Uint128::new(489_108_942),
            user_a_balance_after.amount
        );

        let user_b_balance_before = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(user_b.clone(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(Uint128::new(796_990_000), user_b_balance_before.amount);

        let err = blockchain_contract
            .claim_winnings(&user_b, None)
            .unwrap_err();
        assert_eq!(
            ContractError::NoWinnings {},
            err.downcast::<ContractError>().unwrap()
        );

        let user_b_balance_after = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(user_b.clone(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(user_b_balance_before.amount, user_b_balance_after.amount);

        let user_c_balance_before = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(user_c.clone(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(Uint128::new(699_970_700), user_c_balance_before.amount);

        blockchain_contract.claim_winnings(&user_c, None).unwrap();

        let user_c_balance_after = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(user_c.clone(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(
            user_c_balance_before.amount + Uint128::new(431_319_625),
            user_c_balance_after.amount
        );

        let market_balance_after = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(blockchain_contract.addr().clone(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(Uint128::new(1), market_balance_after.amount);
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
                fee_bps: DEFAULT_FEE_BPS,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            },
            vec![],
        )
        .unwrap();

        let user_a = blockchain_contract.blockchain.api().addr_make(USER_A);

        let user_b = blockchain_contract.blockchain.api().addr_make(USER_B);

        blockchain_contract
            .place_bet(
                &user_a,
                MarketResult::AWAY,
                Some(user_b.clone()),
                &coins(1_000, NATIVE_DENOM),
            )
            .unwrap();

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(1_000, query_bets.totals.away);

        let query_user_a_bets = blockchain_contract.query_bets_by_address(&user_a).unwrap();
        assert_eq!(0, query_user_a_bets.totals.away);

        let query_user_b_bets = blockchain_contract.query_bets_by_address(&user_b).unwrap();
        assert_eq!(1_000, query_user_b_bets.totals.away);
    }

    #[test]
    fn it_cant_place_bet_on_draw_when_market_isnt_drawable() {
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
                fee_bps: DEFAULT_FEE_BPS,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: false,
            },
            vec![],
        )
        .unwrap();

        let user_a = blockchain_contract.blockchain.api().addr_make(USER_A);

        let err = blockchain_contract
            .place_bet(
                &user_a,
                MarketResult::DRAW,
                None,
                &coins(1_000, NATIVE_DENOM),
            )
            .unwrap_err();
        assert_eq!(
            ContractError::MarketNotDrawable {},
            err.downcast::<ContractError>().unwrap()
        );

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(0, query_bets.totals.draw);
        assert_eq!(0, query_bets.totals.away);
        assert_eq!(0, query_bets.totals.home);

        let query_user_a_bets = blockchain_contract.query_bets_by_address(&user_a).unwrap();
        assert_eq!(0, query_user_a_bets.totals.draw);
        assert_eq!(0, query_user_a_bets.totals.away);
        assert_eq!(0, query_user_a_bets.totals.home);
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
                fee_bps: DEFAULT_FEE_BPS,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            },
            vec![],
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
                MarketResult::DRAW,
                None,
                &coins(1_000, NATIVE_DENOM),
            )
            .unwrap_err();
        assert_eq!(
            ContractError::MarketNotActive {},
            err.downcast::<ContractError>().unwrap()
        );

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(0, query_bets.totals.draw);
        assert_eq!(0, query_bets.totals.away);
        assert_eq!(0, query_bets.totals.home);

        let query_user_a_bets = blockchain_contract.query_bets_by_address(&user_a).unwrap();
        assert_eq!(0, query_user_a_bets.totals.draw);
        assert_eq!(0, query_user_a_bets.totals.away);
        assert_eq!(0, query_user_a_bets.totals.home);
    }

    #[test]
    fn it_can_only_place_bets_up_until_5_minutes_before_market_start_timestamp() {
        let block_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        let start_timestamp = block_timestamp + 60 * 5 - 1; // 4 minutes and 59 seconds from now

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
                fee_bps: DEFAULT_FEE_BPS,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            },
            vec![],
        )
        .unwrap();

        blockchain_contract.blockchain.update_block(|block| {
            block.time = Timestamp::from_seconds(start_timestamp);
        });

        let user_a = blockchain_contract.blockchain.api().addr_make(USER_A);

        let err = blockchain_contract
            .place_bet(
                &user_a,
                MarketResult::AWAY,
                None,
                &coins(1_000, NATIVE_DENOM),
            )
            .unwrap_err();
        assert_eq!(
            ContractError::BetsNotAccepted {},
            err.downcast::<ContractError>().unwrap()
        );

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(0, query_bets.totals.draw);
        assert_eq!(0, query_bets.totals.away);
        assert_eq!(0, query_bets.totals.home);

        let query_user_a_bets = blockchain_contract.query_bets_by_address(&user_a).unwrap();
        assert_eq!(0, query_user_a_bets.totals.draw);
        assert_eq!(0, query_user_a_bets.totals.away);
        assert_eq!(0, query_user_a_bets.totals.home);
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
                fee_bps: DEFAULT_FEE_BPS,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            },
            vec![],
        )
        .unwrap();

        let user_a = blockchain_contract.blockchain.api().addr_make(USER_A);

        let err = blockchain_contract
            .place_bet(&user_a, MarketResult::AWAY, None, &[])
            .unwrap_err();
        assert_eq!(
            ContractError::PaymentError {},
            err.downcast::<ContractError>().unwrap()
        );

        let err = blockchain_contract
            .place_bet(
                &user_a,
                MarketResult::AWAY,
                None,
                &coins(1_000_000, FAKE_DENOM),
            )
            .unwrap_err();
        assert_eq!(
            ContractError::PaymentError {},
            err.downcast::<ContractError>().unwrap()
        );

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(0, query_bets.totals.draw);
        assert_eq!(0, query_bets.totals.away);
        assert_eq!(0, query_bets.totals.home);

        let query_user_a_bets = blockchain_contract.query_bets_by_address(&user_a).unwrap();
        assert_eq!(0, query_user_a_bets.totals.draw);
        assert_eq!(0, query_user_a_bets.totals.away);
        assert_eq!(0, query_user_a_bets.totals.home);
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
            - 60 * 10; // 10 minutes ago

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
                fee_bps: DEFAULT_FEE_BPS,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            },
            vec![],
        )
        .unwrap();

        let user_a = blockchain_contract.blockchain.api().addr_make(USER_A);
        blockchain_contract
            .place_bet(
                &user_a,
                MarketResult::AWAY,
                None,
                &coins(1_000, NATIVE_DENOM),
            )
            .unwrap();

        let other = blockchain_contract.blockchain.api().addr_make(OTHER);
        blockchain_contract
            .place_bet(
                &other,
                MarketResult::DRAW,
                None,
                &coins(1_000, NATIVE_DENOM),
            )
            .unwrap();

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(1_000, query_bets.totals.draw);
        assert_eq!(1_000, query_bets.totals.away);
        assert_eq!(0, query_bets.totals.home);

        let user_a_winnings = blockchain_contract
            .query_estimate_winnings(&user_a, MarketResult::AWAY)
            .unwrap();
        assert_eq!(2_000, user_a_winnings.estimate);

        let other_winnings = blockchain_contract
            .query_estimate_winnings(&other, MarketResult::AWAY)
            .unwrap();
        assert_eq!(0, other_winnings.estimate);

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

        let user_a_balance = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(&user_a, NATIVE_DENOM)
            .unwrap();
        assert_eq!(INITIAL_BALANCE - 1_000_u128, user_a_balance.amount.into());

        blockchain_contract.claim_winnings(&user_a, None).unwrap();

        let user_a_balance = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(&user_a, NATIVE_DENOM)
            .unwrap();
        assert_eq!(INITIAL_BALANCE + 950_u128, user_a_balance.amount.into());
    }

    #[test]
    fn it_can_claim_on_behalf_of_the_receiver_when_defined() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            - 60 * 10; // 10 minutes ago

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
                fee_bps: DEFAULT_FEE_BPS,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            },
            vec![],
        )
        .unwrap();

        let user_a = blockchain_contract.blockchain.api().addr_make(USER_A);
        blockchain_contract
            .place_bet(
                &user_a,
                MarketResult::HOME,
                None,
                &coins(1_000, NATIVE_DENOM),
            )
            .unwrap();

        let other = blockchain_contract.blockchain.api().addr_make(OTHER);
        blockchain_contract
            .place_bet(
                &other,
                MarketResult::AWAY,
                None,
                &coins(1_000, NATIVE_DENOM),
            )
            .unwrap();

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(1_000, query_bets.totals.home);
        assert_eq!(1_000, query_bets.totals.away);
        assert_eq!(0, query_bets.totals.draw);

        let user_a_winnings = blockchain_contract
            .query_estimate_winnings(&user_a, MarketResult::HOME)
            .unwrap();
        assert_eq!(2_000, user_a_winnings.estimate);

        let other_winnings = blockchain_contract
            .query_estimate_winnings(&other, MarketResult::HOME)
            .unwrap();
        assert_eq!(0, other_winnings.estimate);

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
        assert_eq!(INITIAL_BALANCE - 1_000_u128, user_a_balance.amount.into());

        let anyone = blockchain_contract.blockchain.api().addr_make(ANYONE);
        blockchain_contract
            .claim_winnings(&anyone, Some(user_a.clone()))
            .unwrap();

        let user_a_balance = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(&user_a, NATIVE_DENOM)
            .unwrap();
        assert_eq!(INITIAL_BALANCE + 950_u128, user_a_balance.amount.into());
    }

    #[test]
    fn it_will_return_all_bets_made_if_market_was_cancelled() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            - 60 * 10; // 10 minutes ago

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
                fee_bps: DEFAULT_FEE_BPS,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            },
            vec![],
        )
        .unwrap();

        let user_a = blockchain_contract.blockchain.api().addr_make(USER_A);
        blockchain_contract
            .place_bet(
                &user_a,
                MarketResult::DRAW,
                None,
                &coins(1_000, NATIVE_DENOM),
            )
            .unwrap();

        let other = blockchain_contract.blockchain.api().addr_make(OTHER);
        blockchain_contract
            .place_bet(
                &other,
                MarketResult::AWAY,
                None,
                &coins(1_000, NATIVE_DENOM),
            )
            .unwrap();

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(1_000, query_bets.totals.draw);
        assert_eq!(1_000, query_bets.totals.away);
        assert_eq!(0, query_bets.totals.home);

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
        assert_eq!(INITIAL_BALANCE - 1_000_u128, user_a_balance.amount.into());

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
        assert_eq!(INITIAL_BALANCE - 1_000_u128, other_balance.amount.into());

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
            - 60 * 10; // 10 minutes ago

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
                fee_bps: DEFAULT_FEE_BPS,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            },
            vec![],
        )
        .unwrap();

        let user_a = blockchain_contract.blockchain.api().addr_make(USER_A);
        blockchain_contract
            .place_bet(
                &user_a,
                MarketResult::DRAW,
                None,
                &coins(1_000, NATIVE_DENOM),
            )
            .unwrap();

        let other = blockchain_contract.blockchain.api().addr_make(OTHER);
        blockchain_contract
            .place_bet(
                &other,
                MarketResult::AWAY,
                None,
                &coins(1_000, NATIVE_DENOM),
            )
            .unwrap();

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(1_000, query_bets.totals.draw);
        assert_eq!(1_000, query_bets.totals.away);
        assert_eq!(0, query_bets.totals.home);

        let query_market = blockchain_contract.query_market().unwrap();
        assert_eq!(Status::ACTIVE, query_market.market.status);
        assert_eq!(None, query_market.market.result);

        let user_a_balance = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(&user_a, NATIVE_DENOM)
            .unwrap();
        assert_eq!(INITIAL_BALANCE - 1_000_u128, user_a_balance.amount.into());

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
        assert_eq!(INITIAL_BALANCE - 1_000_u128, user_a_balance.amount.into());
    }

    #[test]
    fn it_cant_claim_winnings_twice() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            - 60 * 10; // 10 minutes ago

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
                fee_bps: DEFAULT_FEE_BPS,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            },
            vec![],
        )
        .unwrap();

        let user_a = blockchain_contract.blockchain.api().addr_make(USER_A);
        blockchain_contract
            .place_bet(
                &user_a,
                MarketResult::DRAW,
                None,
                &coins(1_000, NATIVE_DENOM),
            )
            .unwrap();

        let other = blockchain_contract.blockchain.api().addr_make(OTHER);
        blockchain_contract
            .place_bet(
                &other,
                MarketResult::AWAY,
                None,
                &coins(1_000, NATIVE_DENOM),
            )
            .unwrap();

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(1_000, query_bets.totals.draw);
        assert_eq!(1_000, query_bets.totals.away);
        assert_eq!(0, query_bets.totals.home);

        blockchain_contract.blockchain.update_block(|block| {
            block.time = Timestamp::from_seconds(
                start_timestamp + 60 * 30, // 30 minutes after the start timestamp
            );
        });

        let user_a_winnings = blockchain_contract
            .query_estimate_winnings(&user_a, MarketResult::DRAW)
            .unwrap();
        assert_eq!(2_000, user_a_winnings.estimate);

        let other_winnings = blockchain_contract
            .query_estimate_winnings(&other, MarketResult::DRAW)
            .unwrap();
        assert_eq!(0, other_winnings.estimate);

        blockchain_contract
            .score_market(&Addr::unchecked(ADMIN_ADDRESS), MarketResult::DRAW)
            .unwrap();

        let query_market = blockchain_contract.query_market().unwrap();
        assert_eq!(Status::CLOSED, query_market.market.status);
        assert_eq!(MarketResult::DRAW, query_market.market.result.unwrap());

        let user_a_balance = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(&user_a, NATIVE_DENOM)
            .unwrap();
        assert_eq!(INITIAL_BALANCE - 1_000_u128, user_a_balance.amount.into());

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
        assert_eq!(INITIAL_BALANCE + 950_u128, user_a_balance.amount.into());
    }

    #[test]
    fn it_cant_claim_winnings_when_there_is_nothing_to_claim() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            - 60 * 10; // 10 minutes ago

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
                fee_bps: DEFAULT_FEE_BPS,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            },
            vec![],
        )
        .unwrap();

        let user_a = blockchain_contract.blockchain.api().addr_make(USER_A);
        blockchain_contract
            .place_bet(
                &user_a,
                MarketResult::DRAW,
                None,
                &coins(1_000, NATIVE_DENOM),
            )
            .unwrap();

        let other = blockchain_contract.blockchain.api().addr_make(OTHER);
        blockchain_contract
            .place_bet(
                &other,
                MarketResult::AWAY,
                None,
                &coins(1_000, NATIVE_DENOM),
            )
            .unwrap();

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(1_000, query_bets.totals.draw);
        assert_eq!(1_000, query_bets.totals.away);
        assert_eq!(0, query_bets.totals.home);

        blockchain_contract.blockchain.update_block(|block| {
            block.time = Timestamp::from_seconds(
                start_timestamp + 60 * 30, // 30 minutes after the start timestamp
            );
        });

        blockchain_contract
            .score_market(&Addr::unchecked(ADMIN_ADDRESS), MarketResult::DRAW)
            .unwrap();

        let query_market = blockchain_contract.query_market().unwrap();
        assert_eq!(Status::CLOSED, query_market.market.status);
        assert_eq!(MarketResult::DRAW, query_market.market.result.unwrap());

        let other_balance = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(&other, NATIVE_DENOM)
            .unwrap();
        assert_eq!(INITIAL_BALANCE - 1_000_u128, other_balance.amount.into());

        let err = blockchain_contract
            .claim_winnings(&other, None)
            .unwrap_err();
        assert_eq!(
            ContractError::NoWinnings {},
            err.downcast::<ContractError>().unwrap()
        );

        let other_balance = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(&other, NATIVE_DENOM)
            .unwrap();
        assert_eq!(INITIAL_BALANCE - 1_000_u128, other_balance.amount.into());
    }
}

mod update_market {
    use super::*;

    #[test]
    fn proper_update_market() {
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
                fee_bps: DEFAULT_FEE_BPS,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            },
            vec![],
        )
        .unwrap();

        let query_market = blockchain_contract.query_market().unwrap();
        assert_eq!(start_timestamp, query_market.market.start_timestamp);

        let new_start_timestamp = start_timestamp - 60 * 30; // 30 minutes ago
        blockchain_contract
            .update_market(&Addr::unchecked(ADMIN_ADDRESS), new_start_timestamp)
            .unwrap();

        let query_market = blockchain_contract.query_market().unwrap();
        assert_eq!(new_start_timestamp, query_market.market.start_timestamp);
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
                fee_bps: DEFAULT_FEE_BPS,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            },
            vec![],
        )
        .unwrap();

        let query_market = blockchain_contract.query_market().unwrap();
        assert_eq!(start_timestamp, query_market.market.start_timestamp);

        let anyone = blockchain_contract.blockchain.api().addr_make(ANYONE);
        let err = blockchain_contract
            .update_market(&anyone, start_timestamp - 60 * 30) // 30 minutes ago
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
                fee_bps: DEFAULT_FEE_BPS,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            },
            vec![],
        )
        .unwrap();

        blockchain_contract
            .place_bet(
                &Addr::unchecked(ADMIN_ADDRESS),
                MarketResult::DRAW,
                None,
                &coins(1_000, NATIVE_DENOM),
            )
            .unwrap();

        blockchain_contract
            .place_bet(
                &Addr::unchecked(ADMIN_ADDRESS),
                MarketResult::AWAY,
                None,
                &coins(1_000, NATIVE_DENOM),
            )
            .unwrap();

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(1_000, query_bets.totals.draw);
        assert_eq!(1_000, query_bets.totals.away);
        assert_eq!(0, query_bets.totals.home);

        blockchain_contract.blockchain.update_block(|block| {
            block.time = Timestamp::from_seconds(
                start_timestamp + 60 * 30, // 30 minutes after the start timestamp
            );
        });

        blockchain_contract
            .score_market(&Addr::unchecked(ADMIN_ADDRESS), MarketResult::DRAW)
            .unwrap();

        let query_market = blockchain_contract.query_market().unwrap();
        assert_ne!(Status::ACTIVE, query_market.market.status);

        let err = blockchain_contract
            .update_market(&Addr::unchecked(ADMIN_ADDRESS), start_timestamp - 60 * 30)
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
            + 60 * 10; // 10 minutes from now

        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![(
                Addr::unchecked(ADMIN_ADDRESS),
                coins(INITIAL_BALANCE, NATIVE_DENOM),
            )],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                fee_bps: DEFAULT_FEE_BPS,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            },
            vec![],
        )
        .unwrap();

        blockchain_contract
            .place_bet(
                &Addr::unchecked(ADMIN_ADDRESS),
                MarketResult::DRAW,
                None,
                &coins(1_000, NATIVE_DENOM),
            )
            .unwrap();

        blockchain_contract
            .place_bet(
                &Addr::unchecked(ADMIN_ADDRESS),
                MarketResult::AWAY,
                None,
                &coins(1_000, NATIVE_DENOM),
            )
            .unwrap();

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(1_000, query_bets.totals.draw);
        assert_eq!(1_000, query_bets.totals.away);
        assert_eq!(0, query_bets.totals.home);

        blockchain_contract.blockchain.update_block(|block| {
            block.time = Timestamp::from_seconds(
                start_timestamp + 60 * 30, // 30 minutes after the start timestamp
            );
        });

        blockchain_contract
            .score_market(&Addr::unchecked(ADMIN_ADDRESS), MarketResult::DRAW)
            .unwrap();

        let treasury_balance = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(Addr::unchecked(TREASURY_ADDRESS), NATIVE_DENOM)
            .unwrap();
        assert_eq!(50_u128, treasury_balance.amount.into());

        let query_market = blockchain_contract.query_market().unwrap();
        assert_eq!(Status::CLOSED, query_market.market.status);
        assert_eq!(MarketResult::DRAW, query_market.market.result.unwrap());
    }

    #[test]
    fn do_not_collect_fees_when_set_to_zero() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 10; // 10 minutes from now

        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![(
                Addr::unchecked(ADMIN_ADDRESS),
                coins(INITIAL_BALANCE, NATIVE_DENOM),
            )],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                fee_bps: 0,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            },
            vec![],
        )
        .unwrap();

        blockchain_contract
            .place_bet(
                &Addr::unchecked(ADMIN_ADDRESS),
                MarketResult::DRAW,
                None,
                &coins(1_000, NATIVE_DENOM),
            )
            .unwrap();

        blockchain_contract
            .place_bet(
                &Addr::unchecked(ADMIN_ADDRESS),
                MarketResult::AWAY,
                None,
                &coins(1_000, NATIVE_DENOM),
            )
            .unwrap();

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(1_000, query_bets.totals.draw);
        assert_eq!(1_000, query_bets.totals.away);
        assert_eq!(0, query_bets.totals.home);

        blockchain_contract.blockchain.update_block(|block| {
            block.time = Timestamp::from_seconds(
                start_timestamp + 60 * 30, // 30 minutes after the start timestamp
            );
        });

        blockchain_contract
            .score_market(&Addr::unchecked(ADMIN_ADDRESS), MarketResult::DRAW)
            .unwrap();

        let treasury_balance = blockchain_contract
            .blockchain
            .wrap()
            .query_balance(Addr::unchecked(TREASURY_ADDRESS), NATIVE_DENOM)
            .unwrap();
        assert_eq!(0_u128, treasury_balance.amount.into());

        let query_market = blockchain_contract.query_market().unwrap();
        assert_eq!(Status::CLOSED, query_market.market.status);
        assert_eq!(MarketResult::DRAW, query_market.market.result.unwrap());
    }

    #[test]
    fn unauthorized() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 10; // 10 minutes from now

        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![(
                Addr::unchecked(ADMIN_ADDRESS),
                coins(INITIAL_BALANCE, NATIVE_DENOM),
            )],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                fee_bps: DEFAULT_FEE_BPS,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            },
            vec![],
        )
        .unwrap();

        blockchain_contract
            .place_bet(
                &Addr::unchecked(ADMIN_ADDRESS),
                MarketResult::HOME,
                None,
                &coins(1_000, NATIVE_DENOM),
            )
            .unwrap();

        blockchain_contract
            .place_bet(
                &Addr::unchecked(ADMIN_ADDRESS),
                MarketResult::DRAW,
                None,
                &coins(1_000, NATIVE_DENOM),
            )
            .unwrap();

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(1_000, query_bets.totals.draw);
        assert_eq!(0, query_bets.totals.away);
        assert_eq!(1_000, query_bets.totals.home);

        blockchain_contract.blockchain.update_block(|block| {
            block.time = Timestamp::from_seconds(
                start_timestamp + 60 * 30, // 30 minutes after the start timestamp
            );
        });

        let err = blockchain_contract
            .score_market(
                &blockchain_contract.blockchain.api().addr_make(ANYONE),
                MarketResult::DRAW,
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
    fn market_not_drawable() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 10; // 10 minutes from now

        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![(
                Addr::unchecked(ADMIN_ADDRESS),
                coins(INITIAL_BALANCE, NATIVE_DENOM),
            )],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                fee_bps: DEFAULT_FEE_BPS,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: false,
            },
            vec![],
        )
        .unwrap();

        blockchain_contract
            .place_bet(
                &Addr::unchecked(ADMIN_ADDRESS),
                MarketResult::HOME,
                None,
                &coins(1_000, NATIVE_DENOM),
            )
            .unwrap();

        blockchain_contract
            .place_bet(
                &Addr::unchecked(ADMIN_ADDRESS),
                MarketResult::AWAY,
                None,
                &coins(1_000, NATIVE_DENOM),
            )
            .unwrap();

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(0, query_bets.totals.draw);
        assert_eq!(1_000, query_bets.totals.away);
        assert_eq!(1_000, query_bets.totals.home);

        blockchain_contract.blockchain.update_block(|block| {
            block.time = Timestamp::from_seconds(
                start_timestamp + 60 * 30, // 30 minutes after the start timestamp
            );
        });

        let err = blockchain_contract
            .score_market(&Addr::unchecked(ADMIN_ADDRESS), MarketResult::DRAW)
            .unwrap_err();
        assert_eq!(
            ContractError::MarketNotDrawable {},
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
            + 60 * 10; // 10 minutes from now

        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![(
                Addr::unchecked(ADMIN_ADDRESS),
                coins(INITIAL_BALANCE, NATIVE_DENOM),
            )],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                fee_bps: DEFAULT_FEE_BPS,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            },
            vec![],
        )
        .unwrap();

        blockchain_contract
            .cancel_market(&Addr::unchecked(ADMIN_ADDRESS))
            .unwrap();

        let query_market = blockchain_contract.query_market().unwrap();
        assert_ne!(Status::ACTIVE, query_market.market.status);
        assert_eq!(None, query_market.market.result);

        let err = blockchain_contract
            .score_market(&Addr::unchecked(ADMIN_ADDRESS), MarketResult::DRAW)
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
                fee_bps: DEFAULT_FEE_BPS,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            },
            vec![],
        )
        .unwrap();

        blockchain_contract
            .place_bet(
                &Addr::unchecked(ADMIN_ADDRESS),
                MarketResult::DRAW,
                None,
                &coins(1_000, NATIVE_DENOM),
            )
            .unwrap();

        blockchain_contract
            .place_bet(
                &Addr::unchecked(ADMIN_ADDRESS),
                MarketResult::HOME,
                None,
                &coins(1_000, NATIVE_DENOM),
            )
            .unwrap();

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(1_000, query_bets.totals.draw);
        assert_eq!(0, query_bets.totals.away);
        assert_eq!(1_000, query_bets.totals.home);

        blockchain_contract.blockchain.update_block(|block| {
            block.time = Timestamp::from_seconds(
                start_timestamp + 60 * 30 - 1, // 29 minutes and 59 seconds from start timestamp
            );
        });

        let err = blockchain_contract
            .score_market(&Addr::unchecked(ADMIN_ADDRESS), MarketResult::DRAW)
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

    #[test]
    fn market_has_no_winnings() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 10; // 10 minutes from now

        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![(
                Addr::unchecked(ADMIN_ADDRESS),
                coins(INITIAL_BALANCE, NATIVE_DENOM),
            )],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                fee_bps: DEFAULT_FEE_BPS,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            },
            vec![],
        )
        .unwrap();

        blockchain_contract
            .place_bet(
                &Addr::unchecked(ADMIN_ADDRESS),
                MarketResult::DRAW,
                None,
                &coins(1_000, NATIVE_DENOM),
            )
            .unwrap();

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(1_000, query_bets.totals.draw);
        assert_eq!(0, query_bets.totals.away);
        assert_eq!(0, query_bets.totals.home);

        blockchain_contract.blockchain.update_block(|block| {
            block.time = Timestamp::from_seconds(
                start_timestamp + 60 * 30, // 30 minutes after the start timestamp
            );
        });

        let err = blockchain_contract
            .score_market(&Addr::unchecked(ADMIN_ADDRESS), MarketResult::DRAW)
            .unwrap_err();
        assert_eq!(
            ContractError::NoWinnings {},
            err.downcast::<ContractError>().unwrap()
        );

        let query_market = blockchain_contract.query_market().unwrap();
        assert_eq!(Status::ACTIVE, query_market.market.status);
        assert_eq!(None, query_market.market.result);
    }

    #[test]
    fn market_has_no_winners() {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 10; // 10 minutes from now

        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![(
                Addr::unchecked(ADMIN_ADDRESS),
                coins(INITIAL_BALANCE, NATIVE_DENOM),
            )],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                fee_bps: DEFAULT_FEE_BPS,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            },
            vec![],
        )
        .unwrap();

        blockchain_contract
            .place_bet(
                &Addr::unchecked(ADMIN_ADDRESS),
                MarketResult::DRAW,
                None,
                &coins(1_000, NATIVE_DENOM),
            )
            .unwrap();

        blockchain_contract
            .place_bet(
                &Addr::unchecked(ADMIN_ADDRESS),
                MarketResult::AWAY,
                None,
                &coins(1_000, NATIVE_DENOM),
            )
            .unwrap();

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(1_000, query_bets.totals.draw);
        assert_eq!(1_000, query_bets.totals.away);
        assert_eq!(0, query_bets.totals.home);

        blockchain_contract.blockchain.update_block(|block| {
            block.time = Timestamp::from_seconds(
                start_timestamp + 60 * 30, // 30 minutes after the start timestamp
            );
        });

        let err = blockchain_contract
            .score_market(&Addr::unchecked(ADMIN_ADDRESS), MarketResult::HOME)
            .unwrap_err();
        assert_eq!(
            ContractError::NoWinnings {},
            err.downcast::<ContractError>().unwrap()
        );

        let query = blockchain_contract.query_market().unwrap();
        assert_eq!(Status::ACTIVE, query.market.status);
        assert_eq!(None, query.market.result);
    }
}

mod cancel_market {
    use super::*;

    #[test]
    fn proper_cancel_market() {
        let mut blockchain_contract = setup_blockchain_and_contract(
            Addr::unchecked(ADMIN_ADDRESS),
            vec![(
                Addr::unchecked(ADMIN_ADDRESS),
                coins(INITIAL_BALANCE, NATIVE_DENOM),
            )],
            InstantiateMsg {
                denom: NATIVE_DENOM.to_string(),
                denom_precision: NATIVE_DENOM_PRECISION,
                fee_bps: DEFAULT_FEE_BPS,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs()
                    + 60 * 5, // 5 minutes from now
                is_drawable: true,
            },
            vec![],
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
                fee_bps: DEFAULT_FEE_BPS,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs()
                    + 60 * 5, // 5 minutes from now
                is_drawable: true,
            },
            vec![],
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
                fee_bps: DEFAULT_FEE_BPS,
                id: "game-cs2-test-league".to_string(),
                label: "CS2 - Test League - Team A vs Team B".to_string(),
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                start_timestamp,
                is_drawable: true,
            },
            vec![],
        )
        .unwrap();

        blockchain_contract
            .place_bet(
                &Addr::unchecked(ADMIN_ADDRESS),
                MarketResult::DRAW,
                None,
                &coins(1_000, NATIVE_DENOM),
            )
            .unwrap();

        blockchain_contract
            .place_bet(
                &Addr::unchecked(ADMIN_ADDRESS),
                MarketResult::AWAY,
                None,
                &coins(1_000, NATIVE_DENOM),
            )
            .unwrap();

        let query_bets = blockchain_contract.query_bets().unwrap();
        assert_eq!(1_000, query_bets.totals.draw);
        assert_eq!(1_000, query_bets.totals.away);
        assert_eq!(0, query_bets.totals.home);

        blockchain_contract.blockchain.update_block(|block| {
            block.time = Timestamp::from_seconds(
                start_timestamp + 60 * 45, // 45 minutes after the start timestamp
            );
        });

        blockchain_contract
            .score_market(&Addr::unchecked(ADMIN_ADDRESS), MarketResult::DRAW)
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
