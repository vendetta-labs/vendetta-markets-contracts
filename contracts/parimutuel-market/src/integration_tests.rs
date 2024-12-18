#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::contract::{ADMIN_ADDRESS, TREASURY_ADDRESS};
    use crate::helpers::CwContract;
    use crate::msg::InstantiateMsg;
    use crate::{
        msg::{BetsByAddressResponse, BetsResponse, ExecuteMsg, MarketResponse, QueryMsg},
        state::{MarketResult, Status},
    };
    use cosmwasm_std::{coin, Timestamp};
    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw_multi_test::{
        App, AppBuilder, BankKeeper, Contract, ContractWrapper, Executor, MockApiBech32,
    };

    pub fn contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    const NATIVE_DENOM: &str = "denom";
    const FAKE_DENOM: &str = "fakedenom";
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
                    cw_multi_test::FailingModule<
                        cosmwasm_std::IbcMsg,
                        cosmwasm_std::IbcQuery,
                        Empty,
                    >,
                    cw_multi_test::FailingModule<Empty, Empty, Empty>,
                    cw_multi_test::StargateFailing,
                >,
                 _,
                 storage| {
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

        let user_c = app.api().addr_make(USER_C);
        assert_eq!(
            app.wrap().query_balance(user_c, FAKE_DENOM).unwrap().amount,
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

    mod market {
        use super::*;

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

            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::DRAW,
                receiver: None,
            };
            let cosmos_msg = contract
                .call(msg, vec![coin(300_029_300, FAKE_DENOM)])
                .unwrap();
            let res = app.execute(user_c.clone(), cosmos_msg).unwrap_err();
            assert_eq!("Payment error", res.root_cause().to_string());

            let query: BetsResponse = app
                .wrap()
                .query_wasm_smart(contract.addr(), &&QueryMsg::Bets {})
                .unwrap();
            assert_eq!(query.totals.total_home, 103_267_000);
            assert_eq!(query.totals.total_away, 200_505_000);
            assert_eq!(query.totals.total_draw, 640_257_300);

            let total_bets = 103_267_000 + 200_505_000 + 640_257_300;
            assert_eq!(
                total_bets,
                query.totals.total_home + query.totals.total_away + query.totals.total_draw
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
            assert_eq!(query.totals.total_home, 100_762_000);
            assert_eq!(query.totals.total_away, 0);
            assert_eq!(query.totals.total_draw, 340_228_000);

            let query: BetsByAddressResponse = app
                .wrap()
                .query_wasm_smart(
                    contract.addr(),
                    &QueryMsg::BetsByAddress {
                        address: user_b.clone(),
                    },
                )
                .unwrap();
            assert_eq!(query.totals.total_home, 2_505_000);
            assert_eq!(query.totals.total_away, 200_505_000);
            assert_eq!(query.totals.total_draw, 0);

            let query: BetsByAddressResponse = app
                .wrap()
                .query_wasm_smart(
                    contract.addr(),
                    &QueryMsg::BetsByAddress {
                        address: user_c.clone(),
                    },
                )
                .unwrap();
            assert_eq!(query.totals.total_home, 0);
            assert_eq!(query.totals.total_away, 0);
            assert_eq!(query.totals.total_draw, 300_029_300);

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
}
