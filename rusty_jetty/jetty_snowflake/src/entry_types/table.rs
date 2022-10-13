use serde::{Deserialize, Serialize};

/// Snowflake Table entry.
#[derive(Clone, Default, Deserialize, Serialize, Debug)]
pub struct Table {
    /// The Table name in Snowflake.
    pub name: String,
    pub schema_name: String,
    pub database_name: String,
}
