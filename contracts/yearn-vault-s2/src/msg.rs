use cosmwasm_schema::{cw_serde, QueryResponses};
use base_contract::msg::Cw20ReceiveMsg;
use base_contract::{TotalBalanceResponse, TotalVtokenResponse};

/// Message type for `instantiate` entry_point
#[cw_serde]
pub struct InstantiateMsg {
    pub supported_token: String,
}

/// Message type for `execute` entry_point
#[cw_serde]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    Strategies {},
}

/// Message type for `migrate` entry_point
#[cw_serde]
pub enum MigrateMsg {}

/// Message type for `query` entry_point
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(TotalBalanceResponse)]
    TotalBalance {},

    #[returns(TotalVtokenResponse)]
    TotalSupply {},
}

// We define a custom struct for each query response
// #[cw_serde]
// pub struct YourQueryResponse {}
