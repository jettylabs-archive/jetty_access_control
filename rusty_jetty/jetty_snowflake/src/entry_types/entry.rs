use serde::Serialize;

use super::Asset;

/// Marker trait for Snowflake Entries
#[derive(Clone, Serialize)]
#[serde(untagged)]
pub(crate) enum Entry {
    Role(crate::Role),
    User(crate::User),
    Asset(Asset),
    Grant(crate::GrantType),
}
