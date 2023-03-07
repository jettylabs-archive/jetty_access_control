pub use super::future_grant::FutureGrant;

use std::collections::HashSet;

use jetty_core::connectors::nodes;
use serde::{Deserialize, Serialize};

use crate::cual::cual_from_snowflake_obj_name_parts;

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
pub enum GrantType {
    Standard(StandardGrant),
}

pub trait Grant {
    /// String representation of the fqn for the object the grant is on.
    fn granted_on_name(&self) -> String;
    fn role_name(&self) -> &str;
    fn privilege(&self) -> &str;
    fn granted_on(&self) -> &str;
    fn into_policy(self, all_privileges: HashSet<String>) -> nodes::RawPolicy;
}

/// This can be totally reworked, but just leaving it is as
impl Grant for GrantType {
    fn granted_on_name(&self) -> String {
        match self {
            GrantType::Standard(s) => s.granted_on_name(),
        }
    }

    fn role_name(&self) -> &str {
        match self {
            GrantType::Standard(s) => s.role_name(),
        }
    }

    fn privilege(&self) -> &str {
        match self {
            GrantType::Standard(s) => s.privilege(),
        }
    }

    fn granted_on(&self) -> &str {
        match self {
            GrantType::Standard(s) => s.granted_on(),
        }
    }

    fn into_policy(self, all_privileges: HashSet<String>) -> nodes::RawPolicy {
        match self {
            GrantType::Standard(s) => s.into_policy(all_privileges),
        }
    }
}

/// Snowflake Grant entry.
#[derive(Default, Deserialize, Serialize, Debug, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub struct StandardGrant {
    // The role name or fully-qualified asset name this grant grants access to.
    name: String,
    table_catalog: String,
    table_schema: String,
    pub(crate) privilege: String,
    granted_on: String,
    grantee_name: String,
}

impl Grant for StandardGrant {
    /// self.name corresponds to the object name when this is a grant on an object.
    fn granted_on_name(&self) -> String {
        match self.granted_on.as_str() {
            "TABLE" | "VIEW" => {
                format!("{}.{}.{}", self.table_catalog, self.table_schema, self.name)
            }
            "DATABASE" => self.table_catalog.to_string(),
            "SCHEMA" => format!("{}.{}", self.table_catalog, self.name),
            _ => panic!("Unknown grant type: {}", self.granted_on),
        }
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
        let cual = cual_from_snowflake_obj_name_parts(
            &self.name,
            &self.table_catalog,
            &self.table_schema,
            &self.granted_on,
        )
        .unwrap();

        let all_privileges = fix_privilege_names(all_privileges);

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

/// For some reason, the query returns at least one weird, improperly formatted privilege name. Fix that
fn fix_privilege_names(mut privileges: HashSet<String>) -> HashSet<String> {
    if privileges.remove("REFERENCE USAGE") {
        privileges.insert("REFERENCE_USAGE".to_owned());
    };
    privileges
}

#[cfg(test)]
mod tests {

    use anyhow::Result;

    use crate::cual::set_cual_account_name;

    use super::*;

    #[test]
    fn grant_into_policy_works() -> Result<()> {
        set_cual_account_name("account");
        let g = StandardGrant {
            name: "db".to_owned(),
            privilege: "priv".to_owned(),
            granted_on: "DATABASE".to_owned(),
            grantee_name: "grantee_name".to_owned(),
            table_catalog: "db".to_owned(),
            table_schema: "".to_owned(),
        };
        let p: nodes::RawPolicy = g.into_policy(HashSet::from(["priv".to_owned()]));
        assert_eq!(
            p,
            nodes::RawPolicy::new(
                "snowflake.grantee_name.db".to_owned(),
                HashSet::from(["priv".to_owned()]),
                HashSet::from([
                    cual_from_snowflake_obj_name_parts("db", "db", "", "DATABASE")?.uri()
                ]),
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
        set_cual_account_name("account");
        let g = StandardGrant {
            name: "db".to_owned(),
            privilege: "priv".to_owned(),
            granted_on: "DATABASE".to_owned(),
            grantee_name: "grantee_name".to_owned(),
            table_catalog: "db".to_owned(),
            table_schema: "".to_owned(),
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
        set_cual_account_name("account");
        let g = StandardGrant {
            name: "db".to_owned(),
            privilege: "priv".to_owned(),
            granted_on: "DATABASE".to_owned(),
            grantee_name: "grantee_name".to_owned(),
            table_catalog: "db".to_owned(),
            table_schema: "".to_owned(),
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
