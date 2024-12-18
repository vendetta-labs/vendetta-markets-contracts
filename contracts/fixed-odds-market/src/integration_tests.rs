#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::contract::{ADMIN_ADDRESS, TREASURY_ADDRESS};
    use crate::helpers::CwContract;
    use crate::msg::InstantiateMsg;
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
            let (mut _app, _contract) = proper_instantiate();
        }
    }
}
