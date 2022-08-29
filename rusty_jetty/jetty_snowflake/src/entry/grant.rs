use serde::Deserialize;
use structmap::FromMap;
use structmap_derive::FromMap;

/// Snowflake Grant entry.
#[derive(FromMap, Default, Deserialize, Debug)]
pub struct Grant {
    pub name: String,
    pub privilege: String,
    pub granted_on: String,
}
