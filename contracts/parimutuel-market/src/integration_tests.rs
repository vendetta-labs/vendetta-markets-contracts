#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::contract::ADMIN_ADDRESS;
    use crate::helpers::CwContract;
    use crate::msg::InstantiateMsg;
    use cosmwasm_std::testing::MockApi;
    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    pub fn contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    const NATIVE_DENOM: &str = "denom";
    const USER_A: &str = "USER_A";
    const USER_B: &str = "USER_B";
    const USER_C: &str = "USER_C";

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &MockApi::default().addr_make(USER_A),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(1_000_000),
                    }],
                )
                .unwrap();
            router
                .bank
                .init_balance(
                    storage,
                    &MockApi::default().addr_make(USER_B),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(2_000_000),
                    }],
                )
                .unwrap();
            router
                .bank
                .init_balance(
                    storage,
                    &MockApi::default().addr_make(USER_C),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(3_000_000),
                    }],
                )
                .unwrap();
        })
    }

    fn proper_instantiate() -> (App, CwContract) {
        let mut app = mock_app();
        let cw_template_id = app.store_code(contract_template());

        let user_a = app.api().addr_make(USER_A);
        assert_eq!(
            app.wrap()
                .query_balance(user_a, NATIVE_DENOM)
                .unwrap()
                .amount,
            Uint128::new(1_000_000)
        );

        let user_b = app.api().addr_make(USER_B);
        assert_eq!(
            app.wrap()
                .query_balance(user_b, NATIVE_DENOM)
                .unwrap()
                .amount,
            Uint128::new(2_000_000)
        );

        let user_c = app.api().addr_make(USER_C);
        assert_eq!(
            app.wrap()
                .query_balance(user_c, NATIVE_DENOM)
                .unwrap()
                .amount,
            Uint128::new(3_000_000)
        );

        let msg = InstantiateMsg {
            denom: NATIVE_DENOM.to_string(),
            fee_bps: 250,
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
        use cosmwasm_std::coin;

        use super::*;
        use crate::{
            msg::{BetsByAddressResponse, BetsResponse, ExecuteMsg, QueryMsg},
            state::MarketResult,
        };

        #[test]
        fn it_accepts_bets() {
            let (mut app, contract) = proper_instantiate();

            let user_b = app.api().addr_make(USER_B);

            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::AWAY,
                receiver: None,
            };
            let cosmos_msg = contract.call(msg, vec![coin(2_000, NATIVE_DENOM)]).unwrap();
            app.execute(user_b.clone(), cosmos_msg).unwrap();

            let user_a = app.api().addr_make(USER_A);

            let msg = ExecuteMsg::PlaceBet {
                result: MarketResult::HOME,
                receiver: None,
            };
            let cosmos_msg = contract.call(msg, vec![coin(1_000, NATIVE_DENOM)]).unwrap();
            app.execute(user_a.clone(), cosmos_msg).unwrap();

            let query: BetsByAddressResponse = app
                .wrap()
                .query_wasm_smart(
                    contract.addr(),
                    &QueryMsg::BetsByAddress {
                        address: user_a.clone(),
                    },
                )
                .unwrap();

            assert_eq!(query.totals.total_home, 1_000);
            assert_eq!(query.totals.total_away, 0);
            assert_eq!(query.totals.total_draw, 0);

            let query: BetsResponse = app
                .wrap()
                .query_wasm_smart(contract.addr(), &&QueryMsg::Bets {})
                .unwrap();

            assert_eq!(query.totals.total_home, 1_000);
            assert_eq!(query.totals.total_away, 2_000);
            assert_eq!(query.totals.total_draw, 0);
        }
    }
}
