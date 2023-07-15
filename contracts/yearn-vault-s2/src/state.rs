use base_contract::{ContractInfo, VaultContract, VaultContractMethods};
use cosmwasm_std::{coin, to_binary, Response, SubMsg, WasmMsg};
use cw_storage_plus::Item;
use mars_red_bank_types::red_bank;
use serde::de::Deserializer;
use serde::{Deserialize, Serialize, Serializer};

pub struct VaultContractWrapper(pub VaultContract);

impl Serialize for VaultContractWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str("VaultContractWrapper")
    }
}

impl<'de, 'a> Deserialize<'de> for VaultContractWrapper {
    fn deserialize<D>(deserializer: D) -> Result<VaultContractWrapper, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VaultContractWrapperVisitor;

        impl<'de> serde::de::Visitor<'de> for VaultContractWrapperVisitor {
            type Value = VaultContractWrapper;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("VaultContractWrapper")
            }

            fn visit_str<E>(self, value: &str) -> Result<VaultContractWrapper, E>
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

impl<'a> VaultContractMethods for VaultContractWrapper {
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
            Ok(_contract) => {
                let execute_withdraw_tx = WasmMsg::Execute {
                    contract_addr:
                        "osmo1g30recyv8pfy3qd4qn3dn7plc0rn5z68y5gn32j39e96tjhthzxsw3uvvu"
                            .to_string(),
                    msg: to_binary(&red_bank::ExecuteMsg::Withdraw {
                        denom: "osmo".to_string(),
                        amount: None,
                        recipient: Some(_env.contract.address.to_string()),
                    })?,
                    funds: vec![],
                };
                const EXECUTE_WITHDRAW_ID: u64 = 2u64;
                let _submessage =
                    SubMsg::reply_on_success(execute_withdraw_tx, EXECUTE_WITHDRAW_ID);
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

    fn after_deposit(
        &self,
        _deps: cosmwasm_std::DepsMut,
        _env: cosmwasm_std::Env,
        _info: cosmwasm_std::MessageInfo,
    ) -> cosmwasm_std::StdResult<Response> {
        Ok(Response::new())
    }

    fn before_deposit(
        &self,
        _deps: cosmwasm_std::DepsMut,
        _env: cosmwasm_std::Env,
        _info: cosmwasm_std::MessageInfo,
    ) -> cosmwasm_std::StdResult<Response> {
        Ok(Response::new())
    }

    fn after_withdraw(
        &self,
        _deps: cosmwasm_std::DepsMut,
        _env: cosmwasm_std::Env,
        _info: cosmwasm_std::MessageInfo,
    ) -> cosmwasm_std::StdResult<Response> {
        Ok(Response::new())
    }

    fn contract_info_state(&mut self) -> &mut Item<'static, ContractInfo> {
        &mut self.0.contract_info
    }

    fn vtoken_address_state(&mut self) -> &mut Item<'static, String> {
        &mut self.0.vtoken_address
    }
}

pub const WRAPPER_CONTRACT: Item<VaultContractWrapper> = Item::new("wrapper_contract");
