use serde::Deserialize;
use structmap::FromMap;
use structmap_derive::FromMap;

/// Snowflake RoleGrant entry.
#[derive(FromMap, Default, Deserialize, Debug)]
pub struct RoleGrant {
    /// The role name in Snowflake.
    pub role: String,
}
