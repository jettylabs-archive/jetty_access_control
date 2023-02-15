//! Module to diff default policies

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fmt::Display,
};

use colored::Colorize;

use crate::{
    access_graph::NodeName,
    connectors::AssetType,
    jetty::ConnectorNamespace,
    write::{
        assets::{CombinedPolicyState, DefaultPolicyState},
        utils::diff_btreeset,
        SplitByConnector,
    },
};

#[derive(Debug, Clone)]
pub(crate) enum DefaultPolicyDiffDetails {
    Add {
        add: DefaultPolicyState,
    },
    Remove {
        remove: DefaultPolicyState,
    },
    Modify {
        add: DefaultPolicyState,
        remove: DefaultPolicyState,
        connector_managed: ConnectorManagementDiff,
    },
}

#[derive(Debug, Clone, Default)]
/// A diff of Default Policies
pub struct DefaultPolicyDiff {
    /// The name of the root asset
    pub(crate) asset: NodeName,
    /// The wildcard path for the default assets
    pub(crate) path: String,
    /// The type of asset that the policy is being applied to
    pub(crate) asset_type: AssetType,
    /// The map of users and their changes
    pub(crate) users: BTreeMap<NodeName, DefaultPolicyDiffDetails>,
    /// Same, but for groups
    pub(crate) groups: BTreeMap<NodeName, DefaultPolicyDiffDetails>,
    pub(crate) connector: ConnectorNamespace,
}

impl Display for DefaultPolicyDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut text = format!(
            "policy: {}\n  path: {}\n  asset type: {}\n",
            self.asset,
            self.path,
            self.asset_type.to_string()
        );

        if !self.users.is_empty() {
            text += print_diff_inner_details(&self.users, "user: ").as_str();
        }

        if !self.groups.is_empty() {
            text += print_diff_inner_details(&self.groups, "group: ").as_str();
        }

        write!(f, "{text}")
    }
}

impl SplitByConnector for DefaultPolicyDiff {
    fn split_by_connector(&self) -> HashMap<ConnectorNamespace, Box<Self>> {
        [(self.connector.to_owned(), Box::new(self.to_owned()))].into()
    }
}

fn print_diff_inner_details(
    inner_details: &BTreeMap<NodeName, DefaultPolicyDiffDetails>,
    prefix: &str,
) -> String {
    let mut text = String::new();
    for (name, details) in inner_details {
        match details {
            DefaultPolicyDiffDetails::Add { add } => {
                text += &format!("{}", format!("  + {prefix}{name}\n").as_str().green());

                text += &format!(
                    "{}",
                    format!("{}connector-managed: {}\n", "    ", add.connector_managed)
                        .as_str()
                        .green()
                );

                if !add.privileges.is_empty() {
                    text += "    privileges:\n";
                    for privilege in &add.privileges {
                        text += &format!("{}", format!("      + {privilege}\n").as_str().green());
                    }
                }
                if !add.metadata.is_empty() {
                    text += "    metadata:\n";
                    for (k, v) in &add.metadata {
                        text +=
                            &format!("{}", format!("{}{k}: {v}\n", "      + ").as_str().green());
                    }
                }
            }
            DefaultPolicyDiffDetails::Remove { remove } => {
                text += &format!("{}", format!("  - {prefix}{name}\n").as_str().red());

                text += &format!(
                    "{}",
                    format!(
                        "{}connector-managed: {}\n",
                        "    ", remove.connector_managed
                    )
                    .as_str()
                    .green()
                );

                if !remove.privileges.is_empty() {
                    text += "    privileges:\n";
                    for privilege in &remove.privileges {
                        text += &format!("{}", format!("      - {privilege}\n").as_str().red());
                    }
                }
                if !remove.metadata.is_empty() {
                    text += "    metadata:\n";
                    for (k, v) in &remove.metadata {
                        text += &format!("{}", format!("{}{k}: {v}\n", "      - ").as_str().red());
                    }
                }
            }
            DefaultPolicyDiffDetails::Modify {
                add,
                remove,
                connector_managed,
            } => {
                text += &format!("{}", format!("  ~ {prefix}{name}\n").as_str().yellow());

                match connector_managed {
                    ConnectorManagementDiff::Changed(v) => {
                        text += &format!(
                            "{}",
                            format!("{}connector-managed: {}\n", "    ", v)
                                .as_str()
                                .yellow()
                        )
                    }
                    ConnectorManagementDiff::Unchanged(_) => (),
                }

                if !add.privileges.is_empty() || !remove.privileges.is_empty() {
                    text += "    privileges:\n";
                    for privilege in &add.privileges {
                        text += &format!("{}", format!("      + {privilege}\n").as_str().green());
                    }
                    for privilege in &remove.privileges {
                        text += &format!("{}", format!("      - {privilege}\n").as_str().red());
                    }
                }
                if !add.metadata.is_empty() || !remove.metadata.is_empty() {
                    text += "      metadata:\n";
                    for (k, v) in &add.metadata {
                        text += &format!("{}", format!("      + {k}: {v}\n").as_str().green());
                    }
                    for (k, v) in &remove.metadata {
                        text += &format!("{}", format!("      - {k}: {v}\n").as_str().red());
                    }
                }
            }
        }
    }
    text
}

