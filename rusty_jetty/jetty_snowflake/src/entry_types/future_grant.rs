use std::collections::HashSet;

use jetty_core::connectors::nodes;

use serde::{Deserialize, Serialize};

use crate::cual::cual_from_snowflake_obj_name;

use super::grant::Grant;

/// Snowflake future grant entry.
///
/// Future grants differ from regular grant objects in that they apply to
/// the parents of currently unnamed assets – ones that will be created later.
#[derive(Default, Deserialize, Serialize, Debug, PartialEq, Eq, Hash, Clone)]
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
    fn granted_on_name(&self) -> &str {
        &self.name
    }

    /// grantee_name is the role that this privilege will be granted to
    /// when new objects within scope are created
    fn role_name(&self) -> &str {
        &self.grantee_name
    }

    fn privilege(&self) -> &str {
        &self.privilege
    }

    fn granted_on(&self) -> &str {
        &self.grant_on
    }

    fn into_policy(self, all_privileges: HashSet<String>) -> nodes::RawPolicy {
        // Modify the name to remove the angle-bracket portion.
        // i.e. DB.SCHEMA.<TABLE> becomes DB.SCHEMA
        // TODO: figure out if angle brackets are valid name characters. If so,
        // we need to do something more robust here.
        let stripped_name = self.name.split_once('<').unwrap().0.trim_end_matches('.');
        let cual = cual_from_snowflake_obj_name(stripped_name).unwrap();
        let mut joined_privileges: Vec<_> = all_privileges.iter().cloned().collect();
        joined_privileges.sort();
        nodes::RawPolicy::new(
            // We add the `.future` to differentiate this policy from
            // non-heirarchical ones
            format!("snowflake.future.{}.{stripped_name}", self.role_name()),
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

    fn jetty_name(&self) -> String {
        format!(
            "snowflake.future.{}.{}",
            self.role_name(),
            self.granted_on_name()
        )
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;

    #[test]
    fn jetty_name_works() {
        let g = FutureGrant {
            name: "db".to_owned(),
            privilege: "priv".to_owned(),
            grant_on: "TABLE".to_owned(),
            grantee_name: "my_table".to_owned(),
        };
        assert_eq!(g.jetty_name(), "snowflake.future.my_table.db".to_owned());
    }

    #[test]
    fn grant_into_policy_works() -> Result<()> {
        let g = FutureGrant {
            name: "db.<SCHEMA>".to_owned(),
            privilege: "priv".to_owned(),
            grant_on: "grant_on".to_owned(),
            grantee_name: "grantee_name".to_owned(),
        };
        let p: nodes::RawPolicy = g.into_policy(HashSet::from(["priv".to_owned()]));
        assert_eq!(
            p,
            nodes::RawPolicy::new(
                "snowflake.future.grantee_name.db".to_owned(),
                HashSet::from(["priv".to_owned()]),
                HashSet::from([cual_from_snowflake_obj_name("DB")?.uri()]),
                HashSet::new(),
                HashSet::from(["grantee_name".to_owned()]),
                HashSet::new(),
                true,
                false,
            ),
        );
        Ok(())
    }

    #[test]
    fn future_grant_to_policy_results_in_idempotent_name() {
        let g = FutureGrant {
            name: "db.<SCHEMA>".to_owned(),
            privilege: "priv".to_owned(),
            grant_on: "grant_on".to_owned(),
            grantee_name: "grantee_name".to_owned(),
        };
        let p: nodes::RawPolicy = g.clone().into_policy(HashSet::from(["priv".to_owned()]));
        let p2: nodes::RawPolicy = g.clone().into_policy(HashSet::from(["priv".to_owned()]));
        let p3: nodes::RawPolicy = g.into_policy(HashSet::from(["priv".to_owned()]));
        assert_eq!(p.name, "snowflake.future.grantee_name.db");
        assert_eq!(p2.name, p.name);
        assert_eq!(p3.name, p2.name);
    }

    #[test]
    fn future_grant_to_policy_with_extra_privileges_works() {
        let g = FutureGrant {
            name: "db.<SCHEMA>".to_owned(),
            privilege: "priv".to_owned(),
            grant_on: "grant_on".to_owned(),
            grantee_name: "grantee_name".to_owned(),
        };
        let p: nodes::RawPolicy =
            g.into_policy(HashSet::from(["priv".to_owned(), "priv2".to_owned()]));
        assert_eq!(p.name, "snowflake.future.grantee_name.db");
        assert_eq!(
            p.privileges,
            HashSet::from(["priv".to_owned(), "priv2".to_owned()])
        );
    }

    #[test]
    fn future_grant_table_into_policy_works() -> Result<()> {
        let g = FutureGrant {
            name: "db.schema.<TABLE>".to_owned(),
            privilege: "priv".to_owned(),
            grant_on: "TABLE".to_owned(),
            grantee_name: "grantee_name".to_owned(),
        };
        let p: nodes::RawPolicy = g.into_policy(HashSet::from(["priv".to_owned()]));
        assert_eq!(
            p,
            nodes::RawPolicy::new(
                "snowflake.future.grantee_name.db.schema".to_owned(),
                HashSet::from(["priv".to_owned()]),
                HashSet::from([cual_from_snowflake_obj_name("DB.SCHEMA")?.uri()]),
                HashSet::new(),
                HashSet::from(["grantee_name".to_owned()]),
                HashSet::new(),
                true,
                false,
            ),
        );
        Ok(())
    }

    #[test]
    #[should_panic]
    fn grant_into_policy_with_unstrippable_name_panics() {
        let g = FutureGrant {
            // Doesn't have the <TABLE> piece... not a valid future grant
            name: "db.".to_owned(),
            privilege: "priv".to_owned(),
            grant_on: "grant_on".to_owned(),
            grantee_name: "grantee_name".to_owned(),
        };
        g.into_policy(HashSet::from(["priv".to_owned()]));
    }
}
