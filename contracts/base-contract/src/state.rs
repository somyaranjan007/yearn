use std::ops::{Div, Mul, Sub};

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    from_binary, to_binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdError,
    StdResult, SubMsg, Uint128, WasmMsg, WasmQuery, Reply
};
use cw20::Cw20QueryMsg::{Balance, TokenInfo};
use cw20::{BalanceResponse, MinterResponse, TokenInfoResponse};
use cw_storage_plus::Item;
use cw0::parse_reply_instantiate_data;

use crate::msg::{
    Cw20InstantiateMsg, Cw20ReceiveMsg, SendCw20Msg, TotalBalanceResponse,
    TotalVtokenResponse, VaultInstantiateMsg,
};

use crate::ContractError;

#[cw_serde]
pub struct ContractInfo {
    pub contract_owner: String,
    pub supported_token: String,
}

pub struct VaultContract {}

pub const CONTRACT_INFO: Item<ContractInfo> = Item::new("contract_info");
pub const VTOKEN_ADDRESS: Item<String> = Item::new("vtoken_address");

pub trait VaultContractMethods {
    // Cosmwasm End point message function
    fn instantiate(
        &self,
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: VaultInstantiateMsg,
    ) -> StdResult<Response> {
        unimplemented!();
    }

    // Cosmwasm Execute msg function
    fn handle_cw20_receive(
        &self,
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: Cw20ReceiveMsg,
    ) -> StdResult<Response> {
        unimplemented!();
    }

    fn strategies(&self, _deps: DepsMut, _env: Env, _info: MessageInfo) -> StdResult<Response> {
        unimplemented!();
    }

    // Extra function for deposit
    fn before_deposit(&self, _deps: DepsMut, _env: Env, _info: MessageInfo) -> StdResult<Response> {
        unimplemented!();
    }

    fn after_deposit(&self, _deps: DepsMut, _env: Env, _info: MessageInfo) -> StdResult<Response> {
        unimplemented!();
    }

    // Extra function for withdraw
    fn before_withdraw(&self, _deps: DepsMut, _env: Env, _info: MessageInfo) -> StdResult<Response> {
        unimplemented!();
    }

    fn after_withdraw(&self, _deps: DepsMut, _env: Env, _info: MessageInfo) -> StdResult<Response> {
        unimplemented!();
    }

    // Cosmwasm Query msg function
    fn get_total_balance(&self, _deps: Deps, _env: Env) -> Result<TotalBalanceResponse, ContractError> {
        unimplemented!();
    }

    fn get_total_supply(&self, _deps: Deps, _env: Env) -> Result<TotalVtokenResponse, ContractError> {
        unimplemented!();
    }

    // Cosmwasm Reply msg function
    fn handle_cw20_instantiate(&self, _deps: DepsMut, _msg: Reply) -> StdResult<Response> {
        unimplemented!();
    }
}

