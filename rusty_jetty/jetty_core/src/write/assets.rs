//! Parse and manage user-configured policies

pub mod bootstrap;
pub mod parser;

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use anyhow::{bail, Result};
use glob::glob;
use petgraph::stable_graph::NodeIndex;
use serde::{Deserialize, Serialize};

use crate::{
    access_graph::{
        merge_map, AccessGraph, AssetPath, EdgeType, JettyNode, NodeName, PolicyAttributes,
    },
    connectors::AssetType,
    jetty::ConnectorNamespace,
    logging::warn,
    project, Jetty,
};

pub(crate) struct Diff {
    /// The name of the asset being changed
    pub(crate) asset: NodeName,
    /// The map of users and their changes
    pub(crate) users: BTreeMap<NodeName, DiffDetails>,
    /// Same, but for groups
    pub(crate) groups: BTreeMap<NodeName, DiffDetails>,
    pub(crate) connectors: HashSet<ConnectorNamespace>,
}

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

#[derive(Clone)]
pub(crate) struct PolicyState {
    privileges: HashSet<String>,
    metadata: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct YamlAssetDoc {
    identifier: YamlAssetIdentifier,
    policies: BTreeSet<YamlPolicy>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct YamlAssetIdentifier {
    name: String,
    asset_type: Option<AssetType>,
    connector: ConnectorNamespace,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct YamlPolicy {
    description: Option<String>,
    users: Option<BTreeSet<String>>,
    groups: Option<BTreeSet<String>>,
    metadata: Option<BTreeMap<String, String>>,
    privileges: BTreeSet<String>,
    /// this is specifically for default policies
    path: Option<String>,
    /// this is specifically for default policies - the types on which the policy should be applied
    types: Option<BTreeSet<AssetType>>,
}

/// State for policies and default policies
#[derive(Default, Clone)]
pub(crate) struct CombinedPolicyState {
    /// Represents the basic policies
    /// HashMap of <(NodeName::Asset, NodeName::User | NodeName::Group), PolicyState>
    policies: HashMap<(NodeName, NodeName), PolicyState>,
    /// Represents the future policies
    /// HashMap of <(NodeName::Asset, NodeName::User | NodeName::Group, wildcard path), PolicyState>
    default_policies:
        HashMap<(NodeName, NodeName, String, Option<BTreeSet<AssetType>>), PolicyState>,
}

/// Collect all the configurations and turn them into a combined policy state object
fn get_config_state(jetty: &Jetty) -> Result<CombinedPolicyState> {
    let mut res = CombinedPolicyState {
        ..Default::default()
    };

    // collect the paths to all the config files
    let paths = glob(
        format!(
            "{}/**/*.y*ml",
            project::assets_cfg_root_path().to_string_lossy()
        )
        .as_str(),
    )?;

    // read the files
    for path in paths {
        let path = path?;
        let yaml = std::fs::read_to_string(path)?;
        // parse the configs and extend the map. At this stage there should could be a user or a group mentioned
        // twice in two different policies, so we need to combine the policies. I guess. Which is weird
        // FUTURE: when we start using metadata, this could break. Policies will need to be keyed off metadata too, somehow.
        res.merge_combining_if_exists(parser::parse_asset_config(&yaml, jetty)?);
    }

    Ok(res)
}

/// Collect all the policy information from the environment and create a map of <(Asset, Agent) -> PolicyState)
fn get_env_state(jetty: &Jetty) -> Result<HashMap<(NodeName, NodeName), PolicyState>> {
    let ag = jetty.try_access_graph()?;

    // iterate through all the policies in the graph and fold them into the needed type
    let res = ag
        .graph
        .nodes
        .policies
        .iter()
        .fold(HashMap::new(), |mut acc, (name, &idx)| {
            let policy: PolicyAttributes = ag[idx].to_owned().try_into().unwrap();

            let agents = get_policy_agents(idx.into(), ag);
            let assets = get_policy_assets(idx.into(), ag);

            for agent in &agents {
                for asset in &assets {
                    acc.insert(
                        (asset.to_owned(), agent.to_owned()),
                        PolicyState {
                            privileges: policy.privileges.to_owned().into_iter().collect(),
                            metadata: Default::default(),
                        },
                    );
                }
            }

            acc
        });

    Ok(res)
}

fn get_policy_agents(idx: NodeIndex, ag: &AccessGraph) -> HashSet<NodeName> {
    let target_agents = ag.get_matching_children(
        idx,
        |e| matches!(e, EdgeType::GrantedTo),
        |_| false,
        |n| matches!(n, JettyNode::User(_)) || matches!(n, JettyNode::Group(_)),
        Some(1),
        Some(1),
    );
    target_agents
        .into_iter()
        .map(|n| ag[n].get_node_name().to_owned())
        .collect()
}

fn get_policy_assets(idx: NodeIndex, ag: &AccessGraph) -> HashSet<NodeName> {
    let target_assets = ag.get_matching_children(
        idx,
        |e| matches!(e, EdgeType::Governs),
        |_| false,
        |n| matches!(n, JettyNode::Asset(_)),
        Some(1),
        Some(1),
    );
    target_assets
        .into_iter()
        .map(|n| ag[n].get_node_name().to_owned())
        .collect()
}

impl CombinedPolicyState {
    /// Resolve all the default policies from the config into materialized policies.
    /// This takes into account the hierarchy of the default policies.
    fn expand_default_policies(&mut self, jetty: Jetty) -> Result<()> {
        let ag = jetty.try_access_graph()?;

        // prioritize default policies
        let mut prioritized_policies = HashMap::new();
        for (k, v) in self.default_policies.to_owned() {
            let asset_path = match &k.0 {
                NodeName::Asset { path, .. } => path,
                _ => bail!("expected an asset node"),
            };
            let wildcard_path = k.2.to_owned();
            prioritized_policies
                .entry(get_path_priority(wildcard_path, asset_path.to_owned()))
                .and_modify(
                    |combined_state: &mut HashMap<
                        (NodeName, NodeName, String, Option<BTreeSet<AssetType>>),
                        PolicyState,
                    >| {
                        combined_state.insert(k.to_owned(), v.to_owned());
                    },
                )
                .or_insert(HashMap::from([(k.to_owned(), v.to_owned())]));
        }

        // This intermediate map holds all of the policies, sorted by priority level. They are merged, combining policies and metadata if they already exist.
        let mut intermediate_map = HashMap::new();
        for (priority, default_policies) in prioritized_policies {
            intermediate_map.insert(
                priority.to_owned(),
                CombinedPolicyState {
                    ..Default::default()
                },
            );

            for ((root_node, grantee, matching_path, types), policy_state) in default_policies {
                let targets = ag.default_policy_targets(&NodeName::DefaultPolicy {
                    root_node: Box::new(root_node.to_owned()),
                    matching_path: matching_path.to_owned(),
                    grantee: Box::new(grantee.to_owned()),
                    types: types.to_owned(),
                })?;

                intermediate_map
                    .get_mut(&priority)
                    .unwrap()
                    .merge_combining_if_exists(CombinedPolicyState {
                        policies: targets
                            .iter()
                            .map(|&t| {
                                (
                                    (ag[t].get_node_name(), grantee.to_owned()),
                                    policy_state.to_owned(),
                                )
                            })
                            .collect(),
                        default_policies: Default::default(),
                    })?;
            }
        }

        // Now we go through each priority, from highest to lowest, and add the policies to self, skipping if exists
        let mut priority_levels = intermediate_map
            .keys()
            .map(|k| k.to_owned())
            .collect::<Vec<_>>();
        priority_levels.sort();
        for priority in priority_levels.into_iter().rev() {
            self.merge_skipping_if_exists(intermediate_map[&priority].to_owned());
        }

        Ok(())
    }

    /// merge a CombinedPolicyState struct into self, replacing entries if they exist
    fn merge_replacing_if_exists(&mut self, other: CombinedPolicyState) {
        self.policies.extend(other.policies);
        self.default_policies.extend(other.default_policies);
    }

    /// merge a CombinedPolicyState struct into self, but if a key already exists, don't replace it
    fn merge_skipping_if_exists(&mut self, other: CombinedPolicyState) {
        for (other_k, other_v) in other.policies {
            self.policies.entry(other_k).or_insert(other_v);
        }
        for (other_k, other_v) in other.default_policies {
            self.default_policies.entry(other_k).or_insert(other_v);
        }
    }

    /// merge a CombinedPolicyState struct into self, combining policy state if an entry already exists
    fn merge_combining_if_exists(&mut self, other: CombinedPolicyState) -> Result<()> {
        for (other_k, other_v) in other.policies {
            let existing_entry = self.policies.get_mut(&other_k);
            match existing_entry {
                Some(self_state) => {
                    // combine the privileges
                    self_state.privileges.extend(other_v.privileges);

                    // merge the metadata
                    for (k, v) in other_v.metadata.iter() {
                        let existing_val = self_state.metadata.get(k);
                        match existing_val {
                            Some(val) => {
                                if val != v {
                                    warn!("unable to merge asset configuration metadata for a policy for {other_k:?}. Values: {v}, {val}. Keeping {val}")
                                }
                            }
                            None => {
                                self_state.metadata.insert(k.to_owned(), v.to_owned());
                            }
                        }
                    }
                }
                None => {
                    self.policies.insert(other_k, other_v);
                }
            };
        }
        for (other_k, other_v) in other.default_policies {
            let existing_entry = self.default_policies.get_mut(&other_k);
            match existing_entry {
                Some(self_state) => {
                    // combine the privileges
                    self_state.privileges.extend(other_v.privileges);

                    // merge the metadata
                    for (k, v) in other_v.metadata.iter() {
                        let existing_val = self_state.metadata.get(k);
                        match existing_val {
                            Some(val) => {
                                if val != v {
                                    warn!("unable to merge asset configuration metadata for a policy for {other_k:?}. Values: {v}, {val}. Keeping {val}")
                                }
                            }
                            None => {
                                self_state.metadata.insert(k.to_owned(), v.to_owned());
                            }
                        }
                    }
                }
                None => {
                    self.default_policies.insert(other_k, other_v);
                }
            };
        }
        Ok(())
    }
}

fn get_path_priority(wildcard_path: String, path: AssetPath) -> String {
    let segments = wildcard_path.split("/").collect::<Vec<_>>();
    let mut path_score = format!("{:03}", path.components().len());
    for segment in segments {
        if segment == "*" {
            path_score += "2"
        } else if segment == "**" {
            path_score += "1"
        }
    }
    path_score
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_path_priority() {
        let x = super::get_path_priority(
            "/*/**".to_string(),
            AssetPath::new(["a".to_owned(), "b".to_owned()].into()),
        );
        dbg!(x);
    }
}
