use serde::Deserialize;
use structmap::FromMap;
use structmap_derive::FromMap;

/// Snowflake User entry.
#[derive(FromMap, Clone, Deserialize, Debug, Default)]
pub struct User {
    pub name: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub login_name: String,
    pub display_name: String,
}
