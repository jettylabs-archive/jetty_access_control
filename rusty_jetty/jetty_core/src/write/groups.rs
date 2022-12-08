//! Parse and manage user-configured groups

pub(crate) mod bootstrap;
mod parser;

use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, bail, Result};
use serde::Deserialize;

use crate::{
    access_graph::{AccessGraph, EdgeType, JettyNode, NodeName, PolicyAttributes},
    connectors::WriteCapabilities,
    jetty::ConnectorNamespace,
    Jetty,
};

use super::policies;

/// group configuration, as represented in the yaml
#[derive(Deserialize, Debug)]
pub(crate) struct GroupConfig {
    name: String,
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

#[derive(Debug)]
struct GroupConfigError {
    message: String,
    pos: u64,
}

#[derive(Debug)]
struct Diff {
    group_name: NodeName,
    details: DiffDetails,
    connectors: HashSet<ConnectorNamespace>,
}

#[derive(Debug)]
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

#[derive(Debug)]
struct GroupMemberChanges {
    users: Vec<NodeName>,
    groups: Vec<NodeName>,
}

struct NodeNameListDiff {
    add: Vec<NodeName>,
    remove: Vec<NodeName>,
}

/// Validate group config by making sure that users, groups, and listed connectors exist. Returns a vec of errors. If the vec is empty, there were no errors.
/// This allows all errors to be displayed at once.
fn validate_group_config(
    groups: &HashMap<String, GroupConfig>,
    jetty: &Jetty,
) -> Vec<GroupConfigError> {
    let mut errors: Vec<GroupConfigError> = Vec::new();
    let ag = jetty.access_graph.as_ref().unwrap();

    for (name, config) in groups {
        // check to see if there's a connector prefix and if it's allowed
        if let Some((prefix, suffix)) = name.split_once("::").map(|p| (p.0, p.1)) {
            if !jetty
                .connectors
                .contains_key(&ConnectorNamespace(prefix.to_owned()))
            {
                errors.push(GroupConfigError { message:format!("configuration specifies a group `{suffix}` with the prefix `{prefix}` but there is no connector `{prefix}` in the project"), pos: config.pos })
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
    jetty: &Jetty,
) -> Result<HashMap<NodeName, Diff>> {
    let mut group_diffs = HashMap::new();
    let mut policy_diffs = Vec::new();

    let ag = jetty.access_graph.as_ref().ok_or_else(|| {
        anyhow!("jetty initialized without an access graph; try running `jetty fetch` first")
    })?;

    let mut ag_groups = ag.graph.nodes.groups.clone();

    // Writing groups is limited to the connectors that have the notion of a groups, and we keep information about whether
    // nested groups are allowed
    let jetty_connector_names: HashMap<ConnectorNamespace, bool> = jetty
        .connector_manifests()
        .into_iter()
        .filter_map(|(n, m)| {
            if m.capabilities
                .write
                .contains(&WriteCapabilities::Groups { nested: true })
            {
                Some((n.to_owned(), true))
            } else if m
                .capabilities
                .write
                .contains(&WriteCapabilities::Groups { nested: false })
            {
                Some((n.to_owned(), false))
            } else {
                None
            }
        })
        .collect();

    let all_config_group_names =
        get_all_group_names(&groups, jetty_connector_names.keys().collect())?;

    for (group_name, group) in groups {
        // get all the node names for the given group
        let binding = all_config_group_names.clone();
        let node_names = binding
            .get(group_name)
            .ok_or(anyhow!("group {} not found in config", group_name))?;

        for (origin, node_name) in node_names {
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

                // Depends on whether nested groups are allowed
                let new = if jetty_connector_names[origin] {
                    users_to_node_names(&group.members.users)
                } else {
                    get_all_inherited_users(&groups, group_name)
                        .into_iter()
                        .collect()
                };

                // filter down to the relevant users
                let new = new
                    .into_iter()
                    .filter(|u| {
                        ag.get_node(u)
                            .unwrap()
                            .get_node_connectors()
                            .contains(origin)
                    })
                    .collect();

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

                // depends on whether nested groups are allowed
                let new = if jetty_connector_names[origin] {
                    groups_to_node_names(&group.members.groups, &all_config_group_names, origin)
                } else {
                    Vec::new()
                };

                // filter and transform to the groups that match the config
                let new = new
                    .into_iter()
                    .filter(|u| {
                        let node_name = ag.get_node(u).unwrap().get_node_name();

                        if let NodeName::Group {
                            origin: ag_origin, ..
                        } = node_name
                        {
                            &ag_origin == origin
                        } else {
                            false
                        }
                    })
                    .collect();

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
                            details: DiffDetails::ModifyGroup {
                                add: GroupMemberChanges {
                                    users: user_changes.add,
                                    groups: group_changes.add,
                                },
                                remove: GroupMemberChanges {
                                    users: user_changes.remove,
                                    groups: group_changes.remove,
                                },
                            },
                            connectors: match node_name {
                                NodeName::Group {origin, .. } => HashSet::from([origin.to_owned()]),
                                _ => bail!("internal error; expected to find a group, but found something else"),
                            },
                        },
                    );
                };
            } else {
                // if it doesn't exist, add a new group diff, with all the appropriate users
                group_diffs.insert(
                    node_name.clone(),
                    Diff {
                        group_name: node_name.clone(),
                        details: DiffDetails::AddGroup {
                            members: GroupMemberChanges {
                                users: users_to_node_names(&group.members.users),
                                groups: groups_to_node_names(&group.members.groups, &all_config_group_names, origin),
                            },
                        },
                        connectors: match node_name {
                            NodeName::Group {origin, .. } => HashSet::from([origin.to_owned()]),
                            _ => bail!("internal error; expected to find a group, but found something else"),
                        },
                    },
                );
            }
        }
    }

