use cosmwasm_schema::cw_serde;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Vault {
    pub name: String,
    pub symbol: String,
    pub vault_id: String,
    pub vault_address: String,
    pub vault_tokens_address: String,
}

// VAULT state is used to gather the vault data
pub const VAULT_RECORD: Item<Vec<Vault>> = Item::new("vault created");

// VAULT_OWNER MAP have data of how much vaults user have 
pub const VAULT_OWNER: Map<&str,Vec<&str>> = Map::new("vault owner");
