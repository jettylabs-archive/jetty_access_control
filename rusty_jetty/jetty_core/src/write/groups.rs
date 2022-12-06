//! Parse and manage user-configured groups

mod parser;

use std::collections::HashMap;

use petgraph::stable_graph::NodeIndex;
use serde::Deserialize;

use crate::{
    access_graph::{AccessGraph, EdgeType, JettyNode, NodeName, PolicyAttributes},
    jetty::ConnectorNamespace,
    Jetty,
};

use super::policies;

/// group configuration, as represented in the yaml
#[derive(Deserialize, Debug)]
pub(crate) struct GroupConfig {
    name: String,
    // This is nice to have, but ignore it for now
    connector_names: Option<Vec<ConnectorName>>,
    members: GroupMembers,
    pos: u64,
}

#[derive(Deserialize, Debug)]
pub(crate) struct ConnectorName {
    connector: ConnectorNamespace,
    alias: String,
    pos: u64,
}

#[derive(Deserialize, Debug)]
pub(crate) struct GroupMembers {
    groups: Option<Vec<MemberGroup>>,
    users: Option<Vec<MemberUser>>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct MemberGroup {
    name: String,
    pos: u64,
}

#[derive(Deserialize, Debug)]
pub(crate) struct MemberUser {
    name: String,
    pos: u64,
}

struct GroupConfigError {
    message: String,
    pos: u64,
}

struct Diff {
    group_name: NodeName,
    // This is a vec because you might both add and remove users
    details: Vec<DiffDetails>,
}

enum DiffDetails {
    AddGroup {
        members: GroupMemberChanges,
    },
    RemoveGroup,
    ModifyGroup {
        add: GroupMemberChanges,
        remove: GroupMemberChanges,
    },
}

struct GroupMemberChanges {
    users: Vec<NodeName>,
    groups: Vec<NodeName>,
}

/// Validate group config by making sure that users, groups, and listed connectors exist. Returns a vec of errors. If the vec is empty, there were no errors.
/// This allows all errors to be displayed at once.
fn validate_group_config(
    groups: &HashMap<String, GroupConfig>,
    jetty: Jetty,
    ag: AccessGraph,
) -> Vec<GroupConfigError> {
    let mut errors: Vec<GroupConfigError> = Vec::new();

    for (name, config) in groups {
        // check to see if there's a connector prefix and if it's allowed
        if let Some(prefix) = name.split("::").next() {
            if !jetty
                .config
                .connectors
                .contains_key(&ConnectorNamespace(prefix.to_owned()))
            {
                errors.push(GroupConfigError { message:format!("configuration specifies a group `{name}` with the prefix `{prefix}` but there is no connector `{prefix}` in the project"), pos: config.pos })
            }
        }

        // check the groups
        if let Some(member_groups) = &config.members.groups {
            for g in member_groups {
                if !groups.contains_key(&g.name) {
                    errors.push(GroupConfigError { message:format!("configuration refers to group `{}`, but there is no group with that name in the configuration", &g.name), pos: g.pos })
                }
            }
        }

        // Check that the connectors exist
        if let Some(connector_names) = &config.connector_names {
            for n in connector_names {
                if !jetty.config.connectors.contains_key(&n.connector) {
                    errors.push(GroupConfigError { message:format!("configuration refers to a connector called `{}`, but there is no connector with that name in the project", &n.connector), pos: n.pos })
                }
            }
        }

        // check the users
        if let Some(member_users) = &config.members.users {
            for u in member_users {
                if let Err(_) = ag.get_node(&NodeName::User(u.name.to_owned())) {
                    errors.push(GroupConfigError { message:format!("configuration refers to user `{}`, but there is no user with that name", &u.name), pos: u.pos })
                }
            }
        }
    }

    errors
}

// Diff with existing graph
fn generate_diff(
    groups: &HashMap<String, GroupConfig>,
    ag: AccessGraph,
) -> HashMap<NodeName, Diff> {
    let mut group_diffs = HashMap::new();
    let mut policy_diffs = Vec::new();

    let mut ag_groups = ag.graph.nodes.groups.clone();

    for (_, group) in groups {
        // build the NodeName
        let node_name = NodeName::Group {
            name: group.name.to_owned(),
            origin: ConnectorNamespace(if let Some(prefix) = group.name.split("::").next() {
                prefix.to_string()
            } else {
                "Jetty".to_string()
            }),
        };

        // check if the group exists, removing the key if it does
        if let Some(group_index) = ag_groups.remove(&node_name) {
            // get all the users in the existing group and diff them
            let ag_member_users = ag.get_matching_children(
                group_index,
                |e| matches!(e, EdgeType::Includes),
                |_| false,
                |n| matches!(n, JettyNode::User(_)),
                None,
                None,
            );

            let old = ag_member_users
                .iter()
                .map(|id| ag[*id].get_node_name())
                .collect::<Vec<_>>();

            let new = users_to_node_names(&group.members.users);

            let user_changes = diff_node_names(&old, &new);

            // get all the groups in the existing group and diff them
            let ag_member_groups = ag.get_matching_children(
                group_index,
                |e| matches!(e, EdgeType::Includes),
                |_| false,
                |n| matches!(n, JettyNode::Group(_)),
                None,
                None,
            );

            let old = ag_member_groups
                .iter()
                .map(|id| ag[*id].get_node_name())
                .collect::<Vec<_>>();

            let new = groups_to_node_names(&group.members.groups);

            let group_changes = diff_node_names(&old, &new);

            if !user_changes.add.is_empty()
                || !group_changes.add.is_empty()
                || !user_changes.remove.is_empty()
                || !group_changes.remove.is_empty()
            {
                group_diffs.insert(
                    node_name.clone(),
                    Diff {
                        group_name: node_name.clone(),
                        details: vec![DiffDetails::ModifyGroup {
                            add: GroupMemberChanges {
                                users: user_changes.add,
                                groups: group_changes.add,
                            },
                            remove: GroupMemberChanges {
                                users: user_changes.remove,
                                groups: group_changes.remove,
                            },
                        }],
                    },
                );
            };
        } else {
            // if it doesn't exist, add a new group diff, with all the appropriate users
            group_diffs.insert(
                node_name.clone(),
                Diff {
                    group_name: node_name.clone(),
                    details: vec![DiffDetails::AddGroup {
                        members: GroupMemberChanges {
                            users: users_to_node_names(&group.members.users),
                            groups: groups_to_node_names(&group.members.groups),
                        },
                    }],
                },
            );
        }
    }

    // now iterate through all of the groups and drop any that don't exist
    for (k, v) in ag_groups {
        group_diffs.insert(
            k.clone(),
            Diff {
                group_name: k.clone(),
                details: vec![DiffDetails::RemoveGroup],
            },
        );

        // Get all related policies and remove them as well
        let remove_policies = ag.get_matching_children(
            v,
            |e| matches!(e, EdgeType::GrantedFrom),
            |_| false,
            |n| matches!(n, JettyNode::Policy(_)),
            None,
            Some(1),
        );

        for policy_index in remove_policies {
            let policy = match TryInto::<PolicyAttributes>::try_into(ag[policy_index].clone()) {
                Ok(p) => p,
                Err(_) => continue,
            };

            let policy_targets = ag.get_matching_children(
                policy_index,
                |e| matches!(e, EdgeType::Governs),
                |_| false,
                |n| matches!(n, JettyNode::Asset(_)),
                None,
                Some(1),
            );

            for target in policy_targets {
                policy_diffs.push(policies::Diff {
                    asset: ag[target].get_node_name(),
                    agent: k.clone(),
                    details: vec![policies::DiffDetails::RemovePolicy],
                });
            }
        }
    }

    group_diffs
}

fn users_to_node_names(users: &Option<Vec<MemberUser>>) -> Vec<NodeName> {
    match users {
        Some(users) => users
            .iter()
            .map(|u| NodeName::User(u.name.to_owned()))
            .collect::<Vec<_>>(),
        None => Vec::new(),
    }
}

fn groups_to_node_names(groups: &Option<Vec<MemberGroup>>) -> Vec<NodeName> {
    match groups {
        Some(groups) => groups
            .iter()
            .map(|g| NodeName::Group {
                name: g.name.to_owned(),
                origin: ConnectorNamespace(if let Some(prefix) = g.name.split("::").next() {
                    prefix.to_string()
                } else {
                    "Jetty".to_string()
                }),
            })
            .collect::<Vec<_>>(),
        None => Vec::new(),
    }
}

struct NodeNameListDiff {
    add: Vec<NodeName>,
    remove: Vec<NodeName>,
}

fn diff_node_names(old: &Vec<NodeName>, new: &Vec<NodeName>) -> NodeNameListDiff {
    // get everything that new contains and old doesn't
    let add = new
        .iter()
        .filter(|n| !old.contains(n))
        .map(|n| n.to_owned())
        .collect();

    // get everything that old contains and new doesn't
    let remove = old
        .iter()
        .filter(|n| !new.contains(n))
        .map(|n| n.to_owned())
        .collect();

    NodeNameListDiff { add, remove }
}