impl VaultContractMethods for VaultContract {
    fn instantiate(
        &self,
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: VaultInstantiateMsg,
    ) -> StdResult<Response> {
        
        let info = ContractInfo {
            contract_owner: _msg.vault_owner,
            supported_token: _msg.supported_token.clone(),
        };

        CONTRACT_INFO.save(_deps.storage, &info)?;

        let supported_token_query = WasmQuery::Smart {
            contract_addr: _msg.supported_token.clone(),
            msg: to_binary(&TokenInfo {})?,
        };

        let supported_token_data: StdResult<TokenInfoResponse> = _deps
            .querier
            .query_wasm_smart(_msg.supported_token, &supported_token_query);

        match supported_token_data {
            Ok(token_data) => {
                const VTOKEN_INSTANTIATE_REPLY_ID: u64 = 1u64;

                let vtoken_instantiate_tx = WasmMsg::Instantiate {
                    admin: None,
                    code_id: 456,
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

                Ok(Response::new().add_attribute("method", "instantiate"))
            }
            Err(_) => {
                return Err(StdError::GenericErr {
                    msg: "Unable to fetch token data!".to_string(),
                });
            }
        }
    }

    fn handle_cw20_receive(
        &self,
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: Cw20ReceiveMsg,
    ) -> StdResult<Response> {
        const DEPOSIT_MESSAGE: &str = "Deposit";
        const WITHDRAW_MESSAGE: &str = "Withdraw";

        let _send_cw20: SendCw20Msg = from_binary(&_msg.msg)?;

        match _send_cw20.message.as_str() {
            DEPOSIT_MESSAGE => {
                let token_address = match CONTRACT_INFO.load(_deps.storage) {
                    Ok(response) => response.supported_token,
                    Err(_) => {
                        return Err(StdError::GenericErr {
                            msg: "Unable to fetch token address!".to_string(),
                        })
                    }
                };

                if _send_cw20.address != token_address {
                    return Err(StdError::GenericErr {
                        msg: "Vault doesn't support this token!".to_string(),
                    });
                }

                let mint_amount: Uint128;

                let total_supply = match self.get_total_supply(_deps.as_ref(), _env.clone()) {
                    Ok(response) => response.total_supply,
                    Err(_) => {
                        return Err(StdError::GenericErr {
                            msg: "Unable to fetch total supply!".to_string(),
                        })
                    }
                };

                if total_supply < Uint128::from(1u128) {
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

                let vtoken_address = VTOKEN_ADDRESS.load(_deps.storage);

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
                match VTOKEN_ADDRESS.load(_deps.storage) {
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

                            let token_address = match CONTRACT_INFO.load(_deps.storage) {
                                Ok(response) => response.supported_token,
                                Err(_) => {
                                    return Err(StdError::GenericErr {
                                        msg: "Token address not found!".to_string(),
                                    })
                                }
                            };

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

    fn get_total_balance(&self, _deps: Deps, _env: Env) -> Result<TotalBalanceResponse, ContractError> {
        let total_balance: Uint128;
        let token_address = CONTRACT_INFO.load(_deps.storage);

        match token_address {
            Ok(response) => {
                let query = WasmQuery::Smart {
                    contract_addr: response.supported_token.clone(),
                    msg: to_binary(&Balance {
                        address: _env.contract.address.to_string(),
                    })?,
                };

                let data: StdResult<BalanceResponse> = _deps
                    .querier
                    .query_wasm_smart(response.supported_token, &query);

                match data {
                    Ok(response) => {
                        total_balance = response.balance;
                    },
                    Err(_) => {
                        return Err(ContractError::CustomError { val: "Unable to Fetch Balance".to_string() });
                    }
                };
            }
            Err(_) => {
                return Err(ContractError::CustomError { val: "Unable to find token address!".to_string() });
            }
        };

        Ok(TotalBalanceResponse {
            balance: total_balance,
        })
    }

    fn get_total_supply(&self, _deps: Deps, _env: Env) -> Result<TotalVtokenResponse, ContractError> {
        let vtoken_address = VTOKEN_ADDRESS.load(_deps.storage);

        match vtoken_address {
            Ok(address) => {
                let query = WasmQuery::Smart {
                    contract_addr: address.clone(),
                    msg: to_binary(&TokenInfo {})?,
                };

                let vtoken_data: StdResult<TokenInfoResponse> =
                    _deps.querier.query_wasm_smart(address, &query);

                match vtoken_data {
                    Ok(token) => Ok(TotalVtokenResponse {
                        total_supply: token.total_supply,
                    }),
                    Err(_) => {
                        return Err(ContractError::CustomError { val: "Unable to fetch vTokens data".to_string() });
                    }
                }
            }
            Err(_) => {
                return Err(ContractError::CustomError { val: "Unable to find vtoken address!".to_string() });
            }
        }
    }

    fn handle_cw20_instantiate(&self, _deps: DepsMut, _msg: Reply) -> StdResult<Response> {
        let result = parse_reply_instantiate_data(_msg);

        match result {
            Ok(response) => {
                VTOKEN_ADDRESS.save(_deps.storage, &response.contract_address)?;
                Ok(Response::new().add_attribute("method", "handle_cw20_instantiate"))
            },
            Err(_) => {
                return Err(StdError::GenericErr { msg: "Unable to instantiate vtoken".to_string() })
            }
        }

        
    }
}
