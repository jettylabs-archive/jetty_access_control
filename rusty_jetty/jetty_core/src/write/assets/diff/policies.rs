//! Module to diff regular policies

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fmt::Display,
};

use colored::Colorize;

use crate::{
    access_graph::{NodeName},
    jetty::ConnectorNamespace,
    write::assets::{CombinedPolicyState, PolicyState},
};

#[derive(Debug)]
/// A diff of assets
pub struct PolicyDiff {
    /// The name of the asset being changed
    pub(crate) asset: NodeName,
    /// The map of users and their changes
    pub(crate) users: BTreeMap<NodeName, DiffDetails>,
    /// Same, but for groups
    pub(crate) groups: BTreeMap<NodeName, DiffDetails>,
    pub(crate) connectors: HashSet<ConnectorNamespace>,
}

#[derive(Debug, Clone)]
pub(crate) enum DiffDetails {
    AddAgent {
        add: PolicyState,
    },
    RemoveAgent,
    ModifyAgent {
        add: PolicyState,
        remove: PolicyState,
    },
}

#[derive(Debug, Default)]
struct PolicyDiffHelper {
    pub(crate) users: BTreeMap<NodeName, DiffDetails>,
    pub(crate) groups: BTreeMap<NodeName, DiffDetails>,
    pub(crate) connectors: HashSet<ConnectorNamespace>,
}

impl Display for PolicyDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut text = format!("asset: {}\n", self.asset);

        if !self.users.is_empty() {
            text += print_diff_inner_details(&self.users, "user: ").as_str();
        }

        if !self.groups.is_empty() {
            text += print_diff_inner_details(&self.groups, "group: ").as_str();
        }

        write!(f, "{text}")
    }
}

fn print_diff_inner_details(
    inner_details: &BTreeMap<NodeName, DiffDetails>,
    prefix: &str,
) -> String {
    let mut text = String::new();
    for (name, details) in inner_details {
        match details {
            DiffDetails::AddAgent { add } => {
                text += &format!(
                    "{}",
                    format!("  + {}{}\n", prefix, name)
                        .as_str()
                        .green()
                );
                if !add.privileges.is_empty() {
                    text += "      privileges:\n";
                    for privilege in &add.privileges {
                        text +=
                            &format!("{}", format!("        + {}\n", privilege).as_str().green());
                    }
                }
                if !add.metadata.is_empty() {
                    text += "      metadata:\n";
                    for (k, v) in &add.metadata {
                        text +=
                            &format!("{}", format!("{}{k}: {v}\n", "        + ").as_str().green());
                    }
                }
            }
            DiffDetails::RemoveAgent => {
                text += &format!(
                    "{}",
                    format!("  - {}{}\n", prefix, name)
                        .as_str()
                        .red()
                );
            }
            DiffDetails::ModifyAgent { add, remove } => {
                text += &format!(
                    "{}",
                    format!("  ~ {}{}\n", prefix, name)
                        .as_str()
                        .yellow()
                );
                if !add.privileges.is_empty() || !remove.privileges.is_empty() {
                    text += "      privileges:\n";
                    for privilege in &add.privileges {
                        text +=
                            &format!("{}", format!("        + {}\n", privilege).as_str().green());
                    }
                    for privilege in &remove.privileges {
                        text += &format!("{}", format!("        - {}\n", privilege).as_str().red());
                    }
                }
                if !add.metadata.is_empty() || !remove.metadata.is_empty() {
                    text += "      metadata:\n";
                    for (k, v) in &add.metadata {
                        text += &format!("{}", format!("        + {k}: {v}\n").as_str().green());
                    }
                    for (k, v) in &add.metadata {
                        text += &format!("{}", format!("        - {k}: {v}\n").as_str().red());
                    }
                }
            }
        }
    }
    text
}

