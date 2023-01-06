//! Diff changes to group membership

use std::{
    collections::{BTreeSet, HashMap, HashSet},
    path::PathBuf,
};

use anyhow::Result;

use crate::{
    access_graph::{graph::typed_indices::TypedIndex, NodeName},
    jetty::ConnectorNamespace,
    write::{
        groups::{
            get_group_capable_connectors, get_group_to_nodename_map,
            parser::get_group_membership_map, GroupConfig,
        },
        users::UserYaml,
        utils::diff_hashset,
    },
    Jetty,
};

/// Differences between identity assignemnts in the config and the environment
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MembershipDiff {
    /// The user with the change
    pub(crate) user: NodeName,
    pub(crate) details: MembershipDiffDetails,
}

/// Details of the changes in group membership for a user
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MembershipDiffDetails {
    /// The groups that exist in the config, not in the environment
    pub(crate) add: BTreeSet<NodeName>,
    /// The groups that exist in the environment, not in the config
    pub(crate) remove: BTreeSet<NodeName>,
}

/// Get diffs to group membership
pub fn get_membership_diffs(
    jetty: &Jetty,
    validated_user_config: &HashMap<PathBuf, UserYaml>,
    validated_group_config: &GroupConfig,
) -> Result<HashSet<MembershipDiff>> {
    let env_state = get_membership_env_state(jetty)?;
    let config_state =
        get_membership_config_state(jetty, validated_user_config, validated_group_config)?;
    // we know that the env and config keys will match 1:1 - that's part of parser validation
    Ok(config_state
        .into_iter()
        .filter_map(|(user, member_of)| {
            let (add, remove) = diff_hashset(&member_of, &env_state[&user]);
            let diff = MembershipDiff {
                user,
                details: MembershipDiffDetails {
                    add: add.collect(),
                    remove: remove.collect(),
                },
            };
            if !diff.details.add.is_empty() || !diff.details.remove.is_empty() {
                Some(diff)
            } else {
                None
            }
        })
        .collect())
}

/// Get the group membership state from the config, and return a Map of
/// <NodeName::User, Set<NodeName::Group>>.
fn get_membership_config_state(
    jetty: &Jetty,
    validated_user_config: &HashMap<PathBuf, UserYaml>,
    validated_group_config: &GroupConfig,
) -> Result<HashMap<NodeName, HashSet<NodeName>>> {
    let connectors = get_group_capable_connectors(jetty);
    let group_node_name_map = get_group_to_nodename_map(
        validated_group_config,
        &connectors.keys().cloned().collect(),
    );
    let group_membership_map = get_group_membership_map(validated_group_config);

    Ok(validated_user_config
        .iter()
        .map(|(_, user)| {
            (
                NodeName::User(user.name.to_owned()),
                user.identifiers
                    .keys()
                    .flat_map(|conn| {
                        user.member_of
                            .iter()
                            .flat_map(|g| {
                                handle_nested_groups(
                                    g,
                                    conn,
                                    &group_node_name_map,
                                    &group_membership_map,
                                    jetty,
                                )
                            })
                            .collect::<HashSet<_>>()
                    })
                    .collect(),
            )
        })
        .collect())
}

/// Get the group membership state from the access graph, and return a Map of
/// <NodeName::User, Set<NodeName::Group>>.
fn get_membership_env_state(jetty: &Jetty) -> Result<HashMap<NodeName, HashSet<NodeName>>> {
    let ag = jetty.try_access_graph()?;
    let res = ag
        .graph
        .node_ids
        .users
        .values()
        .map(|user_idx| -> Result<_> {
            Ok((
                user_idx.name(jetty)?,
                user_idx
                    .member_of_groups(jetty)?
                    .into_iter()
                    .map(|g| -> Result<_> { g.name(jetty) })
                    .collect::<Result<HashSet<_>>>()?,
            ))
        })
        .collect::<Result<HashMap<NodeName, HashSet<NodeName>>>>();
    res
}

/// For every group, check whether nested groups are allowed for the platform. If not,
/// add the upstream groups
// FUTURE: this calls supports_nested_groups() a fair amount (via recursion). We could take some of those calls out at some point
fn handle_nested_groups(
    group_name: &String,
    connector: &ConnectorNamespace,
    node_name_map: &HashMap<String, HashMap<ConnectorNamespace, NodeName>>,
    membership_map: &HashMap<String, HashSet<String>>,
    jetty: &Jetty,
) -> HashSet<NodeName> {
    let mut res = if let Some(g) = node_name_map[group_name].get(connector) {
        [g.to_owned()].into()
    } else {
        return [].into();
    };
    // does the connector support nested groups? If not, collect the upstream groups
    if supports_nested_groups(connector, jetty) {
        res
    } else {
        for child in &membership_map[group_name] {
            res.extend(handle_nested_groups(
                child,
                connector,
                node_name_map,
                membership_map,
                jetty,
            ))
        }
        res
    }
}

fn supports_nested_groups(connector: &ConnectorNamespace, jetty: &Jetty) -> bool {
    jetty.connector_manifests()[connector]
        .capabilities
        .write
        .contains(&crate::connectors::WriteCapabilities::Groups { nested: true })
}
