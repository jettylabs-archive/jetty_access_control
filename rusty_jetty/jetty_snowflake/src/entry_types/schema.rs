use serde::{Deserialize, Serialize};

/// Snowflake Schema entry.
#[derive(Clone, Default, Deserialize, Serialize, Debug)]
pub struct Schema {
    /// The schema name in Snowflake.
    pub name: String,
    pub database_name: String,
}

impl Schema {
    pub fn new(name: String, database_name: String) -> Self {
        Self {
            name,
            database_name,
        }
    }

    pub(crate) fn fqn(&self) -> String {
        format!("{}.{}", self.database_name, self.name)
    }
}