/// diff from the environment state to the config state (additions are what will be added from config to env)
pub(crate) fn diff_policies(
    config: &CombinedPolicyState,
    env: &CombinedPolicyState,
) -> Vec<PolicyDiff> {
    // start with the regular policies
    let config_policies = &config.policies;
    let mut env_policies = env.policies.to_owned();

    let mut policy_diffs: HashMap<NodeName, PolicyDiffHelper> = HashMap::new();

    // iterate through each of the policies in the config. If it doesn't exist in the environment, add it to the policy_diffs.
    // If it does exist, remove it from my copy of the the environment and diff the details.
    for (config_key, config_value) in config_policies {
        let diff_details = match env_policies.remove(config_key) {
            Some(env_state) => {
                if &env_state == config_value {
                    continue;
                } else {
                    diff_policy_state(config_value, &env_state)
                }
            }
            // In this case, we're adding an agent
            None => DiffDetails::AddAgent {
                add: config_value.to_owned(),
            },
        };

        policy_diffs
            // add to the policy diff for the asset
            .entry(config_key.0.to_owned())
            .and_modify(|d| match &config_key.1 {
                NodeName::User(_) => {
                    d.users
                        .insert(config_key.1.to_owned(), diff_details.to_owned());
                }
                NodeName::Group { .. } => {
                    d.groups
                        .insert(config_key.1.to_owned(), diff_details.to_owned());
                }
                _ => panic!("got wrong node type while diffing"),
            })
            .or_insert({
                let mut d = PolicyDiffHelper::default();
                match &config_key.1 {
                    NodeName::User(_) => {
                        d.users.insert(config_key.1.to_owned(), diff_details);
                    }
                    NodeName::Group { .. } => {
                        d.groups.insert(config_key.1.to_owned(), diff_details);
                    }
                    _ => panic!("got wrong node type while diffing"),
                }
                d
            });
    }

    // Now iterate through whatever is left in the env_policies and add removal diffs
    for (env_key, _env_value) in &env_policies {
        let diff_details = DiffDetails::RemoveAgent;
        policy_diffs
            // add to the policy diff for the asset
            .entry(env_key.0.to_owned())
            .and_modify(|d| match &env_key.1 {
                NodeName::User(_) => {
                    d.users.insert(env_key.1.to_owned(), diff_details);
                }
                NodeName::Group { .. } => {
                    d.groups.insert(env_key.1.to_owned(), diff_details);
                }
                _ => panic!("got wrong node type while diffing"),
            });
    }

    let mut collected_diffs = policy_diffs
        .into_iter()
        .map(|(asset, helper)| PolicyDiff {
            asset,
            users: helper.users,
            groups: helper.groups,
            connectors: helper.connectors,
        })
        .collect::<Vec<_>>();
    collected_diffs.sort_by_key(|f| f.asset.to_string());

    collected_diffs
}

// Diff existing policies return an add and remove policy state
fn diff_policy_state(config: &PolicyState, env: &PolicyState) -> DiffDetails {
    // in config, not in env
    let add_privileges: HashSet<_> = config
        .privileges
        .difference(&env.privileges)
        .map(|p| p.to_owned())
        .collect();
    // in env, not in config
    let remove_privileges: HashSet<_> = env
        .privileges
        .difference(&config.privileges)
        .map(|p| p.to_owned())
        .collect();

    let config_metadata_set: HashSet<_> = config.metadata.iter().collect();
    let env_metadata_set: HashSet<_> = env.metadata.iter().collect();

    // in config, not in env
    let add_metadata: HashMap<_, _> = config_metadata_set
        .difference(&env_metadata_set)
        .map(|(k, v)| (k.to_owned().to_owned(), v.to_owned().to_owned()))
        .collect();
    // in env, not in config
    let remove_metadata: HashMap<_, _> = env_metadata_set
        .difference(&config_metadata_set)
        .map(|(k, v)| (k.to_owned().to_owned(), v.to_owned().to_owned()))
        .collect();

    DiffDetails::ModifyAgent {
        add: PolicyState {
            privileges: add_privileges,
            metadata: add_metadata,
        },
        remove: PolicyState {
            privileges: remove_privileges,
            metadata: remove_metadata,
        },
    }
}
