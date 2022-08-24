use serde::Deserialize;
use structmap::FromMap;
use structmap_derive::FromMap;

/// Snowflake User entry.
#[derive(FromMap, Deserialize, Debug, Default)]
pub struct User {
    name: String,
}
