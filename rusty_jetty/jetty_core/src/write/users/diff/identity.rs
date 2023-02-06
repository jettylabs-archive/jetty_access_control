//! Diff changes to user identities

use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fmt::Display,
};

use anyhow::{anyhow, Result};
use colored::Colorize;

use crate::{
    access_graph::{JettyNode, NodeName, UserAttributes},
    jetty::ConnectorNamespace,
    write::{
        groups::GroupConfig, users::parser::get_validated_file_config_map, utils::diff_hashset,
    },
    Jetty,
};

/// Differences between identity assignemnts in the config and the environment
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IdentityDiff {
    /// The user with the change
    pub(crate) user: NodeName,
    /// The details of the change
    pub(crate) details: IdentityDiffDetails,
}

impl Display for IdentityDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut text = "".to_owned();
        match &self.details {
            IdentityDiffDetails::Add { add } => {
                text +=
                    format!("{}{}\n", "+ user: ".green(), self.user.to_string().green()).as_str();
                for (conn, local_name) in add {
                    text += format!("{}", format!("  + {conn}: {local_name}\n").green()).as_str();
                }
            }
            IdentityDiffDetails::Remove { remove } => {
                text += format!("{}", format!("- user: {}\n", self.user).red()).as_str();
                for (conn, local_name) in remove {
                    text += &format!("{}", format!("  - {conn}: {local_name}\n").red());
                }
            }
            IdentityDiffDetails::Modify { add, remove } => {
                text += format!(
                    "{}{}\n",
                    "~ user: ".yellow(),
                    self.user.to_string().yellow()
                )
                .as_str();
                for (conn, local_name) in add {
                    text += format!("{}", format!("  + {conn}: {local_name}\n").green()).as_str();
                }
                for (conn, local_name) in remove {
                    text += format!("{}", format!("  - {conn}: {local_name}\n").red()).as_str();
                }
            }
        }
        write!(f, "{text}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum IdentityDiffDetails {
    Add {
        add: BTreeSet<(ConnectorNamespace, String)>,
    },
    Remove {
        remove: BTreeSet<(ConnectorNamespace, String)>,
    },
    Modify {
        add: BTreeSet<(ConnectorNamespace, String)>,
        remove: BTreeSet<(ConnectorNamespace, String)>,
    },
}

