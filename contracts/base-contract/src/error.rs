use cosmwasm_std::StdError;
use thiserror::Error;
use serde::{ Serialize, Deserialize, Deserializer, Serializer};

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}

impl Serialize for ContractError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer {
        serializer.serialize_str("ContractError")   
    }
}

impl<'de> Deserialize<'de> for ContractError {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ContractErrorVisitor;

        impl<'de> serde::de::Visitor<'de> for ContractErrorVisitor {
            type Value = ContractError;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("ContractError")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if value == "ContractError" {
                    // Return the desired variant based on the value
                    Ok(ContractError::Unauthorized {})
                } else {
                    Err(serde::de::Error::unknown_variant(value, &["ContractError"]))
                }
            }
        }

        deserializer.deserialize_str(ContractErrorVisitor)
    }
}

