use serde::{Deserialize, Serialize};
use structmap::FromMap;
use structmap_derive::FromMap;

/// Snowflake Database entry.
#[derive(FromMap, Clone, Default, Deserialize, Serialize, Debug)]
pub struct Database {
    /// The Database name in Snowflake.
    pub name: String,
}
