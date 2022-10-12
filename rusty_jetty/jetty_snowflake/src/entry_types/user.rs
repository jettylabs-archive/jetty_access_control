use anyhow::{bail, Result};
use serde::{de, Deserialize, Serialize};

/// Snowflake User entry.
#[derive(Clone, Deserialize, Serialize, Debug, Default)]
pub struct User {
    pub name: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub login_name: String,
    pub display_name: String,
    #[serde(deserialize_with = "deserialize_bool")]
    pub disabled: bool,
}

fn deserialize_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: String = de::Deserialize::deserialize(deserializer)?;

    match s.as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => panic!("unknown value for disabled field"),
    }
}
