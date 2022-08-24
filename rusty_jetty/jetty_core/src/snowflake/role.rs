use serde::Deserialize;
use structmap::FromMap;
use structmap_derive::FromMap;

/// Snowflake Role entry.
#[derive(FromMap, Default, Deserialize, Debug)]
pub struct Role {
    /// The role name in Snowflake.
    pub name: String,
}
