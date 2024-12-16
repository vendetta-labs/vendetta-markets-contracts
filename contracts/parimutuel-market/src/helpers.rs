use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{to_json_binary, Addr, Coin, CosmosMsg, StdResult, WasmMsg};

use crate::msg::ExecuteMsg;

/// CwContract is a wrapper around Addr that provides a lot of helpers
/// for working with this contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct CwContract(pub Addr);

impl CwContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn call<T: Into<ExecuteMsg>>(&self, msg: T, funds: Vec<Coin>) -> StdResult<CosmosMsg> {
        let msg = to_json_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds,
        }
        .into())
    }
}
