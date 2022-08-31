use serde::Deserialize;
use structmap::FromMap;
use structmap_derive::FromMap;

/// Snowflake Warehouse entry.
#[derive(FromMap, Default, Deserialize, Debug)]
pub struct Warehouse {
    /// The database name in Snowflake.
    pub name: String,
}
