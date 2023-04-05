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

impl ToString for ObjectKind {
    fn to_string(&self) -> String {
        match self {
            ObjectKind::Table => "TABLE".to_string(),
            ObjectKind::View => "VIEW".to_string(),
        }
    }
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
}
