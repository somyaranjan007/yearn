use base_contract::msg::Cw20InstantiateMsg;
use base_contract::{VaultContract, VaultContractMethods, ContractInfo};
use cosmwasm_std::{coin, to_binary, CustomMsg, Empty, Response, SubMsg, Uint128, WasmMsg, StdError, QueryRequest,WasmQuery};
use cw_storage_plus::Item;
use mars_red_bank_types::red_bank;
use serde::de::{DeserializeOwned, Deserializer};
use serde::{Deserialize, Serialize, Serializer};
use cw20::{BalanceResponse, MinterResponse, TokenInfoResponse};
use cw20::Cw20QueryMsg::{Balance, TokenInfo, self};

pub struct VaultContractWrapper<'a>(pub VaultContract<'a>);

impl Serialize for VaultContractWrapper<'static> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str("VaultContractWrapper")
    }
}

impl<'de, 'a> Deserialize<'de> for VaultContractWrapper<'a> {
    fn deserialize<D>(deserializer: D) -> Result<VaultContractWrapper<'static>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VaultContractWrapperVisitor;

        impl<'de> serde::de::Visitor<'de> for VaultContractWrapperVisitor {
            type Value = VaultContractWrapper<'static>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("VaultContractWrapper")
            }

            fn visit_str<E>(self, value: &str) -> Result<VaultContractWrapper<'static>, E>
            where
                E: serde::de::Error,
            {
                if value == "VaultContractWrapper" {
                    Ok(VaultContractWrapper(VaultContract::default()))
                } else {
                    Err(serde::de::Error::unknown_variant(
                        value,
                        &["VaultContractWrapper"],
                    ))
                }
            }
        }

        deserializer.deserialize_str(VaultContractWrapperVisitor)
    }
}



impl<'a> VaultContractMethods for VaultContractWrapper<'a> {

    fn strategies(
        &self,
        _deps: cosmwasm_std::DepsMut,
        _env: cosmwasm_std::Env,
        _info: cosmwasm_std::MessageInfo,
    ) -> cosmwasm_std::StdResult<cosmwasm_std::Response> {
        let wrapper_contract = WRAPPER_CONTRACT.load(_deps.storage);
        match wrapper_contract {
            Ok(mut contract) => {
                let total_balance = contract.get_total_balance(_deps.as_ref(), _env.clone());
                match total_balance {
                    Ok(balance) => {
                        let convertbalance: u128 = balance.balance.u128();
                        let execute_deposit_tx = WasmMsg::Execute {
                            contract_addr:
                                "osmo1g30recyv8pfy3qd4qn3dn7plc0rn5z68y5gn32j39e96tjhthzxsw3uvvu"
                                    .to_string(),
                            msg: to_binary(&red_bank::ExecuteMsg::Deposit {
                                on_behalf_of: Some(_env.contract.address.to_string()),
                            })?,
                            funds: vec![coin(convertbalance, "osmo")],
                        };
                        const EXECUTE_DEPOSIT_ID: u64 = 1u64;
                        let _submessage =
                            SubMsg::reply_on_success(execute_deposit_tx, EXECUTE_DEPOSIT_ID);
                        Ok(Response::new()
                            .add_attribute("method", "strategies")
                            .add_submessage(_submessage))
                    }
                    Err(_) => {
                        return Err(cosmwasm_std::StdError::GenericErr {
                            msg: "Unable to Fetch Balance".to_string(),
                        });
                    }
                }
            }
            Err(_) => {
                return Err(cosmwasm_std::StdError::GenericErr {
                    msg: "Contract Not Found".to_string(),
                });
            }
        }
    }

    fn before_withdraw(
        &self,
        _deps: cosmwasm_std::DepsMut,
        _env: cosmwasm_std::Env,
        _info: cosmwasm_std::MessageInfo,
    ) -> cosmwasm_std::StdResult<Response> {
        let wrapper_contract = WRAPPER_CONTRACT.load(_deps.storage);
        match wrapper_contract {
            Ok(mut contract) => {
                let execute_withdraw_tx = WasmMsg::Execute {
                    contract_addr: "osmo1g30recyv8pfy3qd4qn3dn7plc0rn5z68y5gn32j39e96tjhthzxsw3uvvu".to_string(),
                    msg: to_binary(&red_bank::ExecuteMsg::Withdraw {
                        denom: "osmo".to_string(),
                        amount: None,
                        recipient: Some(_env.contract.address.to_string()),
                    })?,
                    funds: vec![],
                };
                const EXECUTE_WITHDRAW_ID: u64 = 2u64;
                let _submessage = SubMsg::reply_on_success(execute_withdraw_tx, EXECUTE_WITHDRAW_ID);
                Ok(Response::new()
                    .add_attribute("method", "strategies_withdraw")
                    .add_submessage(_submessage))
            }
            Err(_) => {
                return Err(cosmwasm_std::StdError::GenericErr {
                    msg: "contract not found".to_string(),
                });
            }
        }

        
    }
    
    fn after_deposit(&self, _deps: cosmwasm_std::DepsMut, _env: cosmwasm_std::Env, _info: cosmwasm_std::MessageInfo) -> cosmwasm_std::StdResult<Response>{
        Ok(Response::new())
    }

    fn before_deposit(&self, _deps: cosmwasm_std::DepsMut, _env: cosmwasm_std::Env, _info: cosmwasm_std::MessageInfo) -> cosmwasm_std::StdResult<Response> {
        Ok(Response::new())

    }

    fn after_withdraw(&self, _deps: cosmwasm_std::DepsMut, _env: cosmwasm_std::Env, _info: cosmwasm_std::MessageInfo) -> cosmwasm_std::StdResult<Response> {
        Ok(Response::new())

    }
    


}

pub const WRAPPER_CONTRACT: Item<VaultContractWrapper<'static>> = Item::new("wrapper_contract");
