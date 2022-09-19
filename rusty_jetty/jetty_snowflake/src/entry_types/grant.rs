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
    fn into_policy(&self, all_privileges: HashSet<String>) -> nodes::Policy;

    /// The globally-unique namespaced Jetty name.
    fn jetty_name(&self) -> String {
        format!("snowflake.{}.{}", self.privilege(), self.role_name())
    }
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
            GrantType::Future(f) => f.role_name(),
        }
    }

    fn granted_on<'a>(&'a self) -> &'a str {
        match self {
            GrantType::Standard(s) => s.granted_on(),
            GrantType::Future(f) => f.granted_on(),
        }
    }

    fn into_policy(&self, all_privileges: HashSet<String>) -> nodes::Policy {
        match self {
            GrantType::Standard(s) => s.into_policy(all_privileges),
            GrantType::Future(f) => f.into_policy(all_privileges),
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

    fn into_policy(&self, all_privileges: HashSet<String>) -> nodes::Policy {
        let cual = cual_from_snowflake_obj_name(self.granted_on_name()).unwrap();

        let mut joined_privileges: Vec<_> = all_privileges.iter().cloned().collect();
        joined_privileges.sort();
        nodes::Policy::new(
            format!(
                "snowflake.{}.{}",
                joined_privileges.join("."),
                self.role_name()
            ),
            all_privileges,
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

mod tests {
    use crate::cual::{cual, Cual};

    use super::*;

    #[test]
    fn jetty_name_works() {
        let g = StandardGrant {
            name: "db".to_owned(),
            privilege: "priv".to_owned(),
            granted_on: "TABLE".to_owned(),
            grantee_name: "my_table".to_owned(),
        };
        assert_eq!(g.jetty_name(), "snowflake.priv.my_table".to_owned());
    }

    #[test]
    fn grant_into_policy_works() {
        let g = StandardGrant {
            name: "db".to_owned(),
            privilege: "priv".to_owned(),
            granted_on: "TABLE".to_owned(),
            grantee_name: "grantee_name".to_owned(),
        };
        let p: nodes::Policy = g.into_policy(HashSet::from(["priv".to_owned()]));
        assert_eq!(
            p,
            nodes::Policy::new(
                "snowflake.priv.grantee_name".to_owned(),
                HashSet::from(["priv".to_owned()]),
                HashSet::from([cual!("db").uri()]),
                HashSet::new(),
                HashSet::from(["grantee_name".to_owned()]),
                HashSet::new(),
                true,
                false,
            ),
        );
    }

    #[test]
    fn future_grant_to_policy_results_in_idempotent_name() {
        let g = StandardGrant {
            name: "db.<SCHEMA>".to_owned(),
            privilege: "priv".to_owned(),
            granted_on: "grant_on".to_owned(),
            grantee_name: "grantee_name".to_owned(),
        };
        let p: nodes::Policy = g.into_policy(HashSet::from(["priv".to_owned()]));
        let p2: nodes::Policy = g.into_policy(HashSet::from(["priv".to_owned()]));
        let p3: nodes::Policy = g.into_policy(HashSet::from(["priv".to_owned()]));
        assert_eq!(p.name, "snowflake.priv.grantee_name");
        assert_eq!(p2.name, p.name);
        assert_eq!(p3.name, p2.name);
    }

    #[test]
    fn future_grant_to_policy_with_extra_privileges_works() {
        let g = StandardGrant {
            name: "db.<SCHEMA>".to_owned(),
            privilege: "priv".to_owned(),
            granted_on: "grant_on".to_owned(),
            grantee_name: "grantee_name".to_owned(),
        };
        let p: nodes::Policy =
            g.into_policy(HashSet::from(["priv".to_owned(), "priv2".to_owned()]));
        assert_eq!(p.name, "snowflake.priv.priv2.grantee_name");
        assert_eq!(
            p.privileges,
            HashSet::from(["priv".to_owned(), "priv2".to_owned()])
        );
    }
}
