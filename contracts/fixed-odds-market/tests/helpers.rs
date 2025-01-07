use cosmwasm_std::{Addr, Coin, StdResult};
use cw_multi_test::{App, AppBuilder, BankKeeper, ContractWrapper, Executor, MockApiBech32};
use derivative::Derivative;
use fixed_odds_market::{
    contract::{execute, instantiate, query},
    msg::{ConfigResponse, InstantiateMsg, MarketResponse, QueryMsg},
};

/// BlockchainContract is a wrapper around blockchain App and contract Addr
/// that provides a lot of helpers for working with this contract.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct BlockchainContract {
    #[derivative(Debug = "ignore")]
    pub blockchain: App<BankKeeper, MockApiBech32>,
    pub contract_addr: Addr,
}

impl BlockchainContract {
    pub fn addr(&self) -> Addr {
        self.contract_addr.clone()
    }

    pub fn query_config(&self) -> StdResult<ConfigResponse> {
        self.blockchain
            .wrap()
            .query_wasm_smart(self.addr(), &QueryMsg::Config {})
    }

    pub fn query_market(&self) -> StdResult<MarketResponse> {
        self.blockchain
            .wrap()
            .query_wasm_smart(self.addr(), &QueryMsg::Market {})
    }

    // pub fn call<T: Into<ExecuteMsg>>(&self, msg: T, funds: Vec<Coin>) -> StdResult<CosmosMsg> {
    //     let msg = to_json_binary(&msg.into())?;
    //     Ok(WasmMsg::Execute {
    //         contract_addr: self.addr().into(),
    //         msg,
    //         funds,
    //     }
    //     .into())
    // }
}

pub fn setup_blockchain_and_contract(
    admin: Addr,
    initial_balances: Vec<(Addr, Vec<Coin>)>,
    instantiate_msg: InstantiateMsg,
    instantiate_funds: Vec<Coin>,
) -> BlockchainContract {
    let mut blockchain = AppBuilder::new()
        .with_api(MockApiBech32::new("neutron"))
        .build(|router, _, storage| {
            initial_balances.into_iter().for_each(|(addr, coins)| {
                router.bank.init_balance(storage, &addr, coins).unwrap();
            });
        });

    let code = Box::new(ContractWrapper::new(execute, instantiate, query));

    let code_id = blockchain.store_code(code);

    let contract_addr = blockchain
        .instantiate_contract(
            code_id,
            admin.clone(),
            &instantiate_msg,
            &instantiate_funds,
            "Market",
            None,
        )
        .unwrap();

    BlockchainContract {
        blockchain,
        contract_addr,
    }
}
