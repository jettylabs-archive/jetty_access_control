use serde::{Deserialize, Serialize};
use structmap::FromMap;
use structmap_derive::FromMap;

/// Snowflake Table entry.
#[derive(FromMap, Clone, Default, Deserialize, Serialize, Debug)]
pub struct Object {
    /// The Table name in Snowflake.
    pub name: String,
    pub schema_name: String,
    pub database_name: String,
    pub kind: String,
}
