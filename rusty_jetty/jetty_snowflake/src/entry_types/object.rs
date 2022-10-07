use serde::{Deserialize, Serialize};

/// Snowflake Table entry.
#[derive(Clone, Default, Deserialize, Serialize, Debug)]
pub struct Object {
    /// The Table name in Snowflake.
    pub name: String,
    pub schema_name: String,
    pub database_name: String,
    pub kind: String,
}

impl Object {
    pub(crate) fn fqn(&self) -> String {
        format!("{}.{}.{}", self.database_name, self.schema_name, self.name)
    }
}
