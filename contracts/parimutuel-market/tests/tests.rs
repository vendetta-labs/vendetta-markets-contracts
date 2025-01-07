use cosmwasm_std::{
    coin, from_json,
    testing::{message_info, mock_dependencies, mock_env},
    Addr, BankMsg, Coin, CosmosMsg, Empty, Timestamp, Uint128,
};
use cw_multi_test::{App, AppBuilder, BankKeeper, Executor, MockApiBech32};
use helpers::{contract_template, CwContract};
use parimutuel_market::{
    contract::{execute, instantiate, query, ADMIN_ADDRESS, TREASURY_ADDRESS},
    error::ContractError,
    msg::{
        BetsByAddressResponse, BetsResponse, ConfigResponse, ExecuteMsg, InstantiateMsg,
        MarketResponse, QueryMsg,
    },
    state::{MarketResult, Status},
};
use std::time::{SystemTime, UNIX_EPOCH};

mod helpers;

const NATIVE_DENOM: &str = "denom";
const FAKE_DENOM: &str = "fakedenom";
const OTHER: &str = "USER_OTHER";
const ANYONE: &str = "USER_ANYONE";
const USER_A: &str = "USER_A";
const USER_B: &str = "USER_B";
const USER_C: &str = "USER_C";
const DEFAULT_FEE_BPS: u64 = 250;
const INITIAL_BALANCE: u128 = 1_000_000_000;

fn proper_instantiate() -> (App<BankKeeper, MockApiBech32>, CwContract) {
    let mut app = AppBuilder::new()
        .with_api(MockApiBech32::new("neutron"))
        .build(
            |router: &mut cw_multi_test::Router<
                BankKeeper,
                cw_multi_test::FailingModule<Empty, Empty, Empty>,
                cw_multi_test::WasmKeeper<Empty, Empty>,
                cw_multi_test::FailingModule<Empty, Empty, Empty>,
                cw_multi_test::FailingModule<Empty, Empty, Empty>,
                cw_multi_test::FailingModule<cosmwasm_std::IbcMsg, cosmwasm_std::IbcQuery, Empty>,
                cw_multi_test::FailingModule<Empty, Empty, Empty>,
                cw_multi_test::StargateFailing,
            >,
             _,
             storage| {
                router
                    .bank
                    .init_balance(
                        storage,
                        &MockApiBech32::new("neutron").addr_make(OTHER),
                        vec![Coin {
                            denom: NATIVE_DENOM.to_string(),
                            amount: Uint128::new(INITIAL_BALANCE),
                        }],
                    )
                    .unwrap();
                router
                    .bank
                    .init_balance(
                        storage,
                        &MockApiBech32::new("neutron").addr_make(ANYONE),
                        vec![
                            Coin {
                                denom: NATIVE_DENOM.to_string(),
                                amount: Uint128::new(INITIAL_BALANCE),
                            },
                            Coin {
                                denom: FAKE_DENOM.to_string(),
                                amount: Uint128::new(INITIAL_BALANCE),
                            },
                        ],
                    )
                    .unwrap();
                router
                    .bank
                    .init_balance(
                        storage,
                        &MockApiBech32::new("neutron").addr_make(USER_A),
                        vec![Coin {
                            denom: NATIVE_DENOM.to_string(),
                            amount: Uint128::new(INITIAL_BALANCE),
                        }],
                    )
                    .unwrap();
                router
                    .bank
                    .init_balance(
                        storage,
                        &MockApiBech32::new("neutron").addr_make(USER_B),
                        vec![Coin {
                            denom: NATIVE_DENOM.to_string(),
                            amount: Uint128::new(INITIAL_BALANCE),
                        }],
                    )
                    .unwrap();
                router
                    .bank
                    .init_balance(
                        storage,
                        &MockApiBech32::new("neutron").addr_make(USER_C),
                        vec![Coin {
                            denom: NATIVE_DENOM.to_string(),
                            amount: Uint128::new(INITIAL_BALANCE),
                        }],
                    )
                    .unwrap();
                router
                    .bank
                    .init_balance(
                        storage,
                        &Addr::unchecked(TREASURY_ADDRESS),
                        vec![Coin {
                            denom: NATIVE_DENOM.to_string(),
                            amount: Uint128::new(0),
                        }],
                    )
                    .unwrap();
            },
        );
    let cw_template_id = app.store_code(contract_template());

    let other = app.api().addr_make(OTHER);
    assert_eq!(
        app.wrap()
            .query_balance(other, NATIVE_DENOM)
            .unwrap()
            .amount,
        Uint128::new(INITIAL_BALANCE)
    );

    let anyone = app.api().addr_make(ANYONE);
    assert_eq!(
        app.wrap()
            .query_balance(anyone.clone(), NATIVE_DENOM)
            .unwrap()
            .amount,
        Uint128::new(INITIAL_BALANCE)
    );
    assert_eq!(
        app.wrap().query_balance(anyone, FAKE_DENOM).unwrap().amount,
        Uint128::new(INITIAL_BALANCE)
    );

    let user_a = app.api().addr_make(USER_A);
    assert_eq!(
        app.wrap()
            .query_balance(user_a, NATIVE_DENOM)
            .unwrap()
            .amount,
        Uint128::new(INITIAL_BALANCE)
    );

    let user_b = app.api().addr_make(USER_B);
    assert_eq!(
        app.wrap()
            .query_balance(user_b, NATIVE_DENOM)
            .unwrap()
            .amount,
        Uint128::new(INITIAL_BALANCE)
    );

    let user_c = app.api().addr_make(USER_C);
    assert_eq!(
        app.wrap()
            .query_balance(user_c, NATIVE_DENOM)
            .unwrap()
            .amount,
        Uint128::new(INITIAL_BALANCE)
    );

    assert_eq!(
        app.wrap()
            .query_balance(Addr::unchecked(TREASURY_ADDRESS), NATIVE_DENOM)
            .unwrap()
            .amount,
        Uint128::new(0)
    );

    let msg = InstantiateMsg {
        denom: NATIVE_DENOM.to_string(),
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
    };
    let contract_addr = app
        .instantiate_contract(
            cw_template_id,
            Addr::unchecked(ADMIN_ADDRESS),
            &msg,
            &[],
            "test",
            None,
        )
        .unwrap();

    let contract = CwContract(contract_addr);

    (app, contract)
}

mod create_market {
    use super::*;

    #[test]
    fn it_properly_creates_a_market() {
        let mut deps = mock_dependencies();

        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 5; // 5 minutes from now
        let msg = InstantiateMsg {
            denom: NATIVE_DENOM.to_string(),
            fee_bps: 250,
            id: "game-cs2-test-league".to_string(),
            label: "CS2 - Test League - Team A vs Team B".to_string(),
            home_team: "Team A".to_string(),
            away_team: "Team B".to_string(),
            start_timestamp,
            is_drawable: true,
        };
        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);

        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let config_query = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
        let value: ConfigResponse = from_json(&config_query).unwrap();
        assert_eq!(250, value.config.fee_bps);
        assert_eq!(ADMIN_ADDRESS, value.config.admin_addr.as_str());
        assert_eq!(TREASURY_ADDRESS, value.config.treasury_addr.as_str());
        assert_eq!(NATIVE_DENOM, value.config.denom.as_str());

