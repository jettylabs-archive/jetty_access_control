//! Bootstrap policies from the generated graph into a yaml file

use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fs,
    path::PathBuf,
};

use anyhow::{anyhow, bail, Context, Result};
use petgraph::stable_graph::NodeIndex;

use crate::{
    access_graph::{
        AssetAttributes, DefaultPolicyAttributes, EdgeType, JettyNode, NodeName, PolicyAttributes,
    },
    connectors::AssetType,
    project,
    write::utils::clean_string_for_path,
    Jetty,
};

use super::{
    generate_id_file_map, CombinedPolicyState, DefaultPolicyState, PolicyState, YamlAssetDoc,
    YamlAssetIdentifier, YamlDefaultPolicy, YamlPolicy,
};

type PolicyKey = (NodeName, BTreeSet<String>, BTreeSet<(String, String)>);

impl Jetty {
    /// Collect the policies in the graph and return a map of <(asset name, agent name), SimplePolicy>
    fn build_simple_policy_map(&self) -> Result<HashMap<(NodeName, NodeName), PolicyState>> {
        let ag = self.try_access_graph()?;

        let mut all_basic_policies = HashMap::new();

        // FUTURE: this should be updated so that each policy has a single grantee and a single asset
        let policies = &ag.graph.nodes.policies;
        for idx in policies.values() {
            let attributes = PolicyAttributes::try_from(ag[*idx].clone())?;
            let target_assets = ag.get_matching_descendants(
                *idx,
                |e| matches!(e, EdgeType::Governs),
                |_| false,
                |n| matches!(n, JettyNode::Asset(_)),
                Some(1),
                Some(1),
            );
            let target_users = ag.get_matching_descendants(
                *idx,
                |e| matches!(e, EdgeType::GrantedTo),
                |_| false,
                |n| matches!(n, JettyNode::User(_)),
                Some(1),
                Some(1),
            );
            let target_groups = ag.get_matching_descendants(
                *idx,
                |e| matches!(e, EdgeType::GrantedTo),
                |_| false,
                |n| matches!(n, JettyNode::Group(_)),
                Some(1),
                Some(1),
            );

            let grantees = [target_groups, target_users].concat();

            let simple_policy = PolicyState {
                privileges: attributes.privileges.iter().cloned().collect(),
                metadata: Default::default(),
            };

            for asset in target_assets {
                for &grantee in &grantees {
                    all_basic_policies.insert(
                        (ag[asset].get_node_name(), ag[grantee].get_node_name()),
                        simple_policy.to_owned(),
                    );
                }
            }
        }
        Ok(all_basic_policies)
    }

    /// Collect all the default policies from the graph and return a map of <(Asset, path, types, grantee), DefaultPolicyState>
    #[allow(clippy::type_complexity)]
    fn build_default_policy_map(
        &self,
    ) -> Result<HashMap<(NodeName, String, AssetType, NodeName), DefaultPolicyState>> {
        let ag = self.try_access_graph()?;
        // The contract for getting default policies from connectors will require that it be one agent <-> asset (including root node, path, and type) per policy.
        let default_policies = &ag.graph.nodes.default_policies;

        let mut res = HashMap::new();
        // Push the future policies to the results
        for &idx in default_policies.values() {
            let attributes = DefaultPolicyAttributes::try_from(ag[idx].to_owned())?;
            // There should only be one base_node
            let binding = ag.get_matching_descendants(
                idx,
                |e| matches!(e, EdgeType::ProvidedDefaultForChildren),
                |_| false,
                |n| matches!(n, JettyNode::Asset(_)),
                Some(1),
                Some(1),
            );
            let &base_idx = binding.first().unwrap();

            let grantees = ag.get_matching_descendants(
                idx,
                |e| matches!(e, EdgeType::GrantedTo),
                |_| false,
                |n| matches!(n, JettyNode::Group(_)) || matches!(n, JettyNode::User(_)),
                Some(1),
                Some(1),
            );
            for grantee in grantees {
                res.insert(
                    (
                        ag[base_idx].get_node_name(),
                        attributes.matching_path.to_owned(),
                        attributes.target_type.to_owned(),
                        ag[grantee].get_node_name(),
                    ),
                    DefaultPolicyState {
                        privileges: attributes.privileges.iter().cloned().collect(),
                        metadata: attributes.metadata.to_owned(),
                        connector_managed: true,
                    },
                );
            }
        }
        Ok(res)
    }

