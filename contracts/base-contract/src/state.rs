use std::ops::{Div, Mul, Sub};

use crate::{ContractError, VTokenResponse};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    from_binary, to_binary, Deps, DepsMut, Empty, Env, MessageInfo, Querier, QuerierWrapper,
    QueryRequest, Reply, Response, StdError, StdResult, SubMsg, Uint128, WasmMsg, WasmQuery,
};
use cw0::{parse_reply_instantiate_data, parse_reply_execute_data};
use cw20::Cw20QueryMsg::{self, Balance, TokenInfo};
use cw20::{BalanceResponse, MinterResponse, TokenInfoResponse};
use cw_storage_plus::Item;
use serde::{Deserialize, Serialize};

use crate::msg::{
    Cw20InstantiateMsg, Cw20ReceiveMsg, SendCw20Msg, TotalBalanceResponse, TotalVtokenResponse,
    VaultInstantiateMsg, SupportedTokenResponse,
};

#[cw_serde]
pub struct ContractInfo {
    pub contract_owner: String,
    pub supported_token: String,
}

pub struct VaultContract {
    pub contract_info: Item<'static, ContractInfo>,
    pub vtoken_address: Item<'static, String>,
}

// pub const CONTRACT_INFO: Item<ContractInfo> = Item::new("contract_info");
// pub const VTOKEN_ADDRESS: Item<String> = Item::new("vtoken_address");

impl VaultContract {
    fn new() -> Self {
        Self {
            contract_info: Item::new("contract_info"),
            vtoken_address: Item::new("vtoken_address"),
        }
    }
}

impl Default for VaultContract {
    fn default() -> Self {
        Self::new()
    }
}

// impl<'a> VaultContractMethods for VaultContract<'static> {

//     fn strategies(&self, _deps: DepsMut, _env: Env, _info: MessageInfo) -> StdResult<Response> {
//         Ok(Response::new())
//     }

//     fn before_deposit(&self, _deps: DepsMut, _env: Env, _info: MessageInfo) -> StdResult<Response> {
//         Ok(Response::new())

//     }

//     fn after_deposit(&self, _deps: DepsMut, _env: Env, _info: MessageInfo) -> StdResult<Response> {
//         Ok(Response::new())

//     }

//     fn before_withdraw(&self, _deps: DepsMut,_env: Env,_info: MessageInfo,) -> StdResult<Response> {
//         Ok(Response::new())

//     }

//     fn after_withdraw(&self, _deps: DepsMut, _env: Env, _info: MessageInfo) -> StdResult<Response> {
//         Ok(Response::new())

//     }

//     fn vtoken_address_state(&mut self) -> &mut Item<'static, String> {
//         &mut self.vtoken_address
//     }

//     fn contract_info_state(&mut self) -> &mut Item<'static, ContractInfo> {
//         &mut self.contract_info
//     }

//     // Cosmwasm Execute msg function

//     // Cosmwasm Reply msg function

//     // Cosmwasm Query msg function

// }

pub trait VaultContractMethods {
    fn contract_info_state(&mut self) -> &mut Item<'static, ContractInfo>;
    fn vtoken_address_state(&mut self) -> &mut Item<'static, String>;

