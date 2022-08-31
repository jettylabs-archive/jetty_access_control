use serde::{Deserialize, Serialize};
use structmap::FromMap;
use structmap_derive::FromMap;

/// Snowflake Schema entry.
#[derive(FromMap, Clone, Default, Deserialize, Serialize, Debug)]
pub struct Schema {
    /// The schema name in Snowflake.
    pub name: String,
    pub database_name: String,
}
