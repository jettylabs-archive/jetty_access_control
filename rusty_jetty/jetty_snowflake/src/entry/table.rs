use serde::Deserialize;
use structmap::FromMap;
use structmap_derive::FromMap;

/// Snowflake Table entry.
#[derive(FromMap, Default, Deserialize, Debug)]
pub struct Table {
    /// The Table name in Snowflake.
    pub name: String,
    pub database_name: String,
    pub schema_name: String,
}
