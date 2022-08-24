use serde::Deserialize;
use structmap::FromMap;
use structmap_derive::FromMap;

/// Snowflake Schema entry.
#[derive(FromMap, Default, Deserialize, Debug)]
pub struct Schema {
    /// The schema name in Snowflake.
    pub name: String,
}
