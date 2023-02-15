//! Diffing of config vs env

use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fmt::Display,
};

use anyhow::Result;
use colored::Colorize;

use crate::{
    access_graph::NodeName,
    connectors::WriteCapabilities,
    jetty::ConnectorNamespace,
    write::{utils::diff_hashset, SplitByConnector},
    Jetty,
};

use super::{bootstrap, get_group_to_nodename_map, GroupConfig};

#[derive(Debug, Clone)]
/// A Diff for groups
pub struct Diff {
    /// the group being diffed - this is the connector-specific name, not the jetty name
    pub group_name: NodeName,
    /// The specifics of the diff
    pub details: DiffDetails,
    /// The connector the diff should be applied to
    pub connector: ConnectorNamespace,
}

impl SplitByConnector for Diff {
    fn split_by_connector(&self) -> HashMap<ConnectorNamespace, Box<Self>> {
        [(self.connector.to_owned(), Box::new(self.to_owned()))].into()
    }
}

#[derive(Debug, Clone)]
/// Outlines the diff type needed
pub enum DiffDetails {
    /// Add a group
    AddGroup {
        /// the groups this group is a member of
        member_of: BTreeSet<NodeName>,
    },
    /// Remove a group
    RemoveGroup,
    /// Update a group
    ModifyGroup {
        /// The groups this group is becoming a member of
        add_member_of: BTreeSet<NodeName>,
        /// The groups this group will no longer be a member of
        remove_member_of: BTreeSet<NodeName>,
    },
}

impl Display for Diff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut text = "".to_owned();
        match &self.details {
            DiffDetails::AddGroup { member_of } => {
                text += format!(
                    "{}",
                    format!("+ group: {}\n", self.group_name).green()
                )
                .as_str();
                if !member_of.is_empty() {
                    text += "  member of:\n"
                };
                for group in member_of {
                    text +=
                        format!("{}", format!("    + {group}\n").green()).as_str();
                }
            }
            DiffDetails::RemoveGroup => {
                text += format!(
                    "{}",
                    format!("- group: {}\n", self.group_name).red()
                )
                .as_str();
            }
            DiffDetails::ModifyGroup {
                add_member_of,
                remove_member_of,
            } => {
                text += format!(
                    "{}{}\n",
                    "~ group: ".yellow(),
                    self.group_name.to_string().yellow()
                )
                .as_str();
                if !add_member_of.is_empty() || !remove_member_of.is_empty() {
                    text += "  member of:\n"
                };
                for user in add_member_of {
                    text += format!("{}", format!("    + {user}\n").green()).as_str();
                }

                for user in remove_member_of {
                    text += format!("{}", format!("    - {user}\n").red()).as_str();
                }
            }
        }
        write!(f, "{text}")
    }
}

/// Generate the list of diffs between env and config
pub fn generate_diffs(validated_config: &GroupConfig, jetty: &Jetty) -> Result<Vec<Diff>> {
    let mut env_state = bootstrap::get_env_membership_nodes(jetty)?;
    let config_state = get_config_state(validated_config, jetty);

    let mut res = Vec::new();

    // handle nodes in the config, but not in the env
    for (group, config_member_of) in &config_state {
        let connector = if let NodeName::Group { origin, .. } = &group {
            origin
        } else {
            panic!("expects a NodeName::Group")
        };
        // does this node exist in env? If so remove it. We'll deal with the leftovers later!
        let details = match env_state.remove(group) {
            Some(env_member_of) => {
                if config_member_of == &env_member_of {
                    // No change
                    continue;
                }
                // Node exists, but there's been a change
                else {
                    diff_matching_groups(config_member_of, &env_member_of)
                }
            }
            None => DiffDetails::AddGroup {
                member_of: config_member_of.iter().cloned().collect(),
            },
        };
        res.push(Diff {
            group_name: group.to_owned(),
            details,
            connector: connector.to_owned(),
        });
    }

    // handle nodes in the env, but not in the config
    for (group, _) in env_state {
        let connector = if let NodeName::Group { origin, .. } = &group {
            origin
        } else {
            panic!("expects a NodeName::Group")
        };

        res.push(Diff {
            group_name: group.to_owned(),
            details: DiffDetails::RemoveGroup,
            connector: connector.to_owned(),
        });
    }

    res.sort_by(|a, b| {
        a.group_name
            .to_string()
            .to_lowercase()
            .cmp(&b.group_name.to_string().to_lowercase())
    });
    Ok(res)
}

/// Get the state of groups, according to the configuration files
fn get_config_state(
    validated_config: &GroupConfig,
    jetty: &Jetty,
) -> HashMap<NodeName, HashSet<NodeName>> {
    let connectors = get_group_capable_connectors(jetty);
    // iterate through every group in the config. for each identifier, get the group's identifiers that are from the right connector
    let group_map =
        get_group_to_nodename_map(validated_config, &connectors.keys().cloned().collect());

    validated_config
        .iter()
        .flat_map(|group| {
            // for each connector-specific group
            group_map[&group.name]
                .iter()
                .map(|(conn, node_name)| {
                    (
                        node_name.to_owned(),
                        // branch on whether nested groups are allowed
                        match connectors[conn] {
                            true => group
                                .member_of
                                .iter()
                                .map(|jetty_name| group_map[jetty_name][conn].to_owned())
                                .collect(),
                            false => HashSet::new(),
                        },
                    )
                })
                .collect::<HashMap<_, _>>()
        })
        .collect()
}

/// Diff the member_of property of groups
fn diff_matching_groups(config: &HashSet<NodeName>, env: &HashSet<NodeName>) -> DiffDetails {
    let (add, remove) = diff_hashset(config, env);

    DiffDetails::ModifyGroup {
        add_member_of: add.collect(),
        remove_member_of: remove.collect(),
    }
}

/// Collect all connectors that can write groups, and specify whether they can write nested groups or not.
pub(crate) fn get_group_capable_connectors(jetty: &Jetty) -> HashMap<ConnectorNamespace, bool> {
    jetty
        .connector_manifests()
        .into_iter()
        .filter_map(|(n, m)| {
            if m.capabilities
                .write
                .contains(&WriteCapabilities::Groups { nested: true })
            {
                Some((n, true))
            } else if m
                .capabilities
                .write
                .contains(&WriteCapabilities::Groups { nested: false })
            {
                Some((n, false))
            } else {
                None
            }
        })
        .collect()
}
