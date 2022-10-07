use serde::{Deserialize, Serialize};

/// Snowflake User entry.
#[derive(Clone, Deserialize, Serialize, Debug, Default)]
pub struct User {
    pub name: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub login_name: String,
    pub display_name: String,
}
