#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult, to_binary, WasmMsg, SubMsg, Empty};
use cw2::set_contract_version;
// use cw_multi_test::Contract;
use cw_storage_plus::Item;

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{WRAPPER_CONTRACT,VaultContractWrapper };

use base_contract::{VaultContract, VaultInstantiateMsg, VaultContractMethods};
use yearn_factory::msg::{ExecuteMsg as FactoryExecuteMsg, VaultData};


// const CONTRACT: VaultContract = VaultContract::new()
// impl VaultContractMethods for VaultContractInherit {
//     fn strategies(&self, _deps: DepsMut, _env: Env, _info: MessageInfo) -> StdResult<Response> {
//         Ok(Response::new().add_attribute("method", "stategies"))
//     }
// }

/// version info for migration info
const CONTRACT_NAME: &str = "crates.io:yearn-vault-s2";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// pub type Contract = VaultContract<'static>;

/// Handling contract instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;


    let msg = VaultInstantiateMsg {
        supported_token: _msg.supported_token,
        vault_owner: info.clone().sender.to_string(),
    };
    
    let contract = VaultContract {
        contract_info: Item::new("contract_info"), 
        vtoken_address: Item::new("vtoken_address"),
    };


    let mut wrapper_contract = VaultContractWrapper(contract);
     
    // let instantiate_tx = contract.instantiate(deps.branch(), _env, info.clone(), msg);
    
    let instantiate_tx = wrapper_contract.instantiate(deps.branch(), _env.clone(), info.clone(), msg);

    match instantiate_tx {
        Ok(_response) => {
            WRAPPER_CONTRACT.save(deps.storage, &wrapper_contract )?;
            
        let factory_ex_txn=WasmMsg::Execute { 
            contract_addr: "osmo1a4hw32cu85704sarhfu06uysm292f58lh0s2xmj5c0w6458x5szs6sf7zl".to_string(), 
            msg: to_binary(&FactoryExecuteMsg::RegisterVault(VaultData { 
                name: "usdc".to_string(), 
                symbol:"USDC".to_string(), 
                vault_address: _env.contract.address.to_string(), 
            }))?, 
            funds: vec![]
        };
            
            let _sub_message: SubMsg<Empty> = SubMsg::reply_always(factory_ex_txn, 5);

            Ok(Response::new()
                .add_attribute("method", "instantiate"))
        },
        Err(err) => {
            return Err(err)
        }
    }


    
    // wrapper_contract.contract_info_state().

}


/// Handling contract execution
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> StdResult<Response> {
    let wrapper_contract = WRAPPER_CONTRACT.load(_deps.storage);

    match wrapper_contract {
        Ok(mut contract) => {
            match _msg {
                ExecuteMsg::Receive(cw20_receive_msg) => contract.handle_cw20_receive(_deps, _env, _info, cw20_receive_msg),
                ExecuteMsg::Strategies {} => contract.strategies(_deps, _env, _info)
            }
        },
        Err(_) => {
            return Err(cosmwasm_std::StdError::GenericErr { msg: "contract not found".to_string() });
        }
    }

    
}

/// Handling contract query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let wrapper_contract = WRAPPER_CONTRACT.load(_deps.storage);
    match wrapper_contract {
        Ok(mut contract) => {
            match msg {
                QueryMsg::TotalBalance {  } => to_binary(&contract.get_total_balance(_deps, _env)?),
                QueryMsg::TotalSupply {  } => to_binary(&contract.get_total_supply(_deps, _env)?),
                
            }

        },
        Err(_) => {
            
            return  Err(cosmwasm_std::StdError::GenericErr { msg: "contract not found".to_string() });
        }
    }

     
}

/// Handling submessage reply.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> StdResult<Response> {
    const VTOKEN_INTSANTIATE_REPLY_ID: u64 = 1u64;
    let wrapper_contract = WRAPPER_CONTRACT.load(_deps.storage);
    match wrapper_contract {
        Ok(mut contract) => {
            match _msg.id {
                VTOKEN_INSTANTIATE_REPLY_ID => contract.handle_cw20_instantiate(_deps, _msg),
                _id => {
                    return  Err(cosmwasm_std::StdError::GenericErr { msg: "Id is not defined".to_string() });
                }
                
            }
        },
        Err(_) => {
            return Err(cosmwasm_std::StdError::GenericErr { msg: "Contract not found".to_string() });
        },
    }
    
}

