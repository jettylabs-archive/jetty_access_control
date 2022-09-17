use std::collections::HashSet;

use jetty_core::connectors::nodes;

use serde::{Deserialize, Serialize};
use structmap::FromMap;
use structmap_derive::FromMap;

use crate::cual::cual_from_snowflake_obj_name;

use super::grant::Grant;

/// Snowflake future grant entry.
///
/// Future grants differ from regular grant objects in that they apply to
/// the parents of currently unnamed assets – ones that will be created later.
#[derive(FromMap, Default, Deserialize, Serialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct FutureGrant {
    /// The future grant name in Snowflake.
    /// This typically looks something like
    /// DB.<SCHEMA> for future schema grants and
    /// DB.SCHEMA.<TABLE> or DB.SCHEMA.<VIEW>
    /// for future table/view grants.
    name: String,
    privilege: String,
    grant_on: String,
    // The role the future grant will apply to
    grantee_name: String,
}

impl Grant for FutureGrant {
    /// The formatted future object name.
    fn granted_on_name<'a>(&'a self) -> &'a str {
        &self.name
    }

    /// grantee_name is the role that this privilege will be granted to
    /// when new objects within scope are created
    fn role_name<'a>(&'a self) -> &'a str {
        &self.grantee_name
    }

    fn privilege<'a>(&'a self) -> &'a str {
        &self.privilege
    }

    fn granted_on<'a>(&'a self) -> &'a str {
        &self.grant_on
    }
}
impl Into<nodes::Policy> for FutureGrant {
    fn into(self) -> nodes::Policy {
        // Modify the name to remove the angle-bracket portion.
        // i.e. DB.SCHEMA.<TABLE> becomes DB.SCHEMA
        // TODO: figure out if angle brackets are valid name characters. If so,
        // we need to do something more robust here.
        let stripped_name = self.name.split_once("<").unwrap().0;
        let cual = cual_from_snowflake_obj_name(stripped_name).unwrap();
        nodes::Policy::new(
            format!("snowflake.{}.{}", self.privilege, self.role_name(),),
            HashSet::from([self.privilege.to_owned()]),
            // Unwrap here is fine since we asserted that the set was not empty above.
            HashSet::from([cual.uri()]),
            HashSet::new(),
            HashSet::from([self.role_name().to_owned()]),
            // No direct user grants in Snowflake. Grants must pass through roles.
            HashSet::new(),
            // Defaults here for data read from Snowflake should be false.
            true,
            false,
        )
    }
}
