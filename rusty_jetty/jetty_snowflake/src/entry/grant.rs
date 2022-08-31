use serde::{Deserialize, Serialize};
use structmap::FromMap;
use structmap_derive::FromMap;

/// Snowflake Grant entry.
#[derive(FromMap, Default, Deserialize, Serialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct Grant {
    // The role name or fully-qualified asset name this grant grants access to.
    pub name: String,
    pub privilege: String,
    pub granted_on: String,
}
