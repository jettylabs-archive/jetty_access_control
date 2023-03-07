use std::collections::HashSet;

use anyhow::{bail, Result};
use jetty_core::connectors::{
    nodes::{self, RawPolicyGrantee},
    AssetType,
};

use serde::{Deserialize, Serialize};

use crate::{consts, cual::cual_from_snowflake_obj_name};

/// Snowflake future grant entry.
///
/// Future grants differ from regular grant objects in that they apply to
/// the parents of currently unnamed assets – ones that will be created later.
#[derive(Default, Deserialize, Serialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct FutureGrant {
    /// The future grant name in Snowflake.
    /// This typically looks something like
    /// DB.<SCHEMA> for future schema grants and
    /// DB.SCHEMA.<TABLE>, DB.SCHEMA.<VIEW>, DB.<TABLE>, or DB.<VIEW>
    /// for future table/view grants.
    name: String,
    privilege: String,
    /// This would be SCHEMA, TABLE, VIEW, etc.
    grant_on: String,
    // The role the future grant will apply to
    grantee_name: String,
}

impl FutureGrant {
    pub(crate) fn into_default_policy(
        self,
        all_privileges: HashSet<String>,
    ) -> nodes::RawDefaultPolicy {
        let stripped_name = self.root_asset();
        let cual = cual_from_snowflake_obj_name(stripped_name, self.grant_on()).unwrap();

        let wildcard_path = if self.grant_on == "SCHEMA" {
            "/*"
        } else {
            // If it's Tables/views/other things, but set at the database level
            // TODO This will break if there are any periods in the name of a database or schema
            if stripped_name.split('.').count() == 1 {
                "/*/*"
            }
            // Otherwise, it's tables, views, etc. Set at the schema level
            else {
                "/*"
            }
        }
        .to_owned();

        nodes::RawDefaultPolicy {
            privileges: all_privileges,
            root_asset: cual,
            wildcard_path,
            target_type: convert_to_asset_type(&self.grant_on).unwrap(),
            // Snowflake only allows grants to roles
            grantee: RawPolicyGrantee::Group(self.grantee_name),
            // empty for now
            metadata: Default::default(),
        }
    }

    /// Get the asset type that the grant is applied to
    pub(crate) fn grant_on(&self) -> &str {
        &self.grant_on
    }

    pub(crate) fn root_asset(&self) -> &str {
        self.name.split_once('<').unwrap().0.trim_end_matches('.')
    }

    /// Get the asset the grant is set on
    pub(crate) fn granted_on_name(&self) -> &str {
        &self.name
    }

    /// grantee_name is the role that this privilege will be granted to
    /// when new objects within scope are created
    pub(crate) fn role_name(&self) -> &str {
        &self.grantee_name
    }

    /// privilege
    pub(crate) fn privilege(&self) -> &str {
        &self.privilege
    }
}

fn convert_to_asset_type(grant_on: &str) -> Result<AssetType> {
    Ok(match grant_on {
        "SCHEMA" => AssetType(consts::SCHEMA.to_owned()),
        "TABLE" => AssetType(consts::TABLE.to_owned()),
        "VIEW" => AssetType(consts::VIEW.to_owned()),
        "DATABASE" => AssetType(consts::DATABASE.to_owned()),
        o => bail!("unable to handle asset type: {o}"),
    })
}