        let market_query = query(deps.as_ref(), mock_env(), QueryMsg::Market {}).unwrap();
        let value: MarketResponse = from_json(&market_query).unwrap();
        assert_eq!("game-cs2-test-league", value.market.id);
        assert_eq!("CS2 - Test League - Team A vs Team B", value.market.label);
        assert_eq!("Team A", value.market.home_team);
        assert_eq!("Team B", value.market.away_team);
        assert_eq!(start_timestamp, value.market.start_timestamp);
        assert_eq!(true, value.market.is_drawable);
        assert_eq!(Status::ACTIVE, value.market.status);
        assert_eq!(None, value.market.result);
    }

    #[test]
    fn it_correctly_calculates_the_winnings_for_the_market_and_collects_fees() {
        let (mut app, contract) = proper_instantiate();

        let user_a = app.api().addr_make(USER_A);

        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::HOME,
            receiver: None,
        };
        let cosmos_msg = contract
            .call(msg, vec![coin(100_762_000, NATIVE_DENOM)])
            .unwrap();
        app.execute(user_a.clone(), cosmos_msg).unwrap();

        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::DRAW,
            receiver: None,
        };
        let cosmos_msg = contract
            .call(msg, vec![coin(340_228_000, NATIVE_DENOM)])
            .unwrap();
        app.execute(user_a.clone(), cosmos_msg).unwrap();

        let user_b = app.api().addr_make(USER_B);

        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::AWAY,
            receiver: None,
        };
        let cosmos_msg = contract
            .call(msg, vec![coin(200_505_000, NATIVE_DENOM)])
            .unwrap();
        app.execute(user_b.clone(), cosmos_msg).unwrap();

        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::HOME,
            receiver: None,
        };
        let cosmos_msg = contract
            .call(msg, vec![coin(2_505_000, NATIVE_DENOM)])
            .unwrap();
        app.execute(user_b.clone(), cosmos_msg).unwrap();

        let user_c = app.api().addr_make(USER_C);

        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::DRAW,
            receiver: None,
        };
        let cosmos_msg = contract
            .call(msg, vec![coin(300_029_300, NATIVE_DENOM)])
            .unwrap();
        app.execute(user_c.clone(), cosmos_msg).unwrap();

        let anyone = app.api().addr_make(ANYONE);

        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::DRAW,
            receiver: None,
        };
        let cosmos_msg = contract
            .call(msg, vec![coin(300_029_300, FAKE_DENOM)])
            .unwrap();
        let res = app.execute(anyone.clone(), cosmos_msg).unwrap_err();
        assert_eq!("Payment error", res.root_cause().to_string());

        let query: BetsResponse = app
            .wrap()
            .query_wasm_smart(contract.addr(), &&QueryMsg::Bets {})
            .unwrap();
        assert_eq!(query.totals.home, 103_267_000);
        assert_eq!(query.totals.away, 200_505_000);
        assert_eq!(query.totals.draw, 640_257_300);

        let total_bets = 103_267_000 + 200_505_000 + 640_257_300;
        assert_eq!(
            total_bets,
            query.totals.home + query.totals.away + query.totals.draw
        );

        let query: BetsByAddressResponse = app
            .wrap()
            .query_wasm_smart(
                contract.addr(),
                &QueryMsg::BetsByAddress {
                    address: user_a.clone(),
                },
            )
            .unwrap();
        assert_eq!(query.totals.home, 100_762_000);
        assert_eq!(query.totals.away, 0);
        assert_eq!(query.totals.draw, 340_228_000);

        let query: BetsByAddressResponse = app
            .wrap()
            .query_wasm_smart(
                contract.addr(),
                &QueryMsg::BetsByAddress {
                    address: user_b.clone(),
                },
            )
            .unwrap();
        assert_eq!(query.totals.home, 2_505_000);
        assert_eq!(query.totals.away, 200_505_000);
        assert_eq!(query.totals.draw, 0);

        let query: BetsByAddressResponse = app
            .wrap()
            .query_wasm_smart(
                contract.addr(),
                &QueryMsg::BetsByAddress {
                    address: user_c.clone(),
                },
            )
            .unwrap();
        assert_eq!(query.totals.home, 0);
        assert_eq!(query.totals.away, 0);
        assert_eq!(query.totals.draw, 300_029_300);

        app.update_block(|block| {
            block.height += 1; // increase block height
            block.time = Timestamp::from_seconds(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs()
                    + 60 * 45,
            ); // fastforward 45 minutes
        });

        let treasury_balance = app
            .wrap()
            .query_balance(Addr::unchecked(TREASURY_ADDRESS), NATIVE_DENOM)
            .unwrap();
        assert_eq!(treasury_balance.amount, Uint128::new(0));

        let market_balance = app
            .wrap()
            .query_balance(contract.addr().clone(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(Uint128::new(total_bets), market_balance.amount);

        let msg = ExecuteMsg::Score {
            result: MarketResult::DRAW,
        };
        let cosmos_msg = contract.call(msg, vec![]).unwrap();
        app.execute(Addr::unchecked(ADMIN_ADDRESS), cosmos_msg)
            .unwrap();

        let treasury_balance = app
            .wrap()
            .query_balance(Addr::unchecked(TREASURY_ADDRESS), NATIVE_DENOM)
            .unwrap();
        let fee_amount = Uint128::from(total_bets)
            .multiply_ratio(Uint128::from(DEFAULT_FEE_BPS), Uint128::from(10000_u128));
        assert_eq!(fee_amount, treasury_balance.amount);

        let query: MarketResponse = app
            .wrap()
            .query_wasm_smart(contract.addr(), &QueryMsg::Market {})
            .unwrap();
        assert_eq!(query.market.status, Status::CLOSED);
        assert_eq!(query.market.result.unwrap(), MarketResult::DRAW);

        let market_balance_before = app
            .wrap()
            .query_balance(contract.addr().clone(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(
            Uint128::new(total_bets) - fee_amount,
            market_balance_before.amount
        );

        let user_a_balance_before = app
            .wrap()
            .query_balance(user_a.clone(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(Uint128::new(559_010_000), user_a_balance_before.amount);

        let msg = ExecuteMsg::ClaimWinnings { receiver: None };
        let cosmos_msg = contract.call(msg, vec![]).unwrap();
        app.execute(user_a.clone(), cosmos_msg).unwrap();

        let user_a_balance_after = app
            .wrap()
            .query_balance(user_a.clone(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(
            user_a_balance_before.amount + Uint128::new(489_108_942),
            user_a_balance_after.amount
        );

        let user_b_balance_before = app
            .wrap()
            .query_balance(user_b.clone(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(Uint128::new(796_990_000), user_b_balance_before.amount);

        let msg = ExecuteMsg::ClaimWinnings { receiver: None };
        let cosmos_msg = contract.call(msg, vec![]).unwrap();
        let res = app.execute(user_b.clone(), cosmos_msg).unwrap_err();
        assert_eq!("No winnings", res.root_cause().to_string());

        let user_b_balance_after = app
            .wrap()
            .query_balance(user_b.clone(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(user_b_balance_before.amount, user_b_balance_after.amount);

        let user_c_balance_before = app
            .wrap()
            .query_balance(user_c.clone(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(Uint128::new(699_970_700), user_c_balance_before.amount);

        let msg = ExecuteMsg::ClaimWinnings { receiver: None };
        let cosmos_msg = contract.call(msg, vec![]).unwrap();
        app.execute(user_c.clone(), cosmos_msg).unwrap();

        let user_c_balance_after = app
            .wrap()
            .query_balance(user_c.clone(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(
            user_c_balance_before.amount + Uint128::new(431_319_625),
            user_c_balance_after.amount
        );

        let market_balance_after = app
            .wrap()
            .query_balance(contract.addr().clone(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(Uint128::new(1), market_balance_after.amount);
    }
}

mod place_bet {

    use super::*;

    #[test]
    fn it_properly_accepts_bets() {
        let mut deps = mock_dependencies();
        let block_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(block_timestamp);

        let start_timestamp = block_timestamp + 60 * 5; // 5 minutes from now
        let msg = InstantiateMsg {
            denom: NATIVE_DENOM.to_string(),
            fee_bps: 250,
            id: "game-cs2-test-league".to_string(),
            label: "CS2 - Test League - Team A vs Team B".to_string(),
            home_team: "Team A".to_string(),
            away_team: "Team B".to_string(),
            start_timestamp,
            is_drawable: true,
        };

        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        let anyone = deps.api.addr_make("ANYONE");
        let info = message_info(&anyone, &[coin(1_000, NATIVE_DENOM)]);
        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::AWAY,
            receiver: None,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
        let value: BetsResponse = from_json(&res).unwrap();
        assert_eq!(0, value.totals.draw);
        assert_eq!(1_000, value.totals.away);
        assert_eq!(0, value.totals.home);

        let res = query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::BetsByAddress {
                address: anyone.clone(),
            },
        )
        .unwrap();
        let value: BetsResponse = from_json(&res).unwrap();
        assert_eq!(0, value.totals.draw);
        assert_eq!(1_000, value.totals.away);
        assert_eq!(0, value.totals.home);
    }

    #[test]
    fn the_receiver_will_be_the_beneficiary_when_defined() {
        let mut deps = mock_dependencies();
        let block_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(block_timestamp);

        let start_timestamp = block_timestamp + 60 * 5; // 5 minutes from now
        let msg = InstantiateMsg {
            denom: NATIVE_DENOM.to_string(),
            fee_bps: 250,
            id: "game-cs2-test-league".to_string(),
            label: "CS2 - Test League - Team A vs Team B".to_string(),
            home_team: "Team A".to_string(),
            away_team: "Team B".to_string(),
            start_timestamp,
            is_drawable: true,
        };

        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        let anyone = deps.api.addr_make("ANYONE");
        let other = deps.api.addr_make("OTHER");
        let info = message_info(&anyone, &[coin(1_000, NATIVE_DENOM)]);
        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::AWAY,
            receiver: Some(other.clone()),
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
        let value: BetsResponse = from_json(&res).unwrap();
        assert_eq!(1_000, value.totals.away);

        let res = query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::BetsByAddress {
                address: anyone.clone(),
            },
        )
        .unwrap();
        let value: BetsResponse = from_json(&res).unwrap();
        assert_eq!(0, value.totals.away);

        let res = query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::BetsByAddress {
                address: other.clone(),
            },
        )
        .unwrap();
        let value: BetsResponse = from_json(&res).unwrap();
        assert_eq!(1_000, value.totals.away);
    }

    #[test]
    fn it_cant_place_bet_on_draw_when_market_isnt_drawable() {
        let mut deps = mock_dependencies();
        let block_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(block_timestamp);

        let start_timestamp = block_timestamp + 60 * 5; // 5 minutes from now
        let msg = InstantiateMsg {
            denom: NATIVE_DENOM.to_string(),
            fee_bps: 250,
            id: "game-cs2-test-league".to_string(),
            label: "CS2 - Test League - Team A vs Team B".to_string(),
            home_team: "Team A".to_string(),
            away_team: "Team B".to_string(),
            start_timestamp,
            is_drawable: false,
        };

        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        let anyone = deps.api.addr_make("ANYONE");
        let info = message_info(&anyone, &[coin(1_000, NATIVE_DENOM)]);
        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::DRAW,
            receiver: None,
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
        assert_eq!(ContractError::MarketNotDrawable {}, res);

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
        let value: BetsResponse = from_json(&res).unwrap();
        assert_eq!(0, value.totals.draw);
        assert_eq!(0, value.totals.away);
        assert_eq!(0, value.totals.home);

        let res = query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::BetsByAddress {
                address: anyone.clone(),
            },
        )
        .unwrap();
        let value: BetsResponse = from_json(&res).unwrap();
        assert_eq!(0, value.totals.draw);
        assert_eq!(0, value.totals.away);
        assert_eq!(0, value.totals.home);
    }

    #[test]
    fn it_cant_place_bet_if_market_isnt_active() {
        let mut deps = mock_dependencies();
        let block_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(block_timestamp);

        let start_timestamp = block_timestamp + 60 * 5; // 5 minutes from now
        let msg = InstantiateMsg {
            denom: NATIVE_DENOM.to_string(),
            fee_bps: 250,
            id: "game-cs2-test-league".to_string(),
            label: "CS2 - Test League - Team A vs Team B".to_string(),
            home_team: "Team A".to_string(),
            away_team: "Team B".to_string(),
            start_timestamp,
            is_drawable: true,
        };

        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        let msg = ExecuteMsg::Cancel {};
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
        let value: MarketResponse = from_json(&res).unwrap();
        assert_ne!(Status::ACTIVE, value.market.status);

        let anyone = deps.api.addr_make("ANYONE");
        let info = message_info(&anyone, &[coin(1_000, NATIVE_DENOM)]);
        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::DRAW,
            receiver: None,
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
        assert_eq!(ContractError::MarketNotActive {}, res);

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
        let value: BetsResponse = from_json(&res).unwrap();
        assert_eq!(0, value.totals.draw);
        assert_eq!(0, value.totals.away);
        assert_eq!(0, value.totals.home);

        let res = query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::BetsByAddress {
                address: anyone.clone(),
            },
        )
        .unwrap();
        let value: BetsResponse = from_json(&res).unwrap();
        assert_eq!(0, value.totals.draw);
        assert_eq!(0, value.totals.away);
        assert_eq!(0, value.totals.home);
    }

    #[test]
    fn it_can_only_place_bets_up_until_5_minutes_before_market_start_timestamp() {
        let mut deps = mock_dependencies();
        let block_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(block_timestamp);

        let start_timestamp = block_timestamp + 60 * 5 - 1; // 4 minutes and 59 seconds from now
        let msg = InstantiateMsg {
            denom: NATIVE_DENOM.to_string(),
            fee_bps: 250,
            id: "game-cs2-test-league".to_string(),
            label: "CS2 - Test League - Team A vs Team B".to_string(),
            home_team: "Team A".to_string(),
            away_team: "Team B".to_string(),
            start_timestamp,
            is_drawable: true,
        };

        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        let anyone = deps.api.addr_make("ANYONE");
        let info = message_info(&anyone, &[coin(1_000, NATIVE_DENOM)]);
        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::AWAY,
            receiver: None,
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
        assert_eq!(ContractError::BetsNotAccepted {}, res);

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
        let value: BetsResponse = from_json(&res).unwrap();
        assert_eq!(0, value.totals.draw);
        assert_eq!(0, value.totals.away);
        assert_eq!(0, value.totals.home);

        let res = query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::BetsByAddress {
                address: anyone.clone(),
            },
        )
        .unwrap();
        let value: BetsResponse = from_json(&res).unwrap();
        assert_eq!(0, value.totals.draw);
        assert_eq!(0, value.totals.away);
        assert_eq!(0, value.totals.home);
    }

    #[test]
    fn it_cant_place_bet_without_sending_funds_in_the_market_denom() {
        let mut deps = mock_dependencies();
        let block_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(block_timestamp);

        let start_timestamp = block_timestamp + 60 * 5; // 5 minutes from now
        let msg = InstantiateMsg {
            denom: NATIVE_DENOM.to_string(),
            fee_bps: 250,
            id: "game-cs2-test-league".to_string(),
            label: "CS2 - Test League - Team A vs Team B".to_string(),
            home_team: "Team A".to_string(),
            away_team: "Team B".to_string(),
            start_timestamp,
            is_drawable: true,
        };

        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        let anyone = deps.api.addr_make("ANYONE");
        let info = message_info(&anyone, &[coin(1_000, "otherdenom")]);
        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::AWAY,
            receiver: None,
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
        assert_eq!(ContractError::PaymentError {}, res);

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
        let value: BetsResponse = from_json(&res).unwrap();
        assert_eq!(0, value.totals.draw);
        assert_eq!(0, value.totals.away);
        assert_eq!(0, value.totals.home);

        let res = query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::BetsByAddress {
                address: anyone.clone(),
            },
        )
        .unwrap();
        let value: BetsResponse = from_json(&res).unwrap();
        assert_eq!(0, value.totals.draw);
        assert_eq!(0, value.totals.away);
        assert_eq!(0, value.totals.home);
    }
}

mod claim_winnings {
    use super::*;

    #[test]
    fn it_properly_claims_winnings() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs(),
        );

        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 10; // 10 minutes from now
        let msg = InstantiateMsg {
            denom: NATIVE_DENOM.to_string(),
            fee_bps: 250,
            id: "game-cs2-test-league".to_string(),
            label: "CS2 - Test League - Team A vs Team B".to_string(),
            home_team: "Team A".to_string(),
            away_team: "Team B".to_string(),
            start_timestamp,
            is_drawable: true,
        };
        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        let anyone = deps.api.addr_make("ANYONE");
        let info = message_info(&anyone, &[coin(1_000, NATIVE_DENOM)]);
        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::DRAW,
            receiver: None,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let other = deps.api.addr_make("OTHER");
        let info = message_info(&other, &[coin(1_000, NATIVE_DENOM)]);
        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::AWAY,
            receiver: None,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
        let value: BetsResponse = from_json(&res).unwrap();
        assert_eq!(1_000, value.totals.draw);
        assert_eq!(1_000, value.totals.away);
        assert_eq!(0, value.totals.home);

        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 45, // 45 minutes in the future
        );

        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        let msg = ExecuteMsg::Score {
            result: MarketResult::DRAW,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
        let value: MarketResponse = from_json(&res).unwrap();
        assert_eq!(Status::CLOSED, value.market.status);
        assert_eq!(MarketResult::DRAW, value.market.result.unwrap());

        let info = message_info(&anyone, &[]);
        let msg = ExecuteMsg::ClaimWinnings { receiver: None };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(1, res.messages.len());
        let send: CosmosMsg = res.messages.first().unwrap().msg.clone();
        match send {
            CosmosMsg::Bank(bank_msg) => match bank_msg {
                BankMsg::Send { to_address, amount } => {
                    assert_eq!(to_address, anyone.to_string());
                    assert_eq!(amount, vec![coin(1_950, NATIVE_DENOM)]);
                }
                _ => panic!("Unexpected message: {:?}", bank_msg),
            },
            _ => panic!("Unexpected message: {:?}", send),
        }
    }

    #[test]
    fn it_can_claim_on_behalf_of_the_receiver_when_defined() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs(),
        );

        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 10; // 10 minutes from now
        let msg = InstantiateMsg {
            denom: NATIVE_DENOM.to_string(),
            fee_bps: 250,
            id: "game-cs2-test-league".to_string(),
            label: "CS2 - Test League - Team A vs Team B".to_string(),
            home_team: "Team A".to_string(),
            away_team: "Team B".to_string(),
            start_timestamp,
            is_drawable: true,
        };
        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        let anyone = deps.api.addr_make("ANYONE");
        let info = message_info(&anyone, &[coin(1_000, NATIVE_DENOM)]);
        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::DRAW,
            receiver: None,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let other = deps.api.addr_make("OTHER");
        let info = message_info(&other, &[coin(1_000, NATIVE_DENOM)]);
        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::AWAY,
            receiver: None,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
        let value: BetsResponse = from_json(&res).unwrap();
        assert_eq!(1_000, value.totals.draw);
        assert_eq!(1_000, value.totals.away);
        assert_eq!(0, value.totals.home);

        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 45, // 45 minutes in the future
        );

        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        let msg = ExecuteMsg::Score {
            result: MarketResult::DRAW,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
        let value: MarketResponse = from_json(&res).unwrap();
        assert_eq!(Status::CLOSED, value.market.status);
        assert_eq!(MarketResult::DRAW, value.market.result.unwrap());

        let info = message_info(&other, &[]);
        let msg = ExecuteMsg::ClaimWinnings {
            receiver: Some(anyone.clone()),
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(1, res.messages.len());
        let send: CosmosMsg = res.messages.first().unwrap().msg.clone();
        match send {
            CosmosMsg::Bank(bank_msg) => match bank_msg {
                BankMsg::Send { to_address, amount } => {
                    assert_eq!(to_address, anyone.to_string());
                    assert_eq!(amount, vec![coin(1_950, NATIVE_DENOM)]);
                }
                _ => panic!("Unexpected message: {:?}", bank_msg),
            },
            _ => panic!("Unexpected message: {:?}", send),
        }
    }

    #[test]
    fn it_will_return_all_bets_made_if_market_was_cancelled() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs(),
        );

        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 10; // 10 minutes from now
        let msg = InstantiateMsg {
            denom: NATIVE_DENOM.to_string(),
            fee_bps: 250,
            id: "game-cs2-test-league".to_string(),
            label: "CS2 - Test League - Team A vs Team B".to_string(),
            home_team: "Team A".to_string(),
            away_team: "Team B".to_string(),
            start_timestamp,
            is_drawable: true,
        };
        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        let anyone = deps.api.addr_make("ANYONE");
        let info = message_info(&anyone, &[coin(1_000, NATIVE_DENOM)]);
        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::DRAW,
            receiver: None,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let other = deps.api.addr_make("OTHER");
        let info = message_info(&other, &[coin(750, NATIVE_DENOM)]);
        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::AWAY,
            receiver: None,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
        let value: BetsResponse = from_json(&res).unwrap();
        assert_eq!(1_000, value.totals.draw);
        assert_eq!(750, value.totals.away);
        assert_eq!(0, value.totals.home);

        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 45, // 45 minutes in the future
        );

        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        let msg = ExecuteMsg::Cancel {};
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
        let value: MarketResponse = from_json(&res).unwrap();
        assert_eq!(Status::CANCELLED, value.market.status);
        assert_eq!(None, value.market.result);

        let info = message_info(&anyone, &[]);
        let msg = ExecuteMsg::ClaimWinnings { receiver: None };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(1, res.messages.len());
        let send: CosmosMsg = res.messages.first().unwrap().msg.clone();
        match send {
            CosmosMsg::Bank(bank_msg) => match bank_msg {
                BankMsg::Send { to_address, amount } => {
                    assert_eq!(to_address, anyone.to_string());
                    assert_eq!(amount, vec![coin(1_000, NATIVE_DENOM)]);
                }
                _ => panic!("Unexpected message: {:?}", bank_msg),
            },
            _ => panic!("Unexpected message: {:?}", send),
        }

        let info = message_info(&other, &[]);
        let msg = ExecuteMsg::ClaimWinnings { receiver: None };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(1, res.messages.len());
        let send: CosmosMsg = res.messages.first().unwrap().msg.clone();
        match send {
            CosmosMsg::Bank(bank_msg) => match bank_msg {
                BankMsg::Send { to_address, amount } => {
                    assert_eq!(to_address, other.to_string());
                    assert_eq!(amount, vec![coin(750, NATIVE_DENOM)]);
                }
                _ => panic!("Unexpected message: {:?}", bank_msg),
            },
            _ => panic!("Unexpected message: {:?}", send),
        }
    }

    #[test]
    fn it_cant_claim_winnings_while_market_is_active() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs(),
        );

        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 10; // 10 minutes from now
        let msg = InstantiateMsg {
            denom: NATIVE_DENOM.to_string(),
            fee_bps: 250,
            id: "game-cs2-test-league".to_string(),
            label: "CS2 - Test League - Team A vs Team B".to_string(),
            home_team: "Team A".to_string(),
            away_team: "Team B".to_string(),
            start_timestamp,
            is_drawable: true,
        };
        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        let anyone = deps.api.addr_make("ANYONE");
        let info = message_info(&anyone, &[coin(1_000, NATIVE_DENOM)]);
        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::DRAW,
            receiver: None,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let other = deps.api.addr_make("OTHER");
        let info = message_info(&other, &[coin(1_000, NATIVE_DENOM)]);
        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::AWAY,
            receiver: None,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
        let value: BetsResponse = from_json(&res).unwrap();
        assert_eq!(1_000, value.totals.draw);
        assert_eq!(1_000, value.totals.away);
        assert_eq!(0, value.totals.home);

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
        let value: MarketResponse = from_json(&res).unwrap();
        assert_eq!(Status::ACTIVE, value.market.status);
        assert_eq!(None, value.market.result);

        let info = message_info(&anyone, &[]);
        let msg = ExecuteMsg::ClaimWinnings { receiver: None };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
        assert_eq!(ContractError::MarketNotClosed {}, res);
    }

    #[test]
    fn it_cant_claim_winnings_twice() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs(),
        );

        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 10; // 10 minutes from now
        let msg = InstantiateMsg {
            denom: NATIVE_DENOM.to_string(),
            fee_bps: 250,
            id: "game-cs2-test-league".to_string(),
            label: "CS2 - Test League - Team A vs Team B".to_string(),
            home_team: "Team A".to_string(),
            away_team: "Team B".to_string(),
            start_timestamp,
            is_drawable: true,
        };
        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        let anyone = deps.api.addr_make("ANYONE");
        let info = message_info(&anyone, &[coin(1_000, NATIVE_DENOM)]);
        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::DRAW,
            receiver: None,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let other = deps.api.addr_make("OTHER");
        let info = message_info(&other, &[coin(1_000, NATIVE_DENOM)]);
        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::AWAY,
            receiver: None,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
        let value: BetsResponse = from_json(&res).unwrap();
        assert_eq!(1_000, value.totals.draw);
        assert_eq!(1_000, value.totals.away);
        assert_eq!(0, value.totals.home);

        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 45, // 45 minutes in the future
        );

        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        let msg = ExecuteMsg::Score {
            result: MarketResult::DRAW,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
        let value: MarketResponse = from_json(&res).unwrap();
        assert_eq!(Status::CLOSED, value.market.status);
        assert_eq!(MarketResult::DRAW, value.market.result.unwrap());

        let info = message_info(&anyone, &[]);
        let msg = ExecuteMsg::ClaimWinnings { receiver: None };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(1, res.messages.len());
        let send: CosmosMsg = res.messages.first().unwrap().msg.clone();
        match send {
            CosmosMsg::Bank(bank_msg) => match bank_msg {
                BankMsg::Send { to_address, amount } => {
                    assert_eq!(to_address, anyone.to_string());
                    assert_eq!(amount, vec![coin(1_950, NATIVE_DENOM)]);
                }
                _ => panic!("Unexpected message: {:?}", bank_msg),
            },
            _ => panic!("Unexpected message: {:?}", send),
        }

        let info = message_info(&anyone, &[]);
        let msg = ExecuteMsg::ClaimWinnings { receiver: None };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
        assert_eq!(ContractError::ClaimAlreadyMade {}, res);
    }

    #[test]
    fn it_cant_claim_winnings_when_there_is_nothing_to_claim() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs(),
        );

        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 10; // 10 minutes from now
        let msg = InstantiateMsg {
            denom: NATIVE_DENOM.to_string(),
            fee_bps: 250,
            id: "game-cs2-test-league".to_string(),
            label: "CS2 - Test League - Team A vs Team B".to_string(),
            home_team: "Team A".to_string(),
            away_team: "Team B".to_string(),
            start_timestamp,
            is_drawable: true,
        };
        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        let anyone = deps.api.addr_make("ANYONE");
        let info = message_info(&anyone, &[coin(1_000, NATIVE_DENOM)]);
        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::DRAW,
            receiver: None,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let other = deps.api.addr_make("OTHER");
        let info = message_info(&other, &[coin(1_000, NATIVE_DENOM)]);
        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::AWAY,
            receiver: None,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
        let value: BetsResponse = from_json(&res).unwrap();
        assert_eq!(1_000, value.totals.draw);
        assert_eq!(1_000, value.totals.away);
        assert_eq!(0, value.totals.home);

        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 45, // 45 minutes in the future
        );

        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        let msg = ExecuteMsg::Score {
            result: MarketResult::DRAW,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
        let value: MarketResponse = from_json(&res).unwrap();
        assert_eq!(Status::CLOSED, value.market.status);
        assert_eq!(MarketResult::DRAW, value.market.result.unwrap());

        let info = message_info(&other, &[]);
        let msg = ExecuteMsg::ClaimWinnings { receiver: None };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
        assert_eq!(ContractError::NoWinnings {}, res);
    }
}

