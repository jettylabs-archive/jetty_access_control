//! Diffing for user configurations <-> Env

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fmt::Display,
    path::PathBuf,
};

use anyhow::Result;
use colored::Colorize;

use crate::{access_graph::NodeName, jetty::ConnectorNamespace, Jetty};

use super::{parser::get_validated_file_config_map, UserYaml};

/// Differences between identity assignemnts in the config and the environment
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct IdentityDiff {
    /// The user with the change
    node: NodeName,
    /// The details of the change
    details: IdentityDiffDetails,
}

impl Display for IdentityDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut text = "".to_owned();
        match &self.details {
            IdentityDiffDetails::AddUser { add } => {
                text +=
                    format!("{}{}\n", "+ user: ".green(), self.node.to_string().green()).as_str();
                for (conn, local_name) in add {
                    text += format!("{}", format!("  + {conn}: {local_name}\n").green()).as_str();
                }
            }
            IdentityDiffDetails::RemoveUser { remove } => {
                text +=
                    format!("{}", format!("- user: {}\n", self.node.to_string()).red()).as_str();
                for (conn, local_name) in remove {
                    text += &format!("{}", format!("  - {conn}: {local_name}\n").red());
                }
            }
            IdentityDiffDetails::ModifyUser { add, remove } => {
                text += format!(
                    "{}{}\n",
                    "~ user: ".yellow(),
                    self.node.to_string().yellow()
                )
                .as_str();
                for (conn, local_name) in add {
                    text += format!("{}", format!("  + {conn}: {local_name}\n").green()).as_str();
                }
                for (conn, local_name) in remove {
                    text += format!("{}", format!("  - {conn}:{local_name}\n").red()).as_str();
                }
            }
        }
        write!(f, "{text}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum IdentityDiffDetails {
    AddUser {
        add: BTreeSet<(ConnectorNamespace, String)>,
    },
    RemoveUser {
        remove: BTreeSet<(ConnectorNamespace, String)>,
    },
    ModifyUser {
        add: BTreeSet<(ConnectorNamespace, String)>,
        remove: BTreeSet<(ConnectorNamespace, String)>,
    },
}

/// This diffs the actual identities themselves - what local usernames become what Jetty user?
// FUTURE: Right now, this is inefficient with how it reads in the config
// (we could end up reading it in many times for different processes)
pub fn get_identity_diffs(jetty: &Jetty) -> Result<BTreeSet<IdentityDiff>> {
    let config_identity_state = get_identity_config_state(jetty)?;
    let mut env_identity_state = get_identity_env_state(jetty)?;
    let mut res = BTreeSet::new();

    // handle nodes in the config, but not in the env
    for (config_node, config_identities) in &config_identity_state {
        // does this node exist in env? If so remove it. We'll deal with the leftovers later!
        let details = match env_identity_state.remove(&config_node) {
            Some(env_identities) => {
                if &env_identities == config_identities {
                    // No change
                    continue;
                }
                // Node exists, but there's been a change
                else {
                    get_connector_name_changes(config_identities, &env_identities)
                }
            }
            None => IdentityDiffDetails::AddUser {
                add: config_identities.to_owned().into_iter().collect(),
            },
        };
        res.insert(IdentityDiff {
            node: config_node.to_owned(),
            details,
        });
    }

    // handle nodes in the env, but not in the config
    for (env_node, env_identities) in env_identity_state {
        res.insert(IdentityDiff {
            node: env_node,
            details: IdentityDiffDetails::RemoveUser {
                remove: env_identities.into_iter().collect(),
            },
        });
    }

    Ok(res)
}

/// get a IdentityDiffDetails::ModifyUser created from comparing the identity sets of the config and the environment
fn get_connector_name_changes(
    config: &HashMap<ConnectorNamespace, String>,
    env: &HashMap<ConnectorNamespace, String>,
) -> IdentityDiffDetails {
    // turn them into sets so that they're easier to compare
    let config_set: HashSet<_> = config.to_owned().into_iter().collect();
    let env_set: HashSet<_> = env.to_owned().into_iter().collect();
    let add = config_set.difference(&env_set).cloned().collect();
    let remove = env_set.difference(&config_set).cloned().collect();

    IdentityDiffDetails::ModifyUser { add, remove }
}

/// Get the identity state from the user configuration files, and return a Map of
/// <NodeName, (Connector, Local Name)>.
fn get_identity_config_state(
    jetty: &Jetty,
) -> Result<HashMap<NodeName, HashMap<ConnectorNamespace, String>>> {
    let configs = get_validated_file_config_map(jetty)?;
    let res: HashMap<_, HashMap<_, _>> = configs
        .into_iter()
        .map(|(path, user)| {
            (
                NodeName::User(user.name.to_owned()),
                user.identifiers
                    .iter()
                    .map(|(conn, local_name)| (conn.to_owned(), local_name.to_owned()))
                    .collect::<HashMap<_, _>>(),
            )
        })
        .collect();
    Ok(res)
}

/// Get the identity state from the access graph, and return a Map of
/// <NodeName, (Connector, Local Name)>.
fn get_identity_env_state(
    jetty: &Jetty,
) -> Result<HashMap<NodeName, HashMap<ConnectorNamespace, String>>> {
    let ag = jetty.try_access_graph()?;
    let res = ag.translator().get_all_local_users().iter().fold(
        HashMap::new(),
        |mut acc, ((conn, local_name), node_name)| {
            acc.entry(node_name.to_owned())
                .and_modify(|entry: &mut HashMap<ConnectorNamespace, String>| {
                    entry.insert(conn.to_owned(), local_name.to_owned());
                })
                .or_insert(HashMap::from([(conn.to_owned(), local_name.to_owned())]));
            acc
        },
    );
    Ok(res)
}