#[derive(Debug, Default)]
struct DefaultPolicyDiffHelper {
    pub(crate) users: BTreeMap<NodeName, DefaultPolicyDiffDetails>,
    pub(crate) groups: BTreeMap<NodeName, DefaultPolicyDiffDetails>,
    pub(crate) connector: ConnectorNamespace,
}

#[derive(Debug, Clone)]
pub(crate) enum ConnectorManagementDiff {
    Changed(bool),
    Unchanged(bool),
}

/// Diff existing default policy states return an add and remove policy state
fn diff_default_policy_state(
    config: &DefaultPolicyState,
    env: &DefaultPolicyState,
) -> DefaultPolicyDiffDetails {
    let (add_privileges, remove_privileges) = add_and_remove(&config.privileges, &env.privileges);

    let config_metadata_set = config.metadata.iter().collect();
    let env_metadata_set = env.metadata.iter().collect();

    let (add_metadata, remove_metadata) = add_and_remove(&config_metadata_set, &env_metadata_set);

    DefaultPolicyDiffDetails::Modify {
        add: DefaultPolicyState {
            privileges: add_privileges,
            metadata: add_metadata
                .into_iter()
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .collect(),
            // refer to the field on the diff to know if this has changed
            connector_managed: false,
        },
        remove: DefaultPolicyState {
            privileges: remove_privileges,
            metadata: remove_metadata
                .into_iter()
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .collect(),
            // refer to the field on the diff to know if this has changed
            connector_managed: false,
        },
        connector_managed: match config.connector_managed == env.connector_managed {
            true => ConnectorManagementDiff::Unchanged(config.connector_managed),
            false => ConnectorManagementDiff::Changed(config.connector_managed),
        },
    }
}

// Given two sets of T, create two sets containing the differences
fn add_and_remove<T: Ord + Clone>(
    config: &BTreeSet<T>,
    env: &BTreeSet<T>,
) -> (BTreeSet<T>, BTreeSet<T>) {
    // in config, not in env
    let (add, remove) = diff_btreeset(config, env);

    (add.collect(), remove.collect())
}