mod update_market {
    use super::*;

    #[test]
    fn proper_update_market() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs(), // Now
        );

        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            - 60 * 5; // 5 minutes ago
        let msg = InstantiateMsg {
            denom: NATIVE_DENOM.to_string(),
            fee_bps: 250,
            id: "game-cs2-test-league".to_string(),
            label: "CS2 - Test League - Team A vs Team B".to_string(),
            home_team: "Team A".to_string(),
            away_team: "Team B".to_string(),
            start_timestamp,
            is_drawable: true,
        };
        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
        let value: MarketResponse = from_json(&res).unwrap();
        assert_eq!(start_timestamp, value.market.start_timestamp);

        let new_start_timestamp = start_timestamp - 60 * 30; // 30 minutes ago
        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        let msg = ExecuteMsg::Update {
            start_timestamp: new_start_timestamp,
        };
        let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
        let value: MarketResponse = from_json(&res).unwrap();
        assert_eq!(new_start_timestamp, value.market.start_timestamp);
    }

    #[test]
    fn unauthorized() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs(), // Now
        );

        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 5; // 5 minutes from now
        let msg = InstantiateMsg {
            denom: NATIVE_DENOM.to_string(),
            fee_bps: 250,
            id: "game-cs2-test-league".to_string(),
            label: "CS2 - Test League - Team A vs Team B".to_string(),
            home_team: "Team A".to_string(),
            away_team: "Team B".to_string(),
            start_timestamp,
            is_drawable: true,
        };
        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
        let value: MarketResponse = from_json(&res).unwrap();
        assert_eq!(start_timestamp, value.market.start_timestamp);

        let anyone = deps.api.addr_make("ANYONE");
        let info = message_info(&anyone, &[]);
        let msg = ExecuteMsg::Update {
            start_timestamp: start_timestamp - 60 * 30, // 30 minutes ago
        };
        let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
        assert_eq!(ContractError::Unauthorized {}, res);

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
        let value: MarketResponse = from_json(&res).unwrap();
        assert_eq!(start_timestamp, value.market.start_timestamp);
    }

    #[test]
    fn market_not_active() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs(),
        );

        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 10; // 10 minutes from now
        let msg = InstantiateMsg {
            denom: NATIVE_DENOM.to_string(),
            fee_bps: 250,
            id: "game-cs2-test-league".to_string(),
            label: "CS2 - Test League - Team A vs Team B".to_string(),
            home_team: "Team A".to_string(),
            away_team: "Team B".to_string(),
            start_timestamp,
            is_drawable: true,
        };
        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        let info = message_info(
            &Addr::unchecked(ADMIN_ADDRESS),
            &[coin(1_000, NATIVE_DENOM)],
        );
        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::DRAW,
            receiver: None,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let info = message_info(
            &Addr::unchecked(ADMIN_ADDRESS),
            &[coin(1_000, NATIVE_DENOM)],
        );
        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::AWAY,
            receiver: None,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
        let value: BetsResponse = from_json(&res).unwrap();
        assert_eq!(1_000, value.totals.draw);
        assert_eq!(1_000, value.totals.away);
        assert_eq!(0, value.totals.home);

        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 45, // 45 minutes in the future
        );

        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        let msg = ExecuteMsg::Score {
            result: MarketResult::DRAW,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
        let value: MarketResponse = from_json(&res).unwrap();
        assert_ne!(Status::ACTIVE, value.market.status);

        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        let msg = ExecuteMsg::Update {
            start_timestamp: start_timestamp - 60 * 30, // 30 minutes ago
        };
        let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
        assert_eq!(ContractError::MarketNotActive {}, res);

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
        let value: MarketResponse = from_json(&res).unwrap();
        assert_eq!(start_timestamp, value.market.start_timestamp);
    }
}

