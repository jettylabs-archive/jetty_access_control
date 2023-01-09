//! Parse and manage user-configured policies

pub mod bootstrap;
pub mod diff;
pub mod parser;
mod update;

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    path::PathBuf,
};

use anyhow::{bail, Context, Result};
use glob::{glob, Paths};
use petgraph::stable_graph::NodeIndex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    access_graph::{
        AccessGraph, AssetPath, DefaultPolicyAttributes, EdgeType, JettyNode, NodeName,
        PolicyAttributes,
    },
    connectors::AssetType,
    jetty::ConnectorNamespace,
    logging::warn,
    project, Jetty,
};

use self::diff::{
    default_policies::{diff_default_policies, DefaultPolicyDiff},
    policies::{diff_policies, PolicyDiff},
};

use super::groups::{get_group_capable_connectors, get_group_to_nodename_map, GroupYaml};

pub(crate) use update::{remove_group_name, remove_user_name, update_group_name, update_user_name};

/// Policy state
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct PolicyState {
    /// Included privileges
    pub privileges: HashSet<String>,
    /// Included metadata
    pub metadata: HashMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DefaultPolicyState {
    pub(crate) privileges: BTreeSet<String>,
    pub(crate) metadata: HashMap<String, String>,
    pub(crate) connector_managed: bool,
}
#[derive(Serialize, Deserialize, Debug, Default, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) struct YamlAssetDoc {
    identifier: YamlAssetIdentifier,
    #[serde(skip_serializing_if = "BTreeSet::is_empty", default)]
    policies: BTreeSet<YamlPolicy>,
    #[serde(
        skip_serializing_if = "BTreeSet::is_empty",
        default,
        rename = "default policies"
    )]
    default_policies: BTreeSet<YamlDefaultPolicy>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) struct YamlAssetIdentifier {
    name: String,
    // FUTURE: Make asset_type required, not an option
    #[serde(skip_serializing_if = "Option::is_none", rename = "asset type")]
    asset_type: Option<AssetType>,
    connector: ConnectorNamespace,
    id: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct YamlPolicy {
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    users: Option<BTreeSet<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    groups: Option<BTreeSet<String>>,
    privileges: Option<BTreeSet<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<BTreeMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct YamlDefaultPolicy {
    /// this is specifically for default policies
    path: String,
    /// this is specifically for default policies - the types on which the policy should be applied
    #[serde(rename = "target type")]
    target_type: AssetType,
    #[serde(skip_serializing_if = "bool_is_false", default)]
    /// Whether this default policy is managed by the connector (rather than just by Jetty)
    #[serde(rename = "connector-managed")]
    connector_managed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    users: Option<BTreeSet<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    groups: Option<BTreeSet<String>>,
    privileges: Option<BTreeSet<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<BTreeMap<String, String>>,
}

fn bool_is_false(v: &bool) -> bool {
    !v
}

/// State for policies and default policies
#[derive(Default, Clone, Debug)]
pub(crate) struct CombinedPolicyState {
    /// Represents the basic policies
    /// HashMap of <(NodeName::Asset, NodeName::User | NodeName::Group), PolicyState>
    policies: HashMap<(NodeName, NodeName), PolicyState>,
    /// Represents the future policies
    /// HashMap of <(NodeName::Asset, wildcard path, Asset Types, Grantee), DefaultPolicyState>
    default_policies: HashMap<(NodeName, String, AssetType, NodeName), DefaultPolicyState>,
}

/// Collect all the configurations and turn them into a combined policy state object
fn get_config_state(
    jetty: &Jetty,
    validated_group_config: &BTreeSet<GroupYaml>,
) -> Result<CombinedPolicyState> {
    let mut res = CombinedPolicyState {
        ..Default::default()
    };

    // collect the paths to all the config files
    let paths = get_config_paths()?;

    // We need to get the connectors that can handle groups. With how things are set up, that means that if ever there's a connector that uses groups, and we want to manage with
    // groups, but that doesn't write them, we'll need to fix it here.
    // There is also the case in which groups don't exist, just users. If that's the case, jetty groups should just transform into users.
    // FUTURE: Improve this
    let connectors = &get_group_capable_connectors(jetty)
        .keys()
        .cloned()
        .collect();
    let config_groups = get_group_to_nodename_map(validated_group_config, connectors);

    // read the files
    for path in paths {
        let path = path?;
        let yaml = std::fs::read_to_string(&path)?;
        // parse the configs and extend the map. At this stage there should could be a user or a group mentioned
        // twice in two different policies, so we need to combine the policies. I guess. Which is weird
        // FUTURE: when we start using metadata, this could break. Policies will need to be keyed off metadata too, somehow.
        let (_identifier, policy_state) = parser::parse_asset_config(&yaml, jetty, &config_groups)
            .context(format!(
                "problem with configuration file: {}",
                path.to_string_lossy()
            ))?;
        res.merge_combining_if_exists(policy_state)?;
    }

    // Now that we've built out the state, expand the default policies:
    res.expand_default_policies(jetty)?;

    // and remove non-connector-managed default policies
    res.remove_non_connector_managed_default_policies();

    Ok(res)
}

/// Collect all the policy information from the environment and create a map of <(Asset, Agent) -> PolicyState)
fn get_env_state(jetty: &Jetty) -> Result<CombinedPolicyState> {
    let ag = jetty.try_access_graph()?;

    // iterate through all the policies in the graph and fold them into the needed type
    let policies = ag
        .graph
        .nodes
        .policies
        .iter()
        .fold(HashMap::new(), |mut acc, (_name, &idx)| {
            let policy: PolicyAttributes = ag[idx].to_owned().try_into().unwrap();

            let agents = get_policy_agents(idx.into(), ag);
            let assets = get_policy_assets(idx.into(), ag);

            for agent in &agents {
                for asset in &assets {
                    acc.insert(
                        (asset.to_owned(), agent.to_owned()),
                        PolicyState {
                            privileges: policy.privileges.iter().cloned().collect(),
                            metadata: Default::default(),
                        },
                    );
                }
            }

            acc
        });

    let default_policies =
        ag.graph
            .nodes
            .default_policies
            .iter()
            .fold(HashMap::new(), |mut acc, (_name, &idx)| {
                let policy: DefaultPolicyAttributes = ag[idx].to_owned().try_into().unwrap();

                let agents = get_policy_agents(idx.into(), ag);
                let root_asset = get_default_policy_root_asset(idx.into(), ag);
                let path = policy.matching_path;
                let types = policy.target_type;

                for agent in &agents {
                    acc.insert(
                        (
                            root_asset.to_owned(),
                            path.to_owned(),
                            types.to_owned(),
                            agent.to_owned(),
                        ),
                        DefaultPolicyState {
                            privileges: policy.privileges.iter().cloned().collect(),
                            metadata: policy.metadata.to_owned(),
                            // We're getting this from the graph - only connector-managed default policies appear in the graph
                            connector_managed: true,
                        },
                    );
                }

                acc
            });
    Ok(CombinedPolicyState {
        policies,
        default_policies,
    })
}

/// Get the policy diffs for regular policies
pub fn get_policy_diffs(
    jetty: &Jetty,
    validated_group_config: &BTreeSet<GroupYaml>,
) -> Result<Vec<PolicyDiff>> {
    let config_state = get_config_state(jetty, validated_group_config)?;
    let env_state = get_env_state(jetty)?;

    Ok(diff_policies(&config_state, &env_state))
}

/// Get the paths of all asset config files
fn get_config_paths() -> Result<glob::Paths> {
    // collect the paths to all the config files
    glob(
        format!(
            "{}/**/*.y*ml",
            project::assets_cfg_root_path_local().to_string_lossy()
        )
        .as_str(),
    )
    .context("trouble generating config file paths")
}

/// Get the policy diffs for default policies
pub fn get_default_policy_diffs(
    jetty: &Jetty,
    validated_group_config: &BTreeSet<GroupYaml>,
) -> Result<Vec<DefaultPolicyDiff>> {
    let config_state = get_config_state(jetty, validated_group_config)?;
    let env_state = get_env_state(jetty)?;

    Ok(diff_default_policies(&config_state, &env_state))
}

fn get_policy_agents(idx: NodeIndex, ag: &AccessGraph) -> HashSet<NodeName> {
    let target_agents = ag.get_matching_descendants(
        idx,
        |e| matches!(e, EdgeType::GrantedTo),
        |_| false,
        |n| matches!(n, JettyNode::User(_)) || matches!(n, JettyNode::Group(_)),
        Some(1),
        Some(1),
    );
    target_agents
        .into_iter()
        .map(|n| ag[n].get_node_name())
        .collect()
}

fn get_policy_assets(idx: NodeIndex, ag: &AccessGraph) -> HashSet<NodeName> {
    let target_assets = ag.get_matching_descendants(
        idx,
        |e| matches!(e, EdgeType::Governs),
        |_| false,
        |n| matches!(n, JettyNode::Asset(_)),
        Some(1),
        Some(1),
    );
    target_assets
        .into_iter()
        .map(|n| ag[n].get_node_name())
        .collect()
}

/// Get the root node for the default policy. There should only be one of these
fn get_default_policy_root_asset(idx: NodeIndex, ag: &AccessGraph) -> NodeName {
    let target_assets = ag.get_matching_descendants(
        idx,
        |e| matches!(e, EdgeType::ProvidedDefaultForChildren),
        |_| false,
        |n| matches!(n, JettyNode::Asset(_)),
        Some(1),
        Some(1),
    );
    let nodes = target_assets
        .into_iter()
        .map(|n| ag[n].get_node_name())
        .collect::<Vec<_>>();
    if nodes.len() > 1 {
        panic!("a default policy should never have more than one root node")
    };
    if nodes.is_empty() {
        panic!("a default policy should always have a root node")
    };
    nodes[0].to_owned()
}

impl CombinedPolicyState {
    /// Resolve all the default policies from the config into materialized policies.
    /// This takes into account the hierarchy of the default policies.
    fn expand_default_policies(&mut self, jetty: &Jetty) -> Result<()> {
        let intermediate_map = self.get_prioritized_policies(jetty)?;
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

    /// Remove all non-connector-managed default policies as these are only ephemeral.
    #[allow(clippy::unnecessary_to_owned)]
    fn remove_non_connector_managed_default_policies(&mut self) {
        self.default_policies = self
            .default_policies
            .to_owned()
            .into_iter()
            .filter(|(_, p)| p.connector_managed)
            .collect();
    }

    /// Expand default policies into a map of <priority, combined policy state>
    fn get_prioritized_policies(
        &self,
        jetty: &Jetty,
    ) -> Result<HashMap<String, CombinedPolicyState>> {
        let ag = jetty.try_access_graph()?;

        // prioritize default policies
        let mut prioritized_policies = HashMap::new();

        #[allow(clippy::unnecessary_to_owned)]
        for (k, v) in self.default_policies.to_owned() {
            let asset_path = match &k.0 {
                NodeName::Asset { path, .. } => path,
                _ => bail!("expected an asset node"),
            };
            let wildcard_path = k.1.to_owned();
            prioritized_policies
                .entry(get_path_priority(wildcard_path, asset_path.to_owned()))
                .and_modify(
                    |combined_state: &mut HashMap<
                        (NodeName, String, AssetType, NodeName),
                        DefaultPolicyState,
                    >| {
                        combined_state.insert(k.to_owned(), v.to_owned());
                    },
                )
                .or_insert_with(|| HashMap::from([(k.to_owned(), v.to_owned())]));
        }

        // This intermediate map holds all of the regular policies created by the default policies,
        // sorted by default policies' priority levels.
        // Note: They are merged within each priority level so that there is just one for each asset <-> user combination, combining policies and metadata if they already exist.
        let mut intermediate_map = HashMap::new();
        for (priority, default_policies) in prioritized_policies {
            intermediate_map.insert(
                priority.to_owned(),
                CombinedPolicyState {
                    ..Default::default()
                },
            );

            for ((root_node, matching_path, target_type, grantee), default_policy_state) in
                default_policies
            {
                let targets = ag.default_policy_targets(&NodeName::DefaultPolicy {
                    root_node: Box::new(root_node.to_owned()),
                    matching_path: matching_path.to_owned(),
                    target_type: target_type.to_owned(),
                    grantee: Box::new(grantee.to_owned()),
                })?;

                let policy_state = PolicyState {
                    privileges: default_policy_state.privileges.iter().cloned().collect(),
                    // FUTURE: for now, just leaving this blank. I think we'll need a mechanism to specify policy-level metadata on a default policy
                    metadata: Default::default(),
                };

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
        Ok(intermediate_map)
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
        // Merge the default policies
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
    let segments = wildcard_path.split('/').collect::<Vec<_>>();
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

/// Parse the configuration files into a node id -> filepath map
pub(crate) fn generate_id_file_map(paths: Paths) -> Result<HashMap<Uuid, PathBuf>> {
    let mut res = HashMap::new();
    for path in paths {
        let path = path?;
        let yaml = std::fs::read_to_string(&path)?;

        let YamlAssetDoc {
            identifier: YamlAssetIdentifier { id: asset_id, .. },
            ..
        } = parser::simple_parse(&yaml).context(format!(
            "unable to generate asset -> file map; problem with configuration file: {}",
            path.to_string_lossy()
        ))?;
        res.insert(Uuid::parse_str(asset_id.as_str())?, path);
    }
    Ok(res)
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
