use serde::{Deserialize, Serialize};

/// Snowflake View entry.
#[derive(Clone, Default, Deserialize, Serialize, Debug)]
pub struct View {
    /// The view name in Snowflake.
    pub name: String,
    pub schema_name: String,
    pub database_name: String,
}
