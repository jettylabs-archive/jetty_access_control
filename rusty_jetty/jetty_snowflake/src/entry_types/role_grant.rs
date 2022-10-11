use serde::Deserialize;

/// Snowflake entry for a grant to a role.
#[derive(Default, Deserialize, Debug)]
pub struct RoleGrant {
    /// The role name in Snowflake.
    pub role: String,
}
