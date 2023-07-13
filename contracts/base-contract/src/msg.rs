use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Uint128};
use serde::{ Serialize, Deserialize };
use schemars::JsonSchema;
use cw20::{MinterResponse, Cw20Coin, Logo};
use std::fmt;

#[cw_serde]
pub struct VaultInstantiateMsg {
    pub supported_token: String,
    pub vault_owner: String,
}

#[cw_serde]
pub enum VaultExecuteMsg {
    Receive(Cw20ReceiveMsg),
    Strategies {},
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Cw20ReceiveMsg {
    pub sender: String,
    pub amount: Uint128,
    pub msg: Binary,
    }

impl fmt::Display for Cw20ReceiveMsg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "sender:{} token_id:{} msg:{}",
            self.sender,
            self.amount,
            self.msg.to_string()
        )
    }
}

#[cw_serde]
pub struct SendCw20Msg {
    pub message: String,
    pub address: String,
}

#[cw_serde]
pub struct InstantiateMarketingInfo {
    pub project: Option<String>,
    pub description: Option<String>,
    pub marketing: Option<String>,
    pub logo: Option<Logo>,
}

#[cw_serde]
pub struct Cw20InstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub initial_balances: Vec<Cw20Coin>,
    pub mint: Option<MinterResponse>,
    pub marketing: Option<InstantiateMarketingInfo>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(TotalBalanceResponse)]
    TotalBalance {},

    #[returns(TotalVtokenResponse)]
    TotalSupply {},

    #[returns(SupportedTokenResponse)]
    SupportedToken {},

    #[returns(VTokenResponse)]
    Vtoken {},
}

#[cw_serde] 
pub struct TotalBalanceResponse {
    pub balance: Uint128
}

#[cw_serde] 
pub struct TotalVtokenResponse {
    pub total_supply: Uint128
}

#[cw_serde] 
pub struct SupportedTokenResponse {
    pub supported_token: String
}

#[cw_serde] 
pub struct VTokenResponse {
    pub vtoken: String
}