/// diff from the environment state to the config state (additions are what will be added from config to env)
pub(crate) fn diff_default_policies(
    config: &CombinedPolicyState,
    env: &CombinedPolicyState,
) -> Vec<DefaultPolicyDiff> {
    // start with the regular policies
    let config_policies = &config.default_policies;
    let mut env_policies = env.default_policies.to_owned();

    let mut policy_diffs: HashMap<(NodeName, String, AssetType), DefaultPolicyDiffHelper> =
        HashMap::new();

    // iterate through each of the policies in the config. If it doesn't exist in the environment, add it to the policy_diffs.
    // If it does exist, remove it from my copy of the the environment and diff the details.
    for (config_key, config_value) in config_policies {
        let diff_details = match env_policies.remove(config_key) {
            Some(env_state) => {
                if &env_state == config_value {
                    continue;
                } else {
                    diff_default_policy_state(config_value, &env_state)
                }
            }
            // In this case, we're adding an agent
            None => DefaultPolicyDiffDetails::Add {
                add: config_value.to_owned(),
            },
        };

        policy_diffs
            // add to the policy diff for the asset
            .entry((
                config_key.0.to_owned(),
                config_key.1.to_owned(),
                config_key.2.to_owned(),
            ))
            .and_modify(|helper: &mut DefaultPolicyDiffHelper| match &config_key.3 {
                NodeName::User(_) => {
                    helper
                        .users
                        .insert(config_key.3.to_owned(), diff_details.to_owned());
                }
                NodeName::Group { .. } => {
                    helper
                        .groups
                        .insert(config_key.3.to_owned(), diff_details.to_owned());
                }
                _ => panic!("got wrong node type while diffing"),
            })
            .or_insert({
                let mut helper = DefaultPolicyDiffHelper {
                    connector: match &config_key.0 {
                        NodeName::Asset { connector, .. } => connector.to_owned(),
                        _ => panic!("got wrong node type while diffing"),
                    },
                    ..Default::default()
                };
                match &config_key.3 {
                    NodeName::User(_) => {
                        helper.users.insert(config_key.3.to_owned(), diff_details);
                    }
                    NodeName::Group { .. } => {
                        helper.groups.insert(config_key.3.to_owned(), diff_details);
                    }
                    _ => panic!("got wrong node type while diffing"),
                }
                helper
            });
    }

    // Now iterate through whatever is left in the env_policies and add removal diffs
    for (env_key, env_value) in &env_policies {
        // These will always be connector managed, otherwise they wouldn't show up at all
        let diff_details = DefaultPolicyDiffDetails::Remove {
            remove: env_value.to_owned(),
        };
        policy_diffs
            // add to the policy diff for the asset
            .entry((
                env_key.0.to_owned(),
                env_key.1.to_owned(),
                env_key.2.to_owned(),
            ))
            .and_modify(|helper| match &env_key.3 {
                NodeName::User(_) => {
                    helper
                        .users
                        .insert(env_key.3.to_owned(), diff_details.to_owned());
                }
                NodeName::Group { .. } => {
                    helper
                        .groups
                        .insert(env_key.3.to_owned(), diff_details.to_owned());
                }
                _ => panic!("got wrong node type while diffing"),
            })
            .or_insert({
                let mut helper = DefaultPolicyDiffHelper {
                    connector: match &env_key.0 {
                        NodeName::Asset { connector, .. } => connector.to_owned(),
                        _ => panic!("got wrong node type while diffing"),
                    },
                    ..Default::default()
                };
                match &env_key.3 {
                    NodeName::User(_) => {
                        helper
                            .users
                            .insert(env_key.3.to_owned(), diff_details.to_owned());
                    }
                    NodeName::Group { .. } => {
                        helper
                            .groups
                            .insert(env_key.3.to_owned(), diff_details.to_owned());
                    }
                    _ => panic!("got wrong node type while diffing"),
                }
                helper
            });
    }

    let mut collected_diffs = policy_diffs
        .into_iter()
        .map(
            |((root_asset, path, asset_type), helper)| DefaultPolicyDiff {
                users: helper.users,
                groups: helper.groups,
                connector: helper.connector,
                path,
                asset_type,
                asset: root_asset,
            },
        )
        .collect::<Vec<_>>();
    collected_diffs.sort_by_key(|f| f.asset.to_string());

    collected_diffs
}
