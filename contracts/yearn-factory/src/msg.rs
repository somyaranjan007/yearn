use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::state::Vault;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    RegisterVault(VaultData),
}

#[cw_serde]
#[serde(rename_all = "snake_case")]
pub struct VaultData {
    pub name: String,
    pub symbol: String,
    pub vault_address: String,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetVaultRecordResponse)]
    GetVaults {},
}

#[cw_serde]
pub struct GetVaultRecordResponse {
    pub vault_array: Vec<Vault>,
}