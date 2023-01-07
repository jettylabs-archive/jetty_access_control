//! Manage the reading and writing of group configurations

pub mod bootstrap;
pub mod diff;
pub mod parser;
mod update;

use anyhow::Result;

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::jetty::ConnectorNamespace;

pub use bootstrap::get_env_config;
pub use bootstrap::write_env_config;
pub(crate) use diff::get_group_capable_connectors;
pub use diff::{generate_diffs, Diff};
pub use parser::{get_group_to_nodename_map, parse_and_validate_groups};
pub(crate) use update::{remove_group_name, remove_user_name, update_group_name, update_user_name};

use super::UpdateConfig;

pub(crate) type GroupConfig = BTreeSet<GroupYaml>;

/// The Yaml-serializable configuration of a group
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct GroupYaml {
    /// The name of the group
    name: String,
    /// Description of the group
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    /// The connector-specific names of the group
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    identifiers: BTreeMap<ConnectorNamespace, String>,
    /// The groups that this group is a member of. To be valid here, they must appear in the name field
    /// of another group in the config
    #[serde(
        skip_serializing_if = "BTreeSet::is_empty",
        default,
        rename = "member of"
    )]
    member_of: BTreeSet<String>,
}

impl UpdateConfig for GroupYaml {
    fn update_user_name(&mut self, _old: &str, _new: &str) -> Result<bool> {
        Ok(false)
    }

    fn remove_user_name(&mut self, _name: &str) -> Result<bool> {
        Ok(false)
    }

    fn update_group_name(&mut self, old: &str, new: &str) -> Result<bool> {
        let mut changed = false;
        if self.name == old {
            self.name = new.to_string();
            changed = true;
        }
        if self.member_of.remove(old) {
            self.member_of.insert(new.to_string());
            changed = true;
        }

        Ok(changed)
    }

    /// This will remove references to groups, but not the group itself
    fn remove_group_name(&mut self, name: &str) -> Result<bool> {
        if self.member_of.remove(name) {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
