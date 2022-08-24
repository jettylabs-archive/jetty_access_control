use serde::Deserialize;
use structmap::FromMap;
use structmap_derive::FromMap;

/// Snowflake Database entry.
#[derive(FromMap, Default, Deserialize, Debug)]
pub struct Database {
    /// The database name in Snowflake.
    pub name: String,
}