    /// Go thorugh the config and return a map of node_index (asset) -> (policies, default_policies) that can be written to create the assets directory
    #[allow(clippy::type_complexity)]
    fn build_bootstrapped_policy_config(
        &self,
    ) -> Result<HashMap<NodeIndex, (BTreeSet<YamlPolicy>, BTreeSet<YamlDefaultPolicy>)>> {
        let mut basic_policies = self.build_simple_policy_map()?;
        let default_policies = self.build_default_policy_map()?;

        // now expand the default policies to make a more compact representation of the policies
        let default_policy_state = CombinedPolicyState {
            // the following function doesn't actually use the policies part, so I don't need to pass it in
            policies: Default::default(),
            default_policies: default_policies.to_owned(),
        };

        let expanded_default_policies = default_policy_state.get_prioritized_policies(self)?;

        // now iterate through the expanded default policies from highest priority to lowest and use them to compress the basic policies
        let mut priority_levels = expanded_default_policies
            .keys()
            .map(|k| k.to_owned())
            .collect::<Vec<_>>();
        priority_levels.sort();

        let mut removal_list = HashSet::new();
        for priority in priority_levels.into_iter().rev() {
            removal_list.extend(compact_regular_policies(
                &mut basic_policies,
                &expanded_default_policies[&priority],
            ));
        }
        // now remove redundant policies
        for policy in removal_list {
            basic_policies.remove(&policy);
        }

        // At this point, we basic_policies contains all the non-default policies to bootstrap, and default_policies contains all the defaults. We're ready to turn
        // them into yaml structs

        // Now fold and add the policies to the output map
        let mut res = HashMap::new();
        self.fold_and_build_yaml_policies(basic_policies, &mut res)?;
        self.fold_and_build_yaml_default_policies(default_policies, &mut res)?;

        Ok(res)
    }

    fn fold_and_build_yaml_policies(
        &self,
        basic_policies: HashMap<(NodeName, NodeName), PolicyState>,
        policy_map: &mut HashMap<NodeIndex, (BTreeSet<YamlPolicy>, BTreeSet<YamlDefaultPolicy>)>,
    ) -> Result<()> {
        let ag = self.try_access_graph()?;
        let folded_policies = basic_policies.into_iter().fold(
            HashMap::new(),
            // acc is HashMap of <(Asset, Privileges, Metadata), Set<grantees>>
            |mut acc: HashMap<PolicyKey, HashSet<NodeName>>,
             ((policy_asset, policy_grantee), policy_state)| {
                acc.entry((
                    policy_asset,
                    policy_state.privileges.into_iter().collect(),
                    policy_state.metadata.into_iter().collect(),
                ))
                .and_modify(|f| {
                    f.insert(policy_grantee.to_owned());
                })
                .or_insert_with(|| [policy_grantee.to_owned()].into());
                acc
            },
        );

        for ((asset, privileges, metadata), grantees) in &folded_policies {
            // split users and groups
            let mut users = BTreeSet::new();
            let mut groups = BTreeSet::new();
            for g in grantees {
                match &ag[ag
                    .get_untyped_index_from_name(g)
                    .ok_or_else(|| anyhow!("unable to find grantee node in graph"))?]
                {
                    JettyNode::Group(attr) => groups.insert(attr.name.to_string()),
                    JettyNode::User(attr) => users.insert(attr.name.to_string()),
                    _ => bail!("wrong node type as grantee: expected user or group"),
                };
            }

            let yaml_policy = YamlPolicy {
                privileges: if privileges.is_empty() {
                    None
                } else {
                    Some(privileges.to_owned())
                },
                users: if users.is_empty() { None } else { Some(users) },
                groups: if groups.is_empty() {
                    None
                } else {
                    Some(groups)
                },
                description: Default::default(),
                metadata: if metadata.is_empty() {
                    None
                } else {
                    Some(
                        #[allow(clippy::unnecessary_to_owned)]
                        metadata.to_owned().into_iter().collect(),
                    )
                },
            };

            policy_map
                .entry(
                    ag.get_untyped_index_from_name(asset)
                        .ok_or_else(|| anyhow!("unable to find asset node for policy"))?,
                )
                .and_modify(
                    |(policies, _default_policies): &mut (
                        BTreeSet<YamlPolicy>,
                        BTreeSet<YamlDefaultPolicy>,
                    )| {
                        policies.insert(yaml_policy.to_owned());
                    },
                )
                .or_insert(([yaml_policy].into(), [].into()));
        }
        Ok(())
    }

