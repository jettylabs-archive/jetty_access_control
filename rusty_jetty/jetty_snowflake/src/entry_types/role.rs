use serde::{Deserialize, Serialize};

/// Wrapper struct for role names.
///
/// These are globally unique within a Snowflake account.
#[derive(Clone, Default, Serialize, Deserialize, Debug)]
#[repr(transparent)]
pub(crate) struct RoleName(pub(crate) String);

/// Snowflake Role entry.
#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub(crate) struct Role {
    /// The role name in Snowflake.
    pub(crate) name: RoleName,
}
