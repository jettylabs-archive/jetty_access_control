//! Manage the reading and writing of group configurations

pub mod bootstrap;
pub mod diff;
pub mod parser;

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::jetty::ConnectorNamespace;

pub use bootstrap::get_env_config;
pub use bootstrap::write_env_config;
pub(crate) use diff::get_group_capable_connectors;
pub use diff::{generate_diffs, Diff};
pub use parser::{get_config_map, parse_and_validate_groups};

pub(crate) type GroupConfig = BTreeSet<GroupYaml>;

/// The Yaml-serializable configuration of a group
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct GroupYaml {
    /// The name of the group
    name: String,
    /// The connector-specific names of the group
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    identifiers: BTreeMap<ConnectorNamespace, String>,
    /// The groups that this group is a member of. To be valid here, they must appear in the name field
    /// of another group in the config
    #[serde(skip_serializing_if = "BTreeSet::is_empty", default)]
    member_of: BTreeSet<String>,
}
