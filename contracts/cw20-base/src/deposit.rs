use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{coins, to_binary, Binary, CosmosMsg, HumanAddr, StdResult, WasmMsg, Uint128};

/// DepositStableMsg should be de/serialized under `DepositStable()` variant in a HandleMsg
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct DepositStableMsg {}

impl DepositStableMsg {
    /// serializes the message
    pub fn into_binary(self) -> StdResult<Binary> {
        let msg = DepositStableHandleMsg::DepositStable(self);
        to_binary(&msg)
    }

    /// creates a cosmos_msg sending this struct to the named contract
    pub fn into_cosmos_msg(self, contract_addr: HumanAddr, deposit_amount: Uint128) -> StdResult<CosmosMsg> {
        let msg = self.into_binary()?;
        let execute = WasmMsg::Execute {
            contract_addr,
            msg,
            send: coins(deposit_amount.u128() / 1000000, "uusd"),
        };
        Ok(execute.into())
    }
}

// This is just a helper to properly serialize the above message
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
enum DepositStableHandleMsg {
    DepositStable(DepositStableMsg),
}