    fn fold_and_build_yaml_default_policies(
        &self,
        default_policies: HashMap<(NodeName, String, AssetType, NodeName), DefaultPolicyState>,
        policy_map: &mut HashMap<NodeIndex, (BTreeSet<YamlPolicy>, BTreeSet<YamlDefaultPolicy>)>,
    ) -> Result<()> {
        let ag = self.try_access_graph()?;
        let folded_default_policies = default_policies.into_iter().fold(
            HashMap::new(),
            // acc is HashMap of <(Asset, Path, Types, Privileges, Metadata), Set<grantees>>
            #[allow(clippy::type_complexity)]
            |mut acc: HashMap<
                (
                    NodeName,
                    String,
                    AssetType,
                    BTreeSet<String>,
                    BTreeSet<(String, String)>,
                ),
                HashSet<NodeName>,
            >,
             ((policy_asset, policy_path, asset_type, policy_grantee), policy_state)| {
                acc.entry((
                    policy_asset,
                    policy_path,
                    asset_type,
                    policy_state.privileges.to_owned(),
                    policy_state.metadata.into_iter().collect(),
                ))
                .and_modify(|f| {
                    f.insert(policy_grantee.to_owned());
                })
                .or_insert_with(|| [policy_grantee.to_owned()].into());
                acc
            },
        );

        // Now push the policies to the results

        for ((asset, path, types, privileges, metadata), grantees) in &folded_default_policies {
            // split users and groups
            let mut users = BTreeSet::new();
            let mut groups = BTreeSet::new();
            for g in grantees {
                match &ag[ag
                    .get_untyped_index_from_name(g)
                    .ok_or_else(|| anyhow!("unable to find grantee node in graph"))?]
                {
                    JettyNode::Group(attr) => groups.insert(attr.name.to_string()),
                    JettyNode::User(attr) => users.insert(attr.name.to_string()),
                    _ => bail!("wrong node type as grantee: expected user or group"),
                };
            }

            let yaml_policy = YamlDefaultPolicy {
                privileges: if privileges.is_empty() {
                    None
                } else {
                    Some(privileges.to_owned())
                },
                users: if users.is_empty() { None } else { Some(users) },
                groups: if groups.is_empty() {
                    None
                } else {
                    Some(groups)
                },
                description: Default::default(),
                metadata: if metadata.is_empty() {
                    None
                } else {
                    Some(metadata.iter().cloned().collect())
                },
                path: path.to_owned(),
                target_type: types.to_owned(),
                // only connector-managed defaults should be present when bootstrapping
                connector_managed: true,
            };

            policy_map
                .entry(
                    ag.get_untyped_index_from_name(asset)
                        .ok_or_else(|| anyhow!("unable to find asset node for policy"))?,
                )
                .and_modify(
                    |(_policies, default_policies): &mut (
                        BTreeSet<YamlPolicy>,
                        BTreeSet<YamlDefaultPolicy>,
                    )| {
                        default_policies.insert(yaml_policy.to_owned());
                    },
                )
                .or_insert(([].into(), [yaml_policy].into()));
        }
        Ok(())
    }

    /// Generate the yaml configs for each asset, as well as the proper path for them in the file system
    pub fn generate_bootstrapped_policy_yaml(&self) -> Result<HashMap<PathBuf, String>> {
        let ag = self.try_access_graph()?;
        self.build_bootstrapped_policy_config()?
            .into_iter()
            .map(
                |(idx, (policies, default_policies))| -> Result<(PathBuf, String)> {
                    let attributes: AssetAttributes = ag[idx].to_owned().try_into()?;
                    let node_name = attributes.name();
                    match node_name {
                        NodeName::Asset { .. } => Ok((
                            self.asset_index_to_file_path(idx),
                            yaml_peg::serde::to_string(&YamlAssetDoc {
                                identifier: asset_attributes_to_yaml_identifier(&attributes),
                                policies,
                                default_policies,
                            })?,
                        )),
                        _ => panic!("expected an asset node"),
                    }
                },
            )
            .collect()
    }

    /// given an asset index, get the directory path to its directory. THis doesn't include project directory prefix or the actual filename
    fn asset_index_to_file_path(&self, idx: NodeIndex) -> PathBuf {
        let reverse_path = self.get_reverse_path(idx);
        reverse_path.iter().rev().collect()
    }

    /// Return the file path (in reverse) for a given asset
    fn get_reverse_path(&self, idx: NodeIndex) -> Vec<String> {
        let mut parts = Vec::new();
        let ag = self.try_access_graph().unwrap();

        // add self
        parts.push(ag[idx].get_node_name().to_path_part());

        // extend with parent. There should only be one parent
        let binding = ag.get_matching_descendants(
            idx,
            |e| matches!(e, EdgeType::ChildOf),
            |_| false,
            |n| matches!(n, JettyNode::Asset(_)),
            Some(1),
            Some(1),
        );
        let parent = binding.first();

        match parent {
            Some(&p) => {
                parts.extend(self.get_reverse_path(p));
                parts
            }
            None => {
                // If this is the highest parent in the hierarchy, add the namespace
                parts.push(
                    ag[idx]
                        .get_node_connectors()
                        .iter()
                        .next()
                        .unwrap()
                        .to_string(),
                );
                parts
            }
        }
    }
}

