use serde::{Deserialize, Serialize};

use crate::strip_quotes_and_deserialize;

/// Wrapper struct for role names.
///
/// These are globally unique within a Snowflake account.
#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct RoleName(#[serde(deserialize_with = "strip_quotes_and_deserialize")] pub String);

/// Snowflake Role entry.
#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct Role {
    /// The role name in Snowflake.
    pub name: RoleName,
}
