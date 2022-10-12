use serde::Deserialize;

use super::RoleName;

/// Snowflake entry for a grant to a role.
#[derive(Default, Deserialize, Debug)]
pub struct GrantOf {
    /// The role name in Snowflake.
    pub role: RoleName,
    pub granted_to: String,
    pub grantee_name: String,
}
