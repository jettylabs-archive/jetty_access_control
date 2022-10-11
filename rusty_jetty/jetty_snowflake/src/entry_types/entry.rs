use serde::Serialize;

use super::Asset;

/// Marker trait for Snowflake Entries
#[derive(Clone, Serialize)]
#[serde(untagged)]
// We need this repr for transmuting the object back into an array for
// integration testing.
#[repr(C)]
pub enum Entry {
    Role(crate::Role),
    User(crate::User),
    Asset(Asset),
    Grant(crate::GrantType),
}