    // Cosmwasm End point message function
    fn instantiate(
        &mut self,
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: VaultInstantiateMsg,
    ) -> StdResult<Response> {
        let info = ContractInfo {
            contract_owner: _msg.vault_owner,
            supported_token: _msg.supported_token.clone(),
        };

        // CONTRACT_INFO.save(_deps.storage, &info)?;

        let save_contract_info = self.contract_info_state().save(_deps.storage, &info);

        match save_contract_info {
            Ok(_) => {}
            Err(_) => {
                return Err(StdError::GenericErr {
                    msg: "Unable to save contract info".to_string(),
                });
            }
        }

        let token_info_query = TokenInfo {};

        let supported_token_query: Result<TokenInfoResponse, StdError> =
            _deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: _msg.supported_token.clone(),
                msg: to_binary(&token_info_query)?,
            }));

        match supported_token_query {
            Ok(token_data) => {
                const VTOKEN_INSTANTIATE_REPLY_ID: u64 = 1u64;

                let vtoken_instantiate_tx = WasmMsg::Instantiate {
                    admin: None,
                    code_id: 846,
                    msg: to_binary(&Cw20InstantiateMsg {
                        name: "v".to_string() + &token_data.name,
                        symbol: "V".to_string() + &token_data.symbol,
                        decimals: 18,
                        initial_balances: vec![],
                        mint: Some(MinterResponse {
                            minter: _env.contract.address.to_string(),
                            cap: None,
                        }),
                        marketing: None,
                    })?,
                    funds: vec![],
                    label: "instantiate vtoken contract".to_string(),
                };

                let _submessage: SubMsg<Empty> =
                    SubMsg::reply_on_success(vtoken_instantiate_tx, VTOKEN_INSTANTIATE_REPLY_ID);

                Ok(Response::new().add_attribute("method", "instantiate").add_submessage(_submessage))
            }
            Err(_) => {
                return Err(StdError::GenericErr {
                    msg: "querier me error h".to_string(),
                });
            }
        }
    }

    // Cosmwasm Execute msg function
    fn handle_cw20_receive(
        &mut self,
        mut _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: Cw20ReceiveMsg,
    ) -> StdResult<Response> {
        const DEPOSIT_MESSAGE: &str = "Deposit";
        const WITHDRAW_MESSAGE: &str = "Withdraw";

        let _send_cw20: SendCw20Msg = from_binary(&_msg.msg)?;

        match _send_cw20.message.as_str() {
            DEPOSIT_MESSAGE => {
                // self.before_deposit(_deps, _env.clone(), _info);

                let token_address = match self.contract_info_state().load(_deps.storage) {
                    Ok(response) => response.supported_token,
                    Err(_) => {
                        return Err(StdError::GenericErr {
                            msg: "Unable to fetch token address!".to_string(),
                        });
                    }
                };

                // let token_address = match CONTRACT_INFO.load(_deps.storage) {
                //     Ok(response) => response.supported_token,
                //     Err(_) => {
                //         return Err(StdError::GenericErr {
                //             msg: "Unable to fetch token address!".to_string(),
                //         });
                //     }
                // };

                if _send_cw20.address != token_address {
                    return Err(StdError::GenericErr {
                        msg: "Vault doesn't support this token!".to_string(),
                    });
                }

                let mint_amount: Uint128;

                let total_supply = match self.get_total_supply(_deps.as_ref(), _env.clone()) {
                    Ok(response) => response.total_supply,
                    Err(_) => {
                        Uint128::from(0u128)
                        
                    }
                };

                
                if total_supply.is_zero()  {
                    mint_amount = _msg.amount;
                } else {
                    let total_balance = self.get_total_balance(_deps.as_ref(), _env);

                    match total_balance {
                        Ok(response) => {
                            mint_amount = total_supply
                                .div(response.balance.sub(_msg.amount))
                                .mul(_msg.amount);
                        }
                        Err(_) => {
                            return Err(StdError::GenericErr {
                                msg: "Unable to fetch total balance!".to_string(),
                            });
                        }
                    }
                }

                // let vtoken_address = VTOKEN_ADDRESS.load(_deps.storage);

                let vtoken_address = self.vtoken_address_state().load(_deps.storage);

                match vtoken_address {
                    Ok(address) => {
                        let execute_mint_tx = WasmMsg::Execute {
                            contract_addr: address,
                            msg: to_binary(&cw20::Cw20ExecuteMsg::Mint {
                                recipient: _msg.sender,
                                amount: mint_amount,
                            })?,
                            funds: vec![],
                        };

                        const EXECUTE_MINT_ID: u64 = 2u64;

                        let _submessage: SubMsg<Empty> =
                            SubMsg::reply_on_error(execute_mint_tx, EXECUTE_MINT_ID);
                    }
                    Err(_) => {
                        return Err(StdError::GenericErr {
                            msg: "Unable to find vtoken address!".to_string(),
                        });
                    }
                }
            }
            WITHDRAW_MESSAGE => {
                //for withdraw the depositing balance in redBank
                self.before_withdraw(_deps.branch(), _env.clone(), _info)?;

                match self.vtoken_address_state().load(_deps.storage) {
                    Ok(vtoken) => {
                        if vtoken == _send_cw20.address {
                            let total_supply =
                                match self.get_total_supply(_deps.as_ref(), _env.clone()) {
                                    Ok(response) => response.total_supply,
                                    Err(_) => {
                                        return Err(StdError::GenericErr {
                                            msg: "Unable to fetch the total supply".to_string(),
                                        });
                                    }
                                };

                            let total_balance = match self.get_total_balance(_deps.as_ref(), _env) {
                                Ok(response) => response.balance,
                                Err(_) => {
                                    return Err(StdError::GenericErr {
                                        msg: "Unable to fetch the total balance".to_string(),
                                    });
                                }
                            };

                            let transfer_amount = total_balance.div(total_supply).mul(_msg.amount);

                            let token_address = match self.contract_info_state().load(_deps.storage)
                            {
                                Ok(response) => response.supported_token,
                                Err(_) => {
                                    return Err(StdError::GenericErr {
                                        msg: "Token address not found!".to_string(),
                                    })
                                }
                            };

                            // let token_address = match CONTRACT_INFO.load(_deps.storage) {
                            //     Ok(response) => response.supported_token,
                            //     Err(_) => {
                            //         return Err(StdError::GenericErr {
                            //             msg: "Token address not found!".to_string(),
                            //         })
                            //     }
                            // };

                            let execute_mint_tx = WasmMsg::Execute {
                                contract_addr: token_address,
                                msg: to_binary(&cw20::Cw20ExecuteMsg::Transfer {
                                    recipient: _msg.sender,
                                    amount: transfer_amount,
                                })?,
                                funds: vec![],
                            };

                            const EXECUTE_MINT_ID: u64 = 3u64;
                            let _submessage: SubMsg<Empty> =
                                SubMsg::reply_on_error(execute_mint_tx, EXECUTE_MINT_ID);
                        }
                    }
                    Err(_) => {
                        return Err(StdError::GenericErr {
                            msg: "Vault doesn't assigned any vToken".to_string(),
                        });
                    }
                };
            }
            _message => {
                return Err(StdError::GenericErr {
                    msg: "Invalid Request!".to_string(),
                })
            }
        }

        Ok(Response::new().add_attribute("method", "handle_cw20_receive"))
    }

    fn strategies(&self, _deps: DepsMut, _env: Env, _info: MessageInfo) -> StdResult<Response>;

    // Extra function for deposit
    fn before_deposit(&self, _deps: DepsMut, _env: Env, _info: MessageInfo) -> StdResult<Response>;

    fn after_deposit(&self, _deps: DepsMut, _env: Env, _info: MessageInfo) -> StdResult<Response>;

    // Extra function for withdraw
    fn before_withdraw(&self, _deps: DepsMut, _env: Env, _info: MessageInfo)
        -> StdResult<Response>;

    fn after_withdraw(&self, _deps: DepsMut, _env: Env, _info: MessageInfo) -> StdResult<Response>;

    // Cosmwasm Query msg function
    fn get_total_balance(&mut self, _deps: Deps, _env: Env) -> StdResult<TotalBalanceResponse> {
        let total_balance: Uint128;

        // let token_address = CONTRACT_INFO.load(_deps.storage);

        let token_address = self.contract_info_state().load(_deps.storage);

        match token_address {
            Ok(response) => {
                // previous query now updated
                // let query = WasmQuery::Smart {
                //     contract_addr: response.supported_token.clone(),
                //     msg: to_binary(&Balance {
                //         address: _env.contract.address.to_string(),
                //     })?,
                // };

                // let data: StdResult<BalanceResponse> = _deps
                //     .querier 
                //     .query_wasm_smart(response.supported_token, &query);
                let data:Result<BalanceResponse,StdError> = _deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart { 
                    contract_addr: response.supported_token.clone(),
                     msg: to_binary(&Balance {
                                address: _env.contract.address.to_string(),
                            })?,
                    }));

                match data {
                    Ok(response) => {
                        total_balance = response.balance;
                    }
                    Err(_) => {
                        return Err(StdError::GenericErr {
                            msg: "Unable to Fetch Balance".to_string(),
                        });
                    }
                };
            }
            Err(_) => {
                return Err(StdError::GenericErr {
                    msg: "Unable to find token address!".to_string(),
                });
            }
        };

        Ok(TotalBalanceResponse {
            balance: total_balance,
        })
    }

    fn get_total_supply(&mut self, _deps: Deps, _env: Env) -> StdResult<TotalVtokenResponse> {
        // let vtoken_address = VTOKEN_ADDRESS.load(_deps.storage);

        let vtoken_address = self.vtoken_address_state().load(_deps.storage);

        match vtoken_address {
            Ok(address) => {

                // let query = WasmQuery::Smart {
                //     contract_addr: address.clone(),
                //     msg: to_binary(&TokenInfo {})?,
                // };

                // let vtoken_data: StdResult<TokenInfoResponse> =
                //     _deps.querier.query_wasm_smart(address, &query);


                  let vtoken_data:Result<TotalVtokenResponse,StdError> = _deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                     contract_addr: address.clone(),
                     msg: to_binary(&TokenInfo {})?,
                     })); 

                match vtoken_data {
                    Ok(token) => Ok(TotalVtokenResponse {
                        total_supply: token.total_supply,
                    }),
                    Err(_) => {
                        return Err(StdError::GenericErr {
                            msg: "Unable to fetch vTokens data".to_string(),
                        });
                    }
                }
            }
            Err(_) => {
                return Err(StdError::GenericErr {
                    msg: "Unable to find vtoken address!".to_string(),
                });
            }
        }
    }

    fn get_supported_token(&mut self,_deps:Deps, _env:Env ) -> StdResult<SupportedTokenResponse> {
        let supported_token = self.contract_info_state().load(_deps.storage);
        match supported_token {
            Ok(address) => {
                Ok(SupportedTokenResponse{
                    supported_token: address.supported_token,
                })

            },
            Err(_) => {
                return Err(StdError::GenericErr { msg: "Unable to find Supported Token".to_string() });
            }
            
        }
    }

    fn get_vtoken(&mut self,_deps:Deps, _env:Env) -> StdResult<VTokenResponse>{
        let vtoken = self.vtoken_address_state().load(_deps.storage);
        match vtoken {
            Ok(address) =>{
                Ok(VTokenResponse{
                    vtoken: address,
                })
            },
            Err(_) => {
                return Err(StdError::GenericErr { msg: "Unable to fetch vtoken address".to_string() });
            }
        }
    }

    // Cosmwasm Reply msg function
    fn handle_cw20_instantiate(&mut self, _deps: DepsMut, _msg: Reply) -> StdResult<Response> {
        let result = parse_reply_instantiate_data(_msg);

        match result {
            Ok(response) => {
                // VTOKEN_ADDRESS.save(_deps.storage, &response.contract_address)?;

                let handle_save = self
                    .vtoken_address_state()
                    .save(_deps.storage, &response.contract_address);
                match handle_save {
                    Ok(_) => Ok(Response::new().add_attribute("method", "handle_cw20_instantiate")),
                    Err(_) => {
                        return Err(StdError::GenericErr {
                            msg: "handle save err".to_string(),
                        })
                    }
                }
            }
            Err(err) => {
                return Err(StdError::GenericErr {
                    msg: err.to_string(),
                })
            }
        }
    }
    
    fn handle_register_reply(&mut self, _deps: DepsMut, _msg: Reply) -> StdResult<Response>{
        // let result = parse_reply_execute_data(_msg);
        Ok(Response::new().add_attribute("method", "handle_register"))

        // match result {
        //     Ok(_) => Ok(Response::new().add_attribute("method", "handle_register")),
        //     Err(err) => {
        //         return Err(StdError::GenericErr { msg: err.to_string() })
        //     }
            
        // }

    }

}
