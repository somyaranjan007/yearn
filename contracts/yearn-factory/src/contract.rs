#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use cw2::set_contract_version;

use crate::{error::ContractError, msg::*, state::*};

const CONTRACT_NAME: &str = "crates.io:yearn-factory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::RegisterVault(vault_data) => {
            execute::execute_register(_deps, _env, _info, vault_data)
        }
    }
}

pub mod execute {
    use super::*;

    pub fn execute_register(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        vault_data: VaultData,
    ) -> Result<Response, ContractError> {
        let set_vault_id: String;
        let copy_vault_data = &vault_data;

        let vault_record = VAULT_RECORD.load(_deps.storage);
        match &vault_record {
            Ok(existed_vault) => {
                set_vault_id = (existed_vault.len() + 1).to_string();
            }
            Err(_) => {
                set_vault_id = String::from("1");
            }
        }

        let vault = Vault {
            name: copy_vault_data.name.to_string(),
            symbol: copy_vault_data.symbol.to_string(),
            vault_id: set_vault_id,
            vault_address: copy_vault_data.vault_address.to_string(),
            vault_owner: _info.sender.to_string(),
        };

        match vault_record {
            Ok(existed_vault) => {
                for i in 0..existed_vault.len() {
                    if existed_vault[i].name == vault_data.name
                        || existed_vault[i].symbol == vault_data.symbol
                        || existed_vault[i].vault_address == vault_data.vault_address
                    {
                        return Err(ContractError::CustomError {
                            val: "vault already existed".to_string(),
                        });
                    }
                }

                let updated_record =
                    VAULT_RECORD.update(_deps.storage, |mut update_record| -> StdResult<_> {
                        update_record.push(vault);
                        Ok(update_record)
                    });

                match updated_record {
                    Ok(_) => {}
                    Err(_) => {
                        return Err(ContractError::CustomError {
                            val: "Vault creation failed!".to_string(),
                        });
                    }
                }
            }
            Err(_) => {
                let mut new_vault: Vec<Vault> = Vec::new();
                new_vault.push(vault);

                println!("new_vault: {:?}", new_vault);

                match VAULT_RECORD.save(_deps.storage, &new_vault) {
                    Ok(_) => {}
                    Err(_) => {
                        return Err(ContractError::CustomError {
                            val: "Vault creation failed!".to_string(),
                        });
                    }
                }
 
            }
        }

        Ok(Response::new().add_attribute("method", "execute_register"))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetVaults {} => to_binary(&query::get_vault_array(_deps, _env)?),
    }
}

pub mod query {
    use super::*;

    pub fn get_vault_array(
        _deps: Deps,
        _env: Env,
    ) -> StdResult<GetVaultRecordResponse> {
        let vault_record = VAULT_RECORD.load(_deps.storage)?;

        println!("vault_record: {:?}", vault_record);

        Ok(GetVaultRecordResponse {
            vault_array: vault_record,
        })
           
        
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> Result<Response, ContractError> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{ coins, from_binary};

    #[test]
    fn greet_query() {
        let mut deps = mock_dependencies();
        let info = mock_info("owner", &coins(1000, "earth"));
        let msg = InstantiateMsg {};
        let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        // assert_eq!(0, res.messages.len());

        let vault_date = VaultData {
            name: "usdc".to_string(),
            symbol: "USDC".to_string(),
            vault_address: "osmo1s3c55jg0scuyls8lr8tjhtxl5qrq0tamjwp6d90vwd0jz6f2jvtqad4ep9".to_string()
        };

        let msg = ExecuteMsg::RegisterVault(vault_date);
        let _res = execute(deps.as_mut(), mock_env(), info, msg);

        println!("execute_response: {:?}", _res);

        let msg = QueryMsg::GetVaults {  };
        let _res = query(deps.as_ref(), mock_env(), msg);

        match _res {
            Ok(data) => {
                let value: GetVaultRecordResponse = from_binary(&data).unwrap();
                println!("value: {:?}", value);

                // match value {
                //     Ok(response) => {
                //         println!("vault_array: {:?}", response.vault_array);
                //     },
                //     Err(err) => {
                //         println!("vault_array_err: {:?}", err);
                //     }
                // }
            }, 
            Err(err) => {
                println!("err: {:?}", err);
            }
        }

        

    }
}