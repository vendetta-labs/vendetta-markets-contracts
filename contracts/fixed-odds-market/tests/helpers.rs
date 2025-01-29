use cosmwasm_std::{Addr, Coin, Decimal, StdResult};
use cw_multi_test::{
    error::{AnyError, AnyResult},
    App, AppBuilder, AppResponse, BankKeeper, ContractWrapper, Executor, MockApiBech32,
};
use derivative::Derivative;
use fixed_odds_market::{
    contract::{execute, instantiate, query},
    msg::{
        BetsByAddressResponse, BetsResponse, ConfigResponse, ExecuteMsg, InstantiateMsg,
        MarketResponse, MaxBetsResponse, QueryMsg, UpdateParams,
    },
    state::MarketResult,
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

    pub fn query_max_bets(&self) -> StdResult<MaxBetsResponse> {
        self.blockchain
            .wrap()
            .query_wasm_smart(self.addr(), &QueryMsg::MaxBets {})
    }

    pub fn query_bets(&self) -> StdResult<BetsResponse> {
        self.blockchain
            .wrap()
            .query_wasm_smart(self.addr(), &QueryMsg::Bets {})
    }

    pub fn query_bets_by_address(&self, address: &Addr) -> StdResult<BetsByAddressResponse> {
        self.blockchain.wrap().query_wasm_smart(
            self.addr(),
            &QueryMsg::BetsByAddress {
                address: address.clone(),
            },
        )
    }

    pub fn cancel_market(&mut self, sender: &Addr) -> AnyResult<AppResponse> {
        self.blockchain
            .execute_contract(sender.clone(), self.addr(), &ExecuteMsg::Cancel {}, &[])
    }

    pub fn update_market(&mut self, sender: &Addr, params: UpdateParams) -> AnyResult<AppResponse> {
        self.blockchain.execute_contract(
            sender.clone(),
            self.addr(),
            &ExecuteMsg::Update {
                admin_addr: params.admin_addr,
                treasury_addr: params.treasury_addr,
                fee_spread_odds: params.fee_spread_odds,
                max_bet_risk_factor: params.max_bet_risk_factor,
                seed_liquidity_amplifier: params.seed_liquidity_amplifier,
                initial_odds_home: params.initial_odds_home,
                initial_odds_away: params.initial_odds_away,
                start_timestamp: params.start_timestamp,
            },
            &[],
        )
    }

    pub fn score_market(&mut self, sender: &Addr, result: MarketResult) -> AnyResult<AppResponse> {
        self.blockchain.execute_contract(
            sender.clone(),
            self.addr(),
            &ExecuteMsg::Score { result },
            &[],
        )
    }

    pub fn place_bet(
        &mut self,
        sender: &Addr,
        result: MarketResult,
        min_odds: Decimal,
        receiver: Option<Addr>,
        funds: &[Coin],
    ) -> AnyResult<AppResponse> {
        self.blockchain.execute_contract(
            sender.clone(),
            self.addr(),
            &ExecuteMsg::PlaceBet {
                result,
                min_odds,
                receiver,
            },
            funds,
        )
    }

    pub fn claim_winnings(
        &mut self,
        sender: &Addr,
        receiver: Option<Addr>,
    ) -> AnyResult<AppResponse> {
        self.blockchain.execute_contract(
            sender.clone(),
            self.addr(),
            &ExecuteMsg::ClaimWinnings { receiver },
            &[],
        )
    }
}

pub fn setup_blockchain_and_contract(
    admin: Addr,
    initial_balances: Vec<(Addr, Vec<Coin>)>,
    instantiate_msg: InstantiateMsg,
    instantiate_funds: Vec<Coin>,
) -> Result<BlockchainContract, AnyError> {
    let mut blockchain = AppBuilder::new()
        .with_api(MockApiBech32::new("neutron"))
        .build(|router, _, storage| {
            initial_balances.into_iter().for_each(|(addr, coins)| {
                router.bank.init_balance(storage, &addr, coins).unwrap();
            });
        });

    let code = Box::new(ContractWrapper::new(execute, instantiate, query));

    let code_id = blockchain.store_code(code);

    let contract_addr = blockchain.instantiate_contract(
        code_id,
        admin.clone(),
        &instantiate_msg,
        &instantiate_funds,
        "Market",
        None,
    );

    if contract_addr.is_err() {
        return Err(contract_addr.err().unwrap());
    }

    Ok(BlockchainContract {
        blockchain,
        contract_addr: contract_addr.unwrap(),
    })
}