mod score_market {
    use super::*;

    #[test]
    fn proper_score_market_and_collect_fees() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs(),
        );

        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 10; // 10 minutes from now
        let msg = InstantiateMsg {
            denom: NATIVE_DENOM.to_string(),
            fee_bps: 250,
            id: "game-cs2-test-league".to_string(),
            label: "CS2 - Test League - Team A vs Team B".to_string(),
            home_team: "Team A".to_string(),
            away_team: "Team B".to_string(),
            start_timestamp,
            is_drawable: true,
        };
        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        let info = message_info(
            &Addr::unchecked(ADMIN_ADDRESS),
            &[coin(1_000, NATIVE_DENOM)],
        );
        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::DRAW,
            receiver: None,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let info = message_info(
            &Addr::unchecked(ADMIN_ADDRESS),
            &[coin(1_000, NATIVE_DENOM)],
        );
        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::AWAY,
            receiver: None,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
        let value: BetsResponse = from_json(&res).unwrap();
        assert_eq!(1_000, value.totals.draw);
        assert_eq!(1_000, value.totals.away);
        assert_eq!(0, value.totals.home);

        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 45, // 45 minutes in the future
        );

        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        let msg = ExecuteMsg::Score {
            result: MarketResult::DRAW,
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(1, res.messages.len());
        let send: CosmosMsg = res.messages.first().unwrap().msg.clone();
        match send {
            CosmosMsg::Bank(bank_msg) => match bank_msg {
                BankMsg::Send { to_address, amount } => {
                    assert_eq!(to_address, TREASURY_ADDRESS);
                    assert_eq!(amount, vec![coin(50, NATIVE_DENOM)]);
                }
                _ => panic!("Unexpected message: {:?}", bank_msg),
            },
            _ => panic!("Unexpected message: {:?}", send),
        }

        let fee_event = res
            .attributes
            .iter()
            .find(|attribute| attribute.key == "fee_collected");
        assert_eq!("50", fee_event.unwrap().value);

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
        let value: MarketResponse = from_json(&res).unwrap();
        assert_eq!(Status::CLOSED, value.market.status);
        assert_eq!(MarketResult::DRAW, value.market.result.unwrap());
    }

    #[test]
    fn do_not_collect_fees_when_set_to_zero() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs(),
        );

        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 10; // 10 minutes from now
        let msg = InstantiateMsg {
            denom: NATIVE_DENOM.to_string(),
            fee_bps: 0,
            id: "game-cs2-test-league".to_string(),
            label: "CS2 - Test League - Team A vs Team B".to_string(),
            home_team: "Team A".to_string(),
            away_team: "Team B".to_string(),
            start_timestamp,
            is_drawable: true,
        };
        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        let info = message_info(
            &Addr::unchecked(ADMIN_ADDRESS),
            &[coin(1_000, NATIVE_DENOM)],
        );
        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::DRAW,
            receiver: None,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let info = message_info(
            &Addr::unchecked(ADMIN_ADDRESS),
            &[coin(1_000, NATIVE_DENOM)],
        );
        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::AWAY,
            receiver: None,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
        let value: BetsResponse = from_json(&res).unwrap();
        assert_eq!(1_000, value.totals.draw);
        assert_eq!(1_000, value.totals.away);
        assert_eq!(0, value.totals.home);

        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 45, // 45 minutes in the future
        );

        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        let msg = ExecuteMsg::Score {
            result: MarketResult::DRAW,
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());
        let fee_event = res
            .attributes
            .iter()
            .find(|attribute| attribute.key == "fee_collected");
        assert_eq!("0", fee_event.unwrap().value);

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
        let value: MarketResponse = from_json(&res).unwrap();
        assert_eq!(Status::CLOSED, value.market.status);
        assert_eq!(MarketResult::DRAW, value.market.result.unwrap());
    }

    #[test]
    fn unauthorized() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs(),
        );

        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 10; // 10 minutes from now
        let msg = InstantiateMsg {
            denom: NATIVE_DENOM.to_string(),
            fee_bps: 250,
            id: "game-cs2-test-league".to_string(),
            label: "CS2 - Test League - Team A vs Team B".to_string(),
            home_team: "Team A".to_string(),
            away_team: "Team B".to_string(),
            start_timestamp,
            is_drawable: true,
        };
        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        let info = message_info(
            &Addr::unchecked(ADMIN_ADDRESS),
            &[coin(1_000, NATIVE_DENOM)],
        );
        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::HOME,
            receiver: None,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let info = message_info(
            &Addr::unchecked(ADMIN_ADDRESS),
            &[coin(1_000, NATIVE_DENOM)],
        );
        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::DRAW,
            receiver: None,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
        let value: BetsResponse = from_json(&res).unwrap();
        assert_eq!(1_000, value.totals.draw);
        assert_eq!(0, value.totals.away);
        assert_eq!(1_000, value.totals.home);

        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 45, // 45 minutes in the future
        );

        let anyone = deps.api.addr_make("ANYONE");
        let info = message_info(&anyone, &[]);
        let msg = ExecuteMsg::Score {
            result: MarketResult::DRAW,
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
        assert_eq!(ContractError::Unauthorized {}, res);

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
        let value: MarketResponse = from_json(&res).unwrap();
        assert_eq!(Status::ACTIVE, value.market.status);
        assert_eq!(None, value.market.result);
    }

    #[test]
    fn market_not_drawable() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs(),
        );

        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 10; // 10 minutes from now
        let msg = InstantiateMsg {
            denom: NATIVE_DENOM.to_string(),
            fee_bps: 250,
            id: "game-cs2-test-league".to_string(),
            label: "CS2 - Test League - Team A vs Team B".to_string(),
            home_team: "Team A".to_string(),
            away_team: "Team B".to_string(),
            start_timestamp,
            is_drawable: false,
        };
        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        let info = message_info(
            &Addr::unchecked(ADMIN_ADDRESS),
            &[coin(1_000, NATIVE_DENOM)],
        );
        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::HOME,
            receiver: None,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let info = message_info(
            &Addr::unchecked(ADMIN_ADDRESS),
            &[coin(1_000, NATIVE_DENOM)],
        );
        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::AWAY,
            receiver: None,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
        let value: BetsResponse = from_json(&res).unwrap();
        assert_eq!(0, value.totals.draw);
        assert_eq!(1_000, value.totals.away);
        assert_eq!(1_000, value.totals.home);

        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 45, // 45 minutes in the future
        );

        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        let msg = ExecuteMsg::Score {
            result: MarketResult::DRAW,
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
        assert_eq!(ContractError::MarketNotDrawable {}, res);

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
        let value: MarketResponse = from_json(&res).unwrap();
        assert_eq!(Status::ACTIVE, value.market.status);
        assert_eq!(None, value.market.result);
    }

    #[test]
    fn market_not_active() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs(),
        );

        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 10; // 10 minutes from now
        let msg = InstantiateMsg {
            denom: NATIVE_DENOM.to_string(),
            fee_bps: 250,
            id: "game-cs2-test-league".to_string(),
            label: "CS2 - Test League - Team A vs Team B".to_string(),
            home_team: "Team A".to_string(),
            away_team: "Team B".to_string(),
            start_timestamp,
            is_drawable: true,
        };
        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 45, // 45 minutes in the future
        );

        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        let msg = ExecuteMsg::Cancel {};
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
        let value: MarketResponse = from_json(&res).unwrap();
        assert_ne!(Status::ACTIVE, value.market.status);
        assert_eq!(None, value.market.result);

        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        let msg = ExecuteMsg::Score {
            result: MarketResult::DRAW,
        };
        let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
        assert_eq!(ContractError::MarketNotActive {}, res);

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
        let value: MarketResponse = from_json(&res).unwrap();
        assert_eq!(None, value.market.result);
    }

    #[test]
    fn market_not_scoreable() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        let current_block_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        env.block.time = Timestamp::from_seconds(current_block_time);

        let start_timestamp = current_block_time + 60 * 5; // 5 minutes from block time
        let msg = InstantiateMsg {
            denom: NATIVE_DENOM.to_string(),
            fee_bps: 250,
            id: "game-cs2-test-league".to_string(),
            label: "CS2 - Test League - Team A vs Team B".to_string(),
            home_team: "Team A".to_string(),
            away_team: "Team B".to_string(),
            start_timestamp,
            is_drawable: true,
        };
        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        let info = message_info(
            &Addr::unchecked(ADMIN_ADDRESS),
            &[coin(1_000, NATIVE_DENOM)],
        );
        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::DRAW,
            receiver: None,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let info = message_info(
            &Addr::unchecked(ADMIN_ADDRESS),
            &[coin(1_000, NATIVE_DENOM)],
        );
        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::HOME,
            receiver: None,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
        let value: BetsResponse = from_json(&res).unwrap();
        assert_eq!(1_000, value.totals.draw);
        assert_eq!(0, value.totals.away);
        assert_eq!(1_000, value.totals.home);

        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(
            start_timestamp + 60 * 30 - 1, // 29 minutes and 59 seconds from start timestamp
        );
        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        let msg = ExecuteMsg::Score {
            result: MarketResult::DRAW,
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
        assert_eq!(ContractError::MarketNotScoreable {}, res);

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
        let value: MarketResponse = from_json(&res).unwrap();
        assert_eq!(Status::ACTIVE, value.market.status);
        assert_eq!(None, value.market.result);

        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(
            start_timestamp + 60 * 30, // 30 minutes from start timestamp
        );
        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        let msg = ExecuteMsg::Score {
            result: MarketResult::HOME,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
        let value: MarketResponse = from_json(&res).unwrap();
        assert_eq!(Status::CLOSED, value.market.status);
        assert_eq!(MarketResult::HOME, value.market.result.unwrap());
    }

    #[test]
    fn market_has_no_winnings() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs(),
        );

        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 10; // 10 minutes from now
        let msg = InstantiateMsg {
            denom: NATIVE_DENOM.to_string(),
            fee_bps: 250,
            id: "game-cs2-test-league".to_string(),
            label: "CS2 - Test League - Team A vs Team B".to_string(),
            home_team: "Team A".to_string(),
            away_team: "Team B".to_string(),
            start_timestamp,
            is_drawable: true,
        };
        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        let info = message_info(
            &Addr::unchecked(ADMIN_ADDRESS),
            &[coin(1_000, NATIVE_DENOM)],
        );
        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::DRAW,
            receiver: None,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
        let value: BetsResponse = from_json(&res).unwrap();
        assert_eq!(1_000, value.totals.draw);
        assert_eq!(0, value.totals.away);
        assert_eq!(0, value.totals.home);

        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 45, // 45 minutes in the future
        );

        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        let msg = ExecuteMsg::Score {
            result: MarketResult::DRAW,
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
        assert_eq!(ContractError::NoWinnings {}, res);

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
        let value: MarketResponse = from_json(&res).unwrap();
        assert_eq!(Status::ACTIVE, value.market.status);
        assert_eq!(None, value.market.result);
    }

    #[test]
    fn market_has_no_winners() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs(),
        );

        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 10; // 10 minutes from now
        let msg = InstantiateMsg {
            denom: NATIVE_DENOM.to_string(),
            fee_bps: 250,
            id: "game-cs2-test-league".to_string(),
            label: "CS2 - Test League - Team A vs Team B".to_string(),
            home_team: "Team A".to_string(),
            away_team: "Team B".to_string(),
            start_timestamp,
            is_drawable: true,
        };
        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        let info = message_info(
            &Addr::unchecked(ADMIN_ADDRESS),
            &[coin(1_000, NATIVE_DENOM)],
        );
        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::DRAW,
            receiver: None,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let info = message_info(
            &Addr::unchecked(ADMIN_ADDRESS),
            &[coin(1_000, NATIVE_DENOM)],
        );
        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::AWAY,
            receiver: None,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
        let value: BetsResponse = from_json(&res).unwrap();
        assert_eq!(1_000, value.totals.draw);
        assert_eq!(1_000, value.totals.away);
        assert_eq!(0, value.totals.home);

        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 45, // 45 minutes in the future
        );

        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        let msg = ExecuteMsg::Score {
            result: MarketResult::HOME,
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
        assert_eq!(ContractError::NoWinnings {}, res);

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
        let value: MarketResponse = from_json(&res).unwrap();
        assert_eq!(Status::ACTIVE, value.market.status);
        assert_eq!(None, value.market.result);
    }
}

