use serde::Serialize;

/// Marker trait for Snowflake Entries
#[derive(Clone, Serialize)]
#[serde(untagged)]
pub enum Entry {
    Role(crate::Role),
    User(crate::User),
    Asset(crate::Asset),
    Grant(crate::GrantType),
}
