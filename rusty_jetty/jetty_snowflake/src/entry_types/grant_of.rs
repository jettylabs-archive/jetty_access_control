use serde::Deserialize;

/// Snowflake entry for a grant to a role.
#[derive(Default, Deserialize, Debug)]
pub struct GrantOf {
    /// The role name in Snowflake.
    pub role: String,
    pub granted_to: String,
    pub grantee_name: String,
}
