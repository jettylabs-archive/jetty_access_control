pub(crate) use super::future_grant::FutureGrant;

use std::collections::HashSet;

use jetty_core::connectors::nodes;
use serde::{Deserialize, Serialize};
use structmap::FromMap;
use structmap_derive::FromMap;

use crate::cual::cual_from_snowflake_obj_name;

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
pub enum GrantType {
    Standard(StandardGrant),
    Future(super::future_grant::FutureGrant),
}

pub(crate) trait Grant {
    fn granted_on_name<'a>(&'a self) -> &'a str;
    fn role_name<'a>(&'a self) -> &'a str;
    fn privilege<'a>(&'a self) -> &'a str;
    fn granted_on<'a>(&'a self) -> &'a str;
}

impl Grant for GrantType {
    fn granted_on_name<'a>(&'a self) -> &'a str {
        match self {
            GrantType::Standard(s) => s.granted_on_name(),
            GrantType::Future(f) => f.granted_on_name(),
        }
    }

    fn role_name<'a>(&'a self) -> &'a str {
        match self {
            GrantType::Standard(s) => s.role_name(),
            GrantType::Future(f) => f.role_name(),
        }
    }

    fn privilege<'a>(&'a self) -> &'a str {
        match self {
            GrantType::Standard(s) => s.privilege(),
            GrantType::Future(f) => f.privilege(),
        }
    }

    fn granted_on<'a>(&'a self) -> &'a str {
        match self {
            GrantType::Standard(s) => s.granted_on(),
            GrantType::Future(f) => f.granted_on(),
        }
    }
}

/// Snowflake Grant entry.
#[derive(FromMap, Default, Deserialize, Serialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct StandardGrant {
    // The role name or fully-qualified asset name this grant grants access to.
    pub name: String,
    pub privilege: String,
    pub granted_on: String,
    grantee_name: String,
}

impl Grant for StandardGrant {
    /// self.name corresponds to the object name when this is a grant on an object.
    fn granted_on_name<'a>(&'a self) -> &'a str {
        &self.name
    }

    /// self.grantee_name corresponds to the role name when this is a grant on a role.
    fn role_name<'a>(&'a self) -> &'a str {
        &self.grantee_name
    }

    fn privilege<'a>(&'a self) -> &'a str {
        &self.privilege
    }

    fn granted_on<'a>(&'a self) -> &'a str {
        &self.granted_on
    }
}

impl Into<nodes::Policy> for StandardGrant {
    fn into(self) -> nodes::Policy {
        // Modify the name to remove the angle-bracket portion.
        // i.e. DB.SCHEMA.<TABLE> becomes DB.SCHEMA
        // TODO: figure out if angle brackets are valid name characters. If so,
        // we need to do something more robust here.
        let cual = cual_from_snowflake_obj_name(self.granted_on_name()).unwrap();
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
            false,
            false,
        )
    }
}
