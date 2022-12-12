//! Parse and manage user-configured groups

pub(crate) mod bootstrap;
mod parser;

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    fs,
};

use anyhow::{anyhow, bail, Result};
use colored::Colorize;
use serde::Deserialize;

use crate::{
    access_graph::{AccessGraph, EdgeType, JettyNode, NodeName, PolicyAttributes},
    connectors::WriteCapabilities,
    jetty::ConnectorNamespace,
    project,
    write::parser_common::indicated_msg,
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

#[derive(Debug, Clone)]
/// A Diff for groups
pub struct Diff {
    /// the group being diffed
    pub group_name: NodeName,
    /// The specifics of the diff
    pub details: DiffDetails,
    /// The connector the diff should be applied to
    pub connector: ConnectorNamespace,
}

#[derive(Debug, Clone)]
/// Outlines the diff type needed
pub enum DiffDetails {
    /// Add a group
    AddGroup {
        /// the members of the group
        members: GroupMemberChanges,
    },
    /// Remove a group
    RemoveGroup,
    /// Update a group
    ModifyGroup {
        /// members that are added
        add: GroupMemberChanges,
        /// members that are removed
        remove: GroupMemberChanges,
    },
}

#[derive(Debug, Clone)]
/// Structure showing changes within group members
pub struct GroupMemberChanges {
    /// users
    pub users: Vec<NodeName>,
    /// groups
    pub groups: Vec<NodeName>,
}

/// Structure showing how group membership is changing
struct NodeNameListDiff {
    /// members being added
    pub add: Vec<NodeName>,
    /// members being dropped
    pub remove: Vec<NodeName>,
}

impl Display for Diff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut text = "".to_owned();
        match &self.details {
            DiffDetails::AddGroup { members } => {
                text += format!(
                    "{}{}\n",
                    "+ group: ".green(),
                    self.group_name.to_string().green()
                )
                .as_str();
                if !members.users.is_empty() {
                    text += "    users:\n"
                };
                for user in &members.users {
                    text +=
                        format!("{}{}\n", "      + ".green(), user.to_string().green()).as_str();
                }
                if !members.groups.is_empty() {
                    text += "    groups:\n"
                };
                for group in &members.groups {
                    text +=
                        format!("{}{}\n", "      + ".green(), group.to_string().green()).as_str();
                }
            }
            DiffDetails::RemoveGroup => {
                text += format!(
                    "{}{}\n",
                    "- group: ".red(),
                    self.group_name.to_string().red()
                )
                .as_str();
            }
            DiffDetails::ModifyGroup { add, remove } => {
                text += format!(
                    "{}{}\n",
                    "~ group: ".yellow(),
                    self.group_name.to_string().yellow()
                )
                .as_str();
                if !add.users.is_empty() || !remove.users.is_empty() {
                    text += "    users:\n"
                };
                for user in &add.users {
                    text +=
                        format!("{}{}\n", "      + ".green(), user.to_string().green()).as_str();
                }
                for user in &remove.users {
                    text += format!("{}{}\n", "      - ".red(), user.to_string().red()).as_str();
                }
                if !add.groups.is_empty() || !remove.groups.is_empty() {
                    text += "    groups:\n"
                };
                for group in &add.groups {
                    text +=
                        format!("{}{}\n", "      + ".green(), group.to_string().green()).as_str();
                }
                for group in &remove.groups {
                    text += format!("{}{}\n", "      - ".red(), group.to_string().red()).as_str();
                }
            }
        }
        write!(f, "{text}")
    }
}

