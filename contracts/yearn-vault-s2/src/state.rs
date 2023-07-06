use base_contract::{VaultContract, VaultContractMethods};
use cosmwasm_std::{coins, to_binary, CustomMsg, Empty, SubMsg, WasmMsg, Response};
use cw_storage_plus::Item;
use mars_red_bank_types::red_bank;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct VaultContractWrapper<'a, T, C, E, Q>(pub VaultContract<'a, T, C, E, Q>);

impl<'a, T, C, E, Q> VaultContractMethods<T, C> for VaultContract<'static, (), C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    Q: CustomMsg,
    E: CustomMsg,
    C: CustomMsg,
{
    fn strategies(
        &self,
        _deps: cosmwasm_std::DepsMut,
        _env: cosmwasm_std::Env,
        _info: cosmwasm_std::MessageInfo,
    ) -> cosmwasm_std::StdResult<cosmwasm_std::Response> {
        // osmo1g30recyv8pfy3qd4qn3dn7plc0rn5z68y5gn32j39e96tjhthzxsw3uvvu  contract_addr
        let wrapper_contract = WRAPPER_CONTRACT.load(_deps.storage);
        match wrapper_contract {
            Ok(contract) => {
                let total_balance = contract.get_total_balance(_deps, _env);
                match total_balance {
                    Ok(balance) => {
                        let execute_deposit_tx = WasmMsg::Execute {
                            contract_addr: "osmo1g30recyv8pfy3qd4qn3dn7plc0rn5z68y5gn32j39e96tjhthzxsw3uvvu".to_string(),
                            msg: to_binary(&red_bank::ExecuteMsg::Deposit {
                                on_behalf_of: _env.contract.address,
                            }),
                            funds: vec![coins(balance, "osmo")],
                        };

                        const EXECUTE_DEPOSIT_ID: u64 = 1u64;
                        let _submessage =
                            SubMsg::reply_on_success(execute_deposit_tx, EXECUTE_DEPOSIT_ID);

                        Ok(Response::new().add_attribute("method", "strategies").add_submessage(_submessage))
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
}

pub const WRAPPER_CONTRACT: Item<VaultContractWrapper<Empty, Empty, Empty, Empty>> =
    Item::new("wrapper_contract");
