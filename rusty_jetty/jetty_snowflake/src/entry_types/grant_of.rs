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

impl GrantOf {
    pub fn new(role: RoleName, granted_to: String, grantee_name: String) -> Self {
        Self {
            role,
            granted_to,
            grantee_name,
        }
    }
}