mod cancel_market {
    use super::*;

    #[test]
    fn proper_cancel_market() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 45, // 45 minutes in the future
        );

        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 10; // 10 minutes from now
        let msg = InstantiateMsg {
            denom: NATIVE_DENOM.to_string(),
            fee_bps: 250,
            id: "game-cs2-test-league".to_string(),
            label: "CS2 - Test League - Team A vs Team B".to_string(),
            home_team: "Team A".to_string(),
            away_team: "Team B".to_string(),
            start_timestamp,
            is_drawable: true,
        };
        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
        let value: MarketResponse = from_json(&res).unwrap();
        assert_eq!(Status::ACTIVE, value.market.status);

        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        let msg = ExecuteMsg::Cancel {};
        let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
        let value: MarketResponse = from_json(&res).unwrap();
        assert_eq!(Status::CANCELLED, value.market.status);
    }

    #[test]
    fn unauthorized() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 45, // 45 minutes in the future
        );

        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 10; // 10 minutes from now
        let msg = InstantiateMsg {
            denom: NATIVE_DENOM.to_string(),
            fee_bps: 250,
            id: "game-cs2-test-league".to_string(),
            label: "CS2 - Test League - Team A vs Team B".to_string(),
            home_team: "Team A".to_string(),
            away_team: "Team B".to_string(),
            start_timestamp,
            is_drawable: true,
        };
        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
        let value: MarketResponse = from_json(&res).unwrap();
        assert_eq!(Status::ACTIVE, value.market.status);

        let anyone = deps.api.addr_make("ANYONE");
        let info = message_info(&anyone, &[]);
        let msg = ExecuteMsg::Cancel {};
        let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
        assert_eq!(ContractError::Unauthorized {}, res);

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
        let value: MarketResponse = from_json(&res).unwrap();
        assert_ne!(Status::CANCELLED, value.market.status);
    }

    #[test]
    fn market_not_active() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs(),
        );

        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 10; // 10 minutes from now
        let msg = InstantiateMsg {
            denom: NATIVE_DENOM.to_string(),
            fee_bps: 250,
            id: "game-cs2-test-league".to_string(),
            label: "CS2 - Test League - Team A vs Team B".to_string(),
            home_team: "Team A".to_string(),
            away_team: "Team B".to_string(),
            start_timestamp,
            is_drawable: true,
        };
        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        let info = message_info(
            &Addr::unchecked(ADMIN_ADDRESS),
            &[coin(1_000, NATIVE_DENOM)],
        );
        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::DRAW,
            receiver: None,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let info = message_info(
            &Addr::unchecked(ADMIN_ADDRESS),
            &[coin(1_000, NATIVE_DENOM)],
        );
        let msg = ExecuteMsg::PlaceBet {
            result: MarketResult::AWAY,
            receiver: None,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Bets {}).unwrap();
        let value: BetsResponse = from_json(&res).unwrap();
        assert_eq!(1_000, value.totals.draw);
        assert_eq!(1_000, value.totals.away);
        assert_eq!(0, value.totals.home);

        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 45, // 45 minutes in the future
        );

        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        let msg = ExecuteMsg::Score {
            result: MarketResult::DRAW,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
        let value: MarketResponse = from_json(&res).unwrap();
        assert_ne!(Status::ACTIVE, value.market.status);

        let info = message_info(&Addr::unchecked(ADMIN_ADDRESS), &[]);
        let msg = ExecuteMsg::Cancel {};
        let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
        assert_eq!(ContractError::MarketNotActive {}, res);

        let res = query(deps.as_ref(), env.clone(), QueryMsg::Market {}).unwrap();
        let value: MarketResponse = from_json(&res).unwrap();
        assert_ne!(Status::CANCELLED, value.market.status);
    }
}
