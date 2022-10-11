use serde::Deserialize;

/// Snowflake Warehouse entry.
#[derive(Default, Deserialize, Debug)]
pub struct Warehouse {
    /// The database name in Snowflake.
    pub name: String,
}
