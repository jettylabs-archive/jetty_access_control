use serde::{Deserialize, Serialize};

/// Snowflake Schema entry.
#[derive(Clone, Default, Deserialize, Serialize, Debug)]
pub struct Schema {
    /// The schema name in Snowflake.
    pub name: String,
    pub database_name: String,
}

impl Schema {
    pub(crate) fn fqn(&self) -> String {
        format!("{}.{}", self.name, self.database_name)
    }
}