/// Validate group config by making sure that users, groups, and listed connectors exist. Returns a vec of errors. If the vec is empty, there were no errors.
/// This allows all errors to be displayed at once.
fn validate_group_config(
    groups: &BTreeMap<String, GroupConfig>,
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
                if !jetty.connectors.contains_key(&n.connector) {
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
    groups: &BTreeMap<String, GroupConfig>,
    jetty: &Jetty,
) -> Result<BTreeMap<NodeName, Diff>> {
    let mut group_diffs = BTreeMap::new();
    let mut policy_diffs = Vec::new();

    let ag = jetty.access_graph.as_ref().ok_or_else(|| {
        anyhow!("jetty initialized without an access graph; try running `jetty fetch` first")
    })?;

    let mut ag_groups = ag.graph.nodes.groups.clone();

    // Writing groups is limited to the connectors that have the notion of a groups, and we keep information about whether
    // nested groups are allowed
    let jetty_connector_names: BTreeMap<ConnectorNamespace, bool> = jetty
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
        // get all the node names for the given group. These would be the local names of the groups that need to be created
        let binding = all_config_group_names.clone();
        let node_names = binding
            .get(group_name)
            .ok_or(anyhow!("group {} not found in config", group_name))?;

        for (origin, node_name) in node_names {
            // get all the legal groups and users for the node

            // first, get all the users, depending on whether nested groups are allowed
            let legal_users = if jetty_connector_names[origin] {
                users_to_node_names(&group.members.users)
            } else {
                get_all_inherited_users(&groups, group_name)
                    .into_iter()
                    .collect()
            };

            // now filter down to the relevant users (those with the right connector)
            let legal_users = legal_users
                .into_iter()
                .filter(|u| {
                    ag.get_node(u)
                        .unwrap()
                        .get_node_connectors()
                        .contains(origin)
                })
                .collect();

            // now get all the legal groups
            // depends on whether nested groups are allowed
            let legal_groups = if jetty_connector_names[origin] {
                // This function already filters out ineligible groups, based on connector
                groups_to_node_names(&group.members.groups, &all_config_group_names, origin)
            } else {
                Vec::new()
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

                let user_changes = diff_node_names(&old, &legal_users);

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

                let group_changes = diff_node_names(&old, &legal_groups);

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
                            connector: match node_name {
                                NodeName::Group {origin, .. } => origin.to_owned(),
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
                                users: legal_users,
                                groups: legal_groups,
                            },
                        },
                        connector: match node_name {
                            NodeName::Group {origin, .. } => origin.to_owned(),
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
                connector: match k {
                    NodeName::Group { ref origin, .. } => origin.to_owned(),
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
                    assets: vec![ag[target].get_node_name()],
                    agents: vec![k.clone()],
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

/// This turns group names into node names, filtering out those don't exist on the relevant connector
fn groups_to_node_names(
    groups: &Option<Vec<MemberGroup>>,
    all_groups: &BTreeMap<String, BTreeMap<ConnectorNamespace, NodeName>>,
    origin: &ConnectorNamespace,
) -> Vec<NodeName> {
    match groups {
        Some(groups) => groups
            .iter()
            // Using a filter_map because there are some cases where a group doesn't
            // (and shouldn't exit) for a connector, but it will still look for it
            .filter_map(|g| {
                all_groups
                    .get(&g.name)
                    .unwrap()
                    .get(origin)
                    .map(|n| n.to_owned())
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
    groups: &BTreeMap<String, GroupConfig>,
    jetty_connector_names: BTreeSet<&ConnectorNamespace>,
) -> Result<BTreeMap<String, BTreeMap<ConnectorNamespace, NodeName>>> {
    let mut res = BTreeMap::new();

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
                BTreeMap::from([(
                    prefix.to_owned(),
                    NodeName::Group {
                        name: suffix,
                        origin: prefix.to_owned(),
                    },
                )]),
            );
        } else {
            let mut inner_map = BTreeMap::new();

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
    groups: &BTreeMap<String, GroupConfig>,
    target_group_name: &String,
) -> BTreeSet<NodeName> {
    let mut res = BTreeSet::new();
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

/// Return the diff between the configuration and current state
pub fn get_group_diff(jetty: &Jetty) -> Result<Vec<Diff>> {
    // first, read the config files
    let group_config = fs::read_to_string(project::groups_cfg_path_local())?;
    // parse
    let parsed_config = parser::parse_groups(&group_config)?;
    // validate
    let validation_errors = validate_group_config(&parsed_config, jetty);

    if !validation_errors.is_empty() {
        let error_message = validation_errors
            .iter()
            .map(|e| {
                format!(
                    "error at {}\n{}\n",
                    indicated_msg(group_config.as_bytes(), e.pos, 2),
                    e.message
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");
        bail!(error_message);
    };

    let mut diff_vec = generate_diff(&parsed_config, jetty)?
        .into_values()
        .collect::<Vec<_>>();
    diff_vec.sort_by(|a, b| {
        a.group_name
            .to_string()
            .partial_cmp(&b.group_name.to_string())
            .unwrap()
    });
    Ok(diff_vec)
}

#[cfg(test)]
mod tests {

    use std::{
        collections::{HashMap, HashSet},
        path::PathBuf,
    };

    use crate::{
        access_graph::{
            cual_to_asset_name_test, translate::diffs::LocalDiffs, AssetAttributes,
            GroupAttributes, NodeName, TagAttributes, UserAttributes,
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

        fn plan_changes(&self, diffs: &LocalDiffs) -> Vec<String> {
            todo!()
        }

        fn apply_changes<'life0, 'life1, 'async_trait>(
            &'life0 self,
            diffs: &'life1 LocalDiffs,
        ) -> core::pin::Pin<
            Box<
                dyn core::future::Future<Output = Result<String>>
                    + core::marker::Send
                    + 'async_trait,
            >,
        >
        where
            'life0: 'async_trait,
            'life1: 'async_trait,
            Self: 'async_trait,
        {
            todo!()
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
c1::g1:
    members:
        users:
            - u1
"#;

        let parsed_config = parse_groups(&group_config.to_owned())?;

        let errors = validate_group_config(&parsed_config, &jetty);
        dbg!(errors);
        let diff_map = generate_diff(&parsed_config, &jetty)?;
        for diff in diff_map.values() {
            println!("{diff}");
        }

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
            BTreeSet::from([
                NodeName::User("u1".to_owned()),
                NodeName::User("u2".to_owned()),
                NodeName::User("u3".to_owned())
            ])
        );

        assert_eq!(
            get_all_inherited_users(&parsed_config, &"g3".to_owned()),
            BTreeSet::from([
                NodeName::User("u1".to_owned()),
                NodeName::User("u3".to_owned())
            ])
        );

        Ok(())
    }
}
