use cosmwasm_schema::cw_serde;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Vault {
    pub name: String,
    pub symbol: String,
    pub vault_id: String,
    pub vault_address: String,
    pub vault_owner: String,
}

// VAULT state is used to gather the vault data
pub const VAULT_RECORD: Item<Vec<Vault>> = Item::new("vault created");