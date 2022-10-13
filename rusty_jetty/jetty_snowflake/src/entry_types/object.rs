use serde::{Deserialize, Serialize};

/// A type of object.
#[derive(Copy, Clone, Default, Deserialize, Serialize, Debug)]
pub enum ObjectKind {
    #[default]
    #[serde(rename = "TABLE")]
    Table,
    #[serde(rename = "VIEW")]
    View,
}
/// Snowflake Table entry.
#[derive(Clone, Default, Deserialize, Serialize, Debug)]
pub struct Object {
    /// The Table name in Snowflake.
    pub name: String,
    pub schema_name: String,
    pub database_name: String,
    pub kind: ObjectKind,
}

impl Object {
    pub(crate) fn fqn(&self) -> String {
        format!("{}.{}.{}", self.database_name, self.schema_name, self.name)
    }

    pub(crate) fn schema_fqn(&self) -> String {
        format!("{}.{}", self.database_name, self.schema_name)
    }
}
