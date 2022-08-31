use serde::Deserialize;
use serde_tuple::Serialize_tuple;
use structmap::FromMap;
use structmap_derive::FromMap;

/// Snowflake Role entry.
#[derive(FromMap, Clone, Default, Deserialize, Serialize_tuple, Debug)]
pub struct Role {
    /// The role name in Snowflake.
    pub name: String,
}
