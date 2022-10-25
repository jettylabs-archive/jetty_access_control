pub use super::future_grant::FutureGrant;

use std::collections::HashSet;

use jetty_core::connectors::nodes;
use serde::{Deserialize, Serialize};

use crate::cual::cual_from_snowflake_obj_name;

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
pub enum GrantType {
    Standard(StandardGrant),
    Future(super::future_grant::FutureGrant),
}

pub trait Grant {
    fn granted_on_name(&self) -> &str;
    fn role_name(&self) -> &str;
    fn privilege(&self) -> &str;
    fn granted_on(&self) -> &str;
    fn into_policy(self, all_privileges: HashSet<String>) -> nodes::RawPolicy;

    /// The globally-unique namespaced Jetty name.
    fn jetty_name(&self) -> String {
        format!("snowflake.{}.{}", self.role_name(), self.granted_on_name())
    }
}

impl Grant for GrantType {
    fn granted_on_name(&self) -> &str {
        match self {
            GrantType::Standard(s) => s.granted_on_name(),
            GrantType::Future(f) => f.granted_on_name(),
        }
    }

    fn role_name(&self) -> &str {
        match self {
            GrantType::Standard(s) => s.role_name(),
            GrantType::Future(f) => f.role_name(),
        }
    }

    fn privilege(&self) -> &str {
        match self {
            GrantType::Standard(s) => s.privilege(),
            GrantType::Future(f) => f.privilege(),
        }
    }

    fn granted_on(&self) -> &str {
        match self {
            GrantType::Standard(s) => s.granted_on(),
            GrantType::Future(f) => f.granted_on(),
        }
    }

    fn into_policy(self, all_privileges: HashSet<String>) -> nodes::RawPolicy {
        match self {
            GrantType::Standard(s) => s.into_policy(all_privileges),
            GrantType::Future(f) => f.into_policy(all_privileges),
        }
    }
}

/// Snowflake Grant entry.
#[derive(Default, Deserialize, Serialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct StandardGrant {
    // The role name or fully-qualified asset name this grant grants access to.
    pub name: String,
    pub privilege: String,
    pub granted_on: String,
    grantee_name: String,
}

impl Grant for StandardGrant {
    /// self.name corresponds to the object name when this is a grant on an object.
    fn granted_on_name(&self) -> &str {
        &self.name
    }

    /// self.grantee_name corresponds to the role name when this is a grant on a role.
    fn role_name(&self) -> &str {
        &self.grantee_name
    }

    fn privilege(&self) -> &str {
        &self.privilege
    }

    fn granted_on(&self) -> &str {
        &self.granted_on
    }

    fn into_policy(self, all_privileges: HashSet<String>) -> nodes::RawPolicy {
        let cual = cual_from_snowflake_obj_name(self.granted_on_name()).unwrap();

        nodes::RawPolicy::new(
            format!("snowflake.{}.{}", self.role_name(), self.granted_on_name()),
            all_privileges,
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

#[cfg(test)]
mod tests {

    use anyhow::Result;

    use super::*;

    #[test]
    fn jetty_name_works() {
        let g = StandardGrant {
            name: "db".to_owned(),
            privilege: "priv".to_owned(),
            granted_on: "TABLE".to_owned(),
            grantee_name: "my_role".to_owned(),
        };
        assert_eq!(g.jetty_name(), "snowflake.my_role.db".to_owned());
    }

    #[test]
    fn grant_into_policy_works() -> Result<()> {
        let g = StandardGrant {
            name: "db".to_owned(),
            privilege: "priv".to_owned(),
            granted_on: "TABLE".to_owned(),
            grantee_name: "grantee_name".to_owned(),
        };
        let p: nodes::RawPolicy = g.into_policy(HashSet::from(["priv".to_owned()]));
        assert_eq!(
            p,
            nodes::RawPolicy::new(
                "snowflake.grantee_name.db".to_owned(),
                HashSet::from(["priv".to_owned()]),
                HashSet::from([cual_from_snowflake_obj_name("DB")?.uri()]),
                HashSet::new(),
                HashSet::from(["grantee_name".to_owned()]),
                HashSet::new(),
                false,
                false,
            ),
        );
        Ok(())
    }

    #[test]
    fn future_grant_to_policy_results_in_idempotent_name() {
        let g = StandardGrant {
            name: "db".to_owned(),
            privilege: "priv".to_owned(),
            granted_on: "grant_on".to_owned(),
            grantee_name: "grantee_name".to_owned(),
        };
        let p: nodes::RawPolicy = g.clone().into_policy(HashSet::from(["priv".to_owned()]));
        let p2: nodes::RawPolicy = g.clone().into_policy(HashSet::from(["priv".to_owned()]));
        let p3: nodes::RawPolicy = g.into_policy(HashSet::from(["priv".to_owned()]));
        assert_eq!(p.name, "snowflake.grantee_name.db");
        assert_eq!(p2.name, p.name);
        assert_eq!(p3.name, p2.name);
    }

    #[test]
    fn future_grant_to_policy_with_extra_privileges_works() {
        let g = StandardGrant {
            name: "db".to_owned(),
            privilege: "priv".to_owned(),
            granted_on: "grant_on".to_owned(),
            grantee_name: "grantee_name".to_owned(),
        };
        let p: nodes::RawPolicy =
            g.into_policy(HashSet::from(["priv".to_owned(), "priv2".to_owned()]));
        assert_eq!(p.name, "snowflake.grantee_name.db");
        assert_eq!(
            p.privileges,
            HashSet::from(["priv".to_owned(), "priv2".to_owned()])
        );
    }
}