impl NodeName {
    /// Given an asset node name, get the part of the path that will represent that asset in the hierarchy
    fn to_path_part(&self) -> String {
        match &self {
            NodeName::Asset {
                connector: _,
                asset_type,
                path,
            } => clean_string_for_path(format!(
                "{}{}",
                path.components()[path.components().len() - 1],
                match asset_type {
                    Some(t) => format!(" ({})", t.0),
                    None => "".to_string(),
                }
            )),
            _ => panic!("expected an asset nodename"),
        }
    }
}

/// Write the output of generate_bootstrapped_policy_yaml to the proper directories
pub fn write_bootstrapped_asset_yaml(assets: HashMap<PathBuf, String>) -> Result<()> {
    for (path, policy_doc) in assets {
        let parent_path = project::assets_cfg_root_path_local().join(path);
        // make sure the parent directories exist
        fs::create_dir_all(&parent_path)?;
        fs::write(
            &parent_path.join(format!(
                "{}.yaml",
                parent_path.file_name().unwrap().to_string_lossy()
            )),
            policy_doc,
        )?;
    }
    Ok(())
}

/// merge a CombinedPolicyState struct into self, compacting for default policies.
///  - If there's an existing policy that exactly matches the existing policy, add the existing one to the removal list
///  - If there's a policy that doesn't match, just skip and move on
///  - If there's not a matching policy, create a new one with no privileges and no metadata
/// return the removal list so that unneeded polices can be removed
fn compact_regular_policies(
    existing_policies: &mut HashMap<(NodeName, NodeName), PolicyState>,
    CombinedPolicyState {
        policies: expanded_default_policies,
        ..
    }: &CombinedPolicyState,
) -> HashSet<(NodeName, NodeName)> {
    let mut removal_list = HashSet::new();
    for (other_k, other_v) in expanded_default_policies {
        let entry = existing_policies.get(other_k).cloned();
        match entry {
            Some(p) => {
                // if the policy matches a default policy, drop it
                if p == *other_v {
                    removal_list.insert(other_k.to_owned());
                }
                // if it doesn't match a default policy, leave it as is
                else {
                    continue;
                };
            }
            // If there's no matching policy, add an empty one so that it won't be overwritten by the default
            None => {
                existing_policies.insert(other_k.to_owned(), Default::default());
            }
        };
    }
    // at the end, remove all the unneeded policies
    removal_list
}

/// Add files (without policies or default policies) for all assets in the graph.
pub fn update_asset_files(jetty: &Jetty) -> Result<()> {
    let ag = jetty.try_access_graph()?;
    // Start by collecting all the assets that are in the config
    let paths = super::get_config_paths()?;
    let id_file_map = generate_id_file_map(paths)?;
    let config_id_set: HashSet<uuid::Uuid> = id_file_map.keys().cloned().collect();

    // Get the assets that don't exist in config, and create files for them
    let env_ids: HashSet<uuid::Uuid> = ag.graph.node_ids.assets.keys().cloned().collect();
    for id in env_ids.difference(&config_id_set) {
        let idx = ag
            .get_asset_index_from_id(id)
            .to_owned()
            .ok_or_else(|| anyhow!("unable to get asset by id"))?;
        let attributes: AssetAttributes = ag[idx].to_owned().try_into()?;
        let policy_doc = YamlAssetDoc {
            identifier: asset_attributes_to_yaml_identifier(&attributes),
            policies: Default::default(),
            default_policies: Default::default(),
        };
        let yaml = yaml_peg::serde::to_string(&policy_doc)?;
        let parent_path =
            project::assets_cfg_root_path_local().join(jetty.asset_index_to_file_path(idx.into()));
        // make sure the parent directories exist
        fs::create_dir_all(&parent_path)?;
        fs::write(
            &parent_path.join(format!(
                "{}.yaml",
                parent_path.file_name().unwrap().to_string_lossy()
            )),
            yaml,
        )?;
    }

    // Get the assets that don't exist in the env and delete files.
    // They should be using version control, so this shouldn't be a big deal.
    for id in config_id_set.difference(&env_ids) {
        let x = &id_file_map[id];
        fs::remove_file(x).context("removing nonexistent assets from config")?;
    }

    // Delete any empty folders
    let asset_directories = glob::glob(
        format!(
            "{}/**/",
            project::assets_cfg_root_path_local().to_string_lossy()
        )
        .as_str(),
    )
    .context("trouble generating config directory paths")?;
    for dir in asset_directories {
        fs::remove_dir(dir?).ok();
    }

    Ok(())
}

fn asset_attributes_to_yaml_identifier(attributes: &AssetAttributes) -> YamlAssetIdentifier {
    if let NodeName::Asset {
        connector,
        asset_type,
        path,
    } = attributes.name()
    {
        YamlAssetIdentifier {
            name: path.to_string(),
            asset_type: asset_type.to_owned(),
            connector: connector.to_owned(),
            id: attributes.id.to_string(),
        }
    } else {
        panic!("wrong node name type for an asset node")
    }
}