/// This diffs the actual identities themselves - what local usernames become what Jetty user?
// FUTURE: Right now, this is inefficient with how it reads in the config
// (we could end up reading it in many times for different processes)
pub fn get_identity_diffs(
    jetty: &Jetty,
    validated_group_config: &GroupConfig,
) -> Result<HashSet<IdentityDiff>> {
    let config_identity_state = get_identity_config_state(jetty, validated_group_config)?;
    let mut env_identity_state = get_identity_env_state(jetty)?;
    let mut res = HashSet::new();

    // handle nodes in the config, but not in the env
    for (config_node, config_identities) in &config_identity_state {
        // does this node exist in env? If so remove it. We'll deal with the leftovers later!
        let details = match env_identity_state.remove(config_node) {
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
            None => IdentityDiffDetails::Add {
                add: {
                    #[allow(clippy::unnecessary_to_owned)]
                    config_identities.to_owned().into_iter().collect()
                },
            },
        };
        res.insert(IdentityDiff {
            user: config_node.to_owned(),
            details,
        });
    }

    // handle nodes in the env, but not in the config
    for (env_node, env_identities) in env_identity_state {
        res.insert(IdentityDiff {
            user: env_node,
            details: IdentityDiffDetails::Remove {
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

    #[allow(clippy::unnecessary_to_owned)]
    let config_set: HashSet<_> = config.to_owned().into_iter().collect();

    #[allow(clippy::unnecessary_to_owned)]
    let env_set: HashSet<_> = env.to_owned().into_iter().collect();
    let (add, remove) = diff_hashset(&config_set, &env_set);

    IdentityDiffDetails::Modify {
        add: add.collect(),
        remove: remove.collect(),
    }
}

/// Get the identity state from the user configuration files, and return a Map of
/// <NodeName, (Connector, Local Name)>.
fn get_identity_config_state(
    jetty: &Jetty,
    validated_group_config: &GroupConfig,
) -> Result<HashMap<NodeName, HashMap<ConnectorNamespace, String>>> {
    let configs = get_validated_file_config_map(jetty, validated_group_config)?;
    let res: HashMap<_, HashMap<_, _>> = configs
        .into_values()
        .map(|user| {
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
                .or_insert_with(|| HashMap::from([(conn.to_owned(), local_name.to_owned())]));
            acc
        },
    );
    Ok(res)
}

/// Given user identity diffs, update the access graph to match the new user mapping. This
/// should be run before other configurations are read and diffs are generated because it
/// may affect them!
pub fn update_graph(jetty: &mut Jetty, diffs: &HashSet<IdentityDiff>) -> Result<()> {
    let mut to = HashMap::new();
    let mut from = HashMap::new();
    let mut remove_list = HashSet::new();
    let mut add_list = HashSet::new();

    // create new users as needed
    for diff in diffs {
        match &diff.details {
            IdentityDiffDetails::Add { add } => {
                to.extend(add.iter().map(|d| (d.to_owned(), diff.user.to_owned())));
                add_list.insert(diff.user.to_owned());
            }
            IdentityDiffDetails::Remove { remove } => {
                from.extend(remove.iter().map(|d| (d.to_owned(), diff.user.to_owned())));
                remove_list.insert(diff.user.to_owned());
            }
            IdentityDiffDetails::Modify { add, remove } => {
                from.extend(remove.iter().map(|d| (d.to_owned(), diff.user.to_owned())));
                to.extend(add.iter().map(|d| (d.to_owned(), diff.user.to_owned())));
            }
        }
    }

    // Because of how fetch works, every "to" will have a matching "from". Our validation requires that all local users
    // are accounted for, so all froms will have a to as well.

    // Order
    // Create new nodes
    for node_name in add_list {
        add_new_user_node(jetty, &node_name)?;
    }

    update_access_graph_edges(jetty, &to, &from)?;
    modify_translator_mapping(jetty, &to, &from)?;

    // remove the nodes we don't need anymore
    let ag = jetty.try_access_graph_mut()?;
    for node_name in remove_list {
        let idx = ag.get_untyped_index_from_name(&node_name).ok_or_else(|| {
            anyhow!("unable to remove user {node_name}: couldn't find node index")
        })?;
        ag.graph.remove_node(idx)?;
    }

    Ok(())
}

/// adds a new user node to the graph. Doesn't specify connectors - this should be done when local names are added
fn add_new_user_node(jetty: &mut Jetty, node_name: &NodeName) -> Result<()> {
    let ag = jetty.try_access_graph_mut()?;

    let node = match &node_name {
        NodeName::User(_) => JettyNode::User(UserAttributes::new(
            node_name,
            &Default::default(),
            &Default::default(),
            Default::default(),
        )),
        _ => panic!("invalid node name: expecting NodeName::User"),
    };

    ag.graph.add_node(&node)
}

fn update_access_graph_edges(
    jetty: &mut Jetty,
    to: &HashMap<(ConnectorNamespace, String), NodeName>,
    from: &HashMap<(ConnectorNamespace, String), NodeName>,
) -> Result<()> {
    let ag = jetty.try_access_graph_mut()?;
    let mut plans = Vec::new();
    let mut connector_additions = HashMap::new();
    let mut connector_removals = HashMap::new();

    for (key, new_node) in to {
        let conn = &key.0.to_owned();
        let new_node_idx = ag
            .get_untyped_index_from_name(new_node)
            .ok_or_else(|| anyhow!("unable to find source node"))?;

        if let Some(old_node) = from.get(key) {
            let old_node_idx = ag
                .get_untyped_index_from_name(old_node)
                .ok_or_else(|| anyhow!("unable to find original node"))?;
            // redirect the edges
            plans.extend(ag.graph.plan_conditional_redirect_edges_from_node(
                old_node_idx,
                new_node_idx,
                // It's ok to move any node that shares the connector because right now, users are actually the only nodes that are allowed to have multiple connectors
                // Groups, policies, and default policies will be moved
                |n| n.get_node_connectors().contains(conn),
            )?);
            connector_additions
                .entry(new_node_idx)
                .and_modify(|c: &mut HashSet<ConnectorNamespace>| {
                    c.insert(conn.to_owned());
                })
                .or_insert_with(|| HashSet::from([(conn.to_owned())]));
            connector_removals
                .entry(old_node_idx)
                .and_modify(|c: &mut HashSet<ConnectorNamespace>| {
                    c.insert(conn.to_owned());
                })
                .or_insert_with(|| HashSet::from([(conn.to_owned())]));
        } else {
            panic!("unable to find origin for user identifier")
        }
    }

    // update graph edges
    ag.graph.execute_edge_redirects(plans)?;

    // remove connectors from user attributes that have had a connector removed
    for (idx, connectors) in connector_removals {
        for conn in connectors {
            ag.remove_connector_from_user(idx, &conn)?;
        }
    }

    // add connectors to user attributes that have had a connector added
    for (idx, connectors) in connector_additions {
        for conn in connectors {
            ag.add_connector_to_user(idx, &conn)?;
        }
    }

    Ok(())
}

/// for all the changes, point the translator to the correct node names
fn modify_translator_mapping(
    jetty: &mut Jetty,
    to: &HashMap<(ConnectorNamespace, String), NodeName>,
    from: &HashMap<(ConnectorNamespace, String), NodeName>,
) -> Result<()> {
    let ag = jetty.try_access_graph_mut()?;

    for (key, new_node) in to {
        let conn = &key.0.to_owned();
        let local_name = &key.1.to_owned();

        if let Some(old_node) = from.get(key) {
            ag.translator_mut()
                .modify_user_mapping(conn, local_name, old_node, new_node)?;
        } else {
            panic!("unable to find origin for user identifier")
        }
    }

    Ok(())
}