    // now iterate through all of the groups and drop any that don't exist
    for (k, v) in ag_groups {
        group_diffs.insert(
            k.clone(),
            Diff {
                group_name: k.clone(),
                details: DiffDetails::RemoveGroup,
                connectors: match k {
                    NodeName::Group { ref origin, .. } => HashSet::from([origin.to_owned()]),
                    _ => {
                        bail!("internal error; expected to find a group, but found something else")
                    }
                },
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

        // Iterate over the policies that we need to remove
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

            // iterate over the policy targets to build the diff structs
            for target in policy_targets {
                policy_diffs.push(policies::Diff {
                    asset: ag[target].get_node_name(),
                    agent: k.clone(),
                    details: vec![policies::DiffDetails::RemovePolicy],
                    connectors: policy.connectors.to_owned(),
                });
            }
        }
    }

    Ok(group_diffs)
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

fn groups_to_node_names(
    groups: &Option<Vec<MemberGroup>>,
    all_groups: &HashMap<String, HashMap<ConnectorNamespace, NodeName>>,
    origin: &ConnectorNamespace,
) -> Vec<NodeName> {
    match groups {
        Some(groups) => groups
            .iter()
            .map(|g| {
                all_groups
                    .get(&g.name)
                    .unwrap()
                    .get(origin)
                    .unwrap()
                    .to_owned()
            })
            .collect::<Vec<_>>(),
        None => Vec::new(),
    }
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

/// Given a config, get all the final, connector-scoped node names for the groups.
fn get_all_group_names(
    groups: &HashMap<String, GroupConfig>,
    jetty_connector_names: HashSet<&ConnectorNamespace>,
) -> Result<HashMap<String, HashMap<ConnectorNamespace, NodeName>>> {
    let mut res = HashMap::new();

    for (group_name, group_config) in groups {
        if let Some((prefix, suffix)) = group_name
            .split_once("::")
            .map(|p| (ConnectorNamespace(p.0.to_string()), p.1.to_owned()))
        {
            if !jetty_connector_names.contains(&prefix) {
                bail!("looking for connector with name `{}`, but there is no connector with that name", prefix);
            };
            res.insert(
                group_name.to_owned(),
                HashMap::from([(
                    prefix.to_owned(),
                    NodeName::Group {
                        name: suffix,
                        origin: prefix.to_owned(),
                    },
                )]),
            );
        } else {
            let mut inner_map = HashMap::new();

            // Iterate through the Jetty connectors
            for &n in &jetty_connector_names {
                // set the default group name
                let mut group_name = NodeName::Group {
                    name: group_name.to_owned(),
                    origin: n.to_owned(),
                };
                // are there custom names in the config?
                if let Some(custom_names) = &group_config.connector_names {
                    // look for a match
                    if let Some(g) = custom_names.iter().find_map(|f| {
                        if f.connector == *n {
                            Some(NodeName::Group {
                                name: f.alias.to_owned(),
                                origin: n.to_owned(),
                            })
                        } else {
                            None
                        }
                    }) {
                        // If there is a match, update group_name
                        group_name = g;
                    };
                };
                inner_map.insert(n.to_owned(), group_name);
            }
            res.insert(group_name.to_owned(), inner_map);
        };
    }
    Ok(res)
}

/// Return all of the users of a given group, including from nested groups
fn get_all_inherited_users(
    groups: &HashMap<String, GroupConfig>,
    target_group_name: &String,
) -> HashSet<NodeName> {
    let mut res = HashSet::new();
    // get the target groups
    let target_group = &groups[target_group_name];
    // Add the explicit users to the list
    if let Some(group_users) = &target_group.members.users {
        res.extend(group_users.iter().map(|u| NodeName::User(u.name.clone())));
    }
    // Add the groups users to the list
    if let Some(group_groups) = &target_group.members.groups {
        for g in group_groups {
            res.extend(get_all_inherited_users(groups, &g.name));
        }
    }
    res
}

#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use crate::{
        access_graph::{
            cual_to_asset_name_test, AssetAttributes, GroupAttributes, NodeName, TagAttributes,
            UserAttributes,
        },
        connectors::ConnectorCapabilities,
        cual::Cual,
        Connector,
    };

    use anyhow::Result;

    use super::{parser::parse_groups, *};

    fn get_test_graph() -> AccessGraph {
        AccessGraph::new_dummy(
            &[
                &JettyNode::Group(GroupAttributes {
                    name: NodeName::Group {
                        name: "g1".to_string(),
                        origin: ConnectorNamespace("c1".to_string()),
                    },
                    ..Default::default()
                }),
                &JettyNode::Group(GroupAttributes {
                    name: NodeName::Group {
                        name: "g2".to_string(),
                        origin: ConnectorNamespace("c2".to_string()),
                    },
                    ..Default::default()
                }),
                &JettyNode::Group(GroupAttributes {
                    name: NodeName::Group {
                        name: "g3".to_string(),
                        origin: ConnectorNamespace("c2".to_string()),
                    },
                    ..Default::default()
                }),
                &JettyNode::Group(GroupAttributes {
                    name: NodeName::Group {
                        name: "g3".to_string(),
                        origin: ConnectorNamespace("c1".to_string()),
                    },
                    ..Default::default()
                }),
                &JettyNode::User(UserAttributes {
                    name: NodeName::User("u1".to_owned()),
                    connectors: HashSet::from([
                        ConnectorNamespace("c1".to_string()),
                        ConnectorNamespace("c2".to_string()),
                    ]),
                    ..Default::default()
                }),
                &JettyNode::User(UserAttributes {
                    name: NodeName::User("u2".to_owned()),
                    connectors: HashSet::from([ConnectorNamespace("c1".to_string())]),
                    ..Default::default()
                }),
                &JettyNode::User(UserAttributes {
                    name: NodeName::User("u3".to_owned()),
                    connectors: HashSet::from([ConnectorNamespace("c2".to_string())]),
                    ..Default::default()
                }),
            ],
            &[
                (
                    NodeName::Group {
                        name: "g1".to_string(),
                        origin: ConnectorNamespace("c1".to_string()),
                    },
                    NodeName::User("u1".to_owned()),
                    EdgeType::Includes,
                ),
                (
                    NodeName::Group {
                        name: "g1".to_string(),
                        origin: ConnectorNamespace("c1".to_string()),
                    },
                    NodeName::User("u2".to_owned()),
                    EdgeType::Includes,
                ),
                (
                    NodeName::Group {
                        name: "g2".to_string(),
                        origin: ConnectorNamespace("c2".to_string()),
                    },
                    NodeName::User("u3".to_owned()),
                    EdgeType::Includes,
                ),
                (
                    NodeName::Group {
                        name: "g2".to_string(),
                        origin: ConnectorNamespace("c2".to_string()),
                    },
                    NodeName::Group {
                        name: "g3".to_string(),
                        origin: ConnectorNamespace("c2".to_string()),
                    },
                    EdgeType::Includes,
                ),
                (
                    NodeName::Group {
                        name: "g3".to_string(),
                        origin: ConnectorNamespace("c2".to_string()),
                    },
                    NodeName::User("u3".to_owned()),
                    EdgeType::Includes,
                ),
                (
                    NodeName::Group {
                        name: "g3".to_string(),
                        origin: ConnectorNamespace("c2".to_string()),
                    },
                    NodeName::User("u1".to_owned()),
                    EdgeType::Includes,
                ),
                (
                    NodeName::Group {
                        name: "g3".to_string(),
                        origin: ConnectorNamespace("c1".to_string()),
                    },
                    NodeName::User("u1".to_owned()),
                    EdgeType::Includes,
                ),
            ],
        )
    }

    struct DummyConn {
        nested: bool,
    }
    impl Connector for DummyConn {
        fn check<'life0, 'async_trait>(
            &'life0 self,
        ) -> core::pin::Pin<
            Box<dyn core::future::Future<Output = bool> + core::marker::Send + 'async_trait>,
        >
        where
            'life0: 'async_trait,
            Self: 'async_trait,
        {
            todo!()
        }

        fn get_data<'life0, 'async_trait>(
            &'life0 mut self,
        ) -> core::pin::Pin<
            Box<
                dyn core::future::Future<Output = crate::connectors::nodes::ConnectorData>
                    + core::marker::Send
                    + 'async_trait,
            >,
        >
        where
            'life0: 'async_trait,
            Self: 'async_trait,
        {
            todo!()
        }

        fn get_manifest(&self) -> crate::jetty::ConnectorManifest {
            crate::jetty::ConnectorManifest {
                capabilities: ConnectorCapabilities {
                    write: HashSet::from([WriteCapabilities::Groups {
                        nested: self.nested,
                    }]),
                    read: HashSet::new(),
                },
            }
        }
    }

    pub(crate) fn get_jetty() -> Jetty {
        let mut connectors: HashMap<ConnectorNamespace, Box<dyn Connector>> = HashMap::new();
        connectors.insert(
            ConnectorNamespace("c1".to_string()),
            Box::new(DummyConn { nested: true }),
        );
        connectors.insert(
            ConnectorNamespace("c2".to_string()),
            Box::new(DummyConn { nested: true }),
        );
        let mut jetty =
            Jetty::new_with_config(Default::default(), PathBuf::default(), connectors).unwrap();
        jetty.access_graph = Some(get_test_graph());
        jetty
    }

    #[test]
    fn try_anything() -> Result<()> {
        let jetty = get_jetty();
        let group_config = r#"
All Analysts:
    names:
        c1: ANALYSTS
    members:
        groups:
            - Sales Analysts
        users:
            - u1
            - u2

Sales Analysts:
    members:
        users:
            - u1
            - u2
"#;

        let parsed_config = parse_groups(&group_config.to_owned())?;

        let errors = validate_group_config(&parsed_config, &jetty);
        dbg!(errors);
        dbg!(generate_diff(&parsed_config, &jetty));
        Ok(())
    }

    #[test]
    fn no_change_no_diff() -> Result<()> {
        let jetty = get_jetty();
        let group_config = r#"
c1::g1:
    members:
        users:
            - u1
            - u2
c2::g2:
    members:
        users:
            - u3
        groups:
            - g3
g3:
    members:
        users:
            - u1
            - u3
"#;

        let parsed_config = parse_groups(&group_config.to_owned())?;

        let errors = validate_group_config(&parsed_config, &jetty);
        dbg!(errors);
        let diff = generate_diff(&parsed_config, &jetty);
        assert_eq!(diff?.len(), 0);
        Ok(())
    }

    #[test]
    fn inherited_users_works() -> Result<()> {
        let jetty = get_jetty();
        let group_config = r#"
c1::g1:
    members:
        users:
            - u1
            - u2
c2::g2:
    members:
        users:
            - u3
            - u2
        groups:
            - g3
g3:
    members:
        users:
            - u1
            - u3
"#;
        let parsed_config = parse_groups(&group_config.to_owned())?;

        let errors = validate_group_config(&parsed_config, &jetty);
        dbg!(errors);

        assert_eq!(
            get_all_inherited_users(&parsed_config, &"c2::g2".to_owned()),
            HashSet::from([
                NodeName::User("u1".to_owned()),
                NodeName::User("u2".to_owned()),
                NodeName::User("u3".to_owned())
            ])
        );

        assert_eq!(
            get_all_inherited_users(&parsed_config, &"g3".to_owned()),
            HashSet::from([
                NodeName::User("u1".to_owned()),
                NodeName::User("u3".to_owned())
            ])
        );

        Ok(())
    }
}
