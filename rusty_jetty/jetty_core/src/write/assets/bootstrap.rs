//! Bootstrap policies from the generated graph into a yaml file

use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fs,
    path::PathBuf,
};

use anyhow::Result;
use petgraph::stable_graph::NodeIndex;

use crate::{
    access_graph::{
        AssetAttributes, DefaultPolicyAttributes, EdgeType, JettyNode, NodeName, PolicyAttributes,
    },
    project, Jetty,
};

use super::{YamlAssetDoc, YamlAssetIdentifier, YamlDefaultPolicy, YamlPolicy};

struct SimplePolicy {
    privileges: BTreeSet<String>,
    asset: NodeIndex,
    users: HashSet<NodeIndex>,
    groups: HashSet<NodeIndex>,
}

impl Jetty {
    /// Go thorugh the config and return a map of node_index -> (policies, default_policies) that can be written to create the assets directory
    fn build_bootstrapped_policy_config(
        &self,
    ) -> Result<HashMap<NodeIndex, (BTreeSet<YamlPolicy>, BTreeSet<YamlDefaultPolicy>)>> {
        let ag = self.try_access_graph()?;

        let mut all_basic_policies = Vec::new();

        let policies = &ag.graph.nodes.policies;
        for (_, idx) in policies {
            let attributes = PolicyAttributes::try_from(ag[*idx].clone())?;
            let privileges = attributes.privileges.to_owned();
            let target_assets = ag.get_matching_children(
                *idx,
                |e| matches!(e, EdgeType::Governs),
                |_| false,
                |n| matches!(n, JettyNode::Asset(_)),
                Some(1),
                Some(1),
            );
            let target_users = ag.get_matching_children(
                *idx,
                |e| matches!(e, EdgeType::GrantedTo),
                |_| false,
                |n| matches!(n, JettyNode::User(_)),
                Some(1),
                Some(1),
            );
            let target_groups = ag.get_matching_children(
                *idx,
                |e| matches!(e, EdgeType::GrantedTo),
                |_| false,
                |n| matches!(n, JettyNode::Group(_)),
                Some(1),
                Some(1),
            );

            for asset in target_assets {
                all_basic_policies.push(SimplePolicy {
                    privileges: privileges.clone().into_iter().collect(),
                    asset,
                    users: target_users.iter().copied().collect(),
                    groups: target_groups.iter().copied().collect(),
                })
            }
        }

        // We'll assume that all privileges for a given user/group were already combined.

        // Fold based on the name of the asset and the privileges -> If they are equal, create
        // a single entry - this means multiple groups/users
        let basic_policies = all_basic_policies.into_iter().fold(
            HashMap::new(),
            |mut acc: HashMap<(NodeIndex, BTreeSet<String>), HashSet<NodeIndex>>, x| {
                acc.entry((x.asset.to_owned(), x.privileges.to_owned()))
                    .and_modify(|f| {
                        f.extend(&x.groups);
                        f.extend(&x.users);
                    })
                    .or_insert({
                        let mut z = x.groups.to_owned();
                        z.extend(x.users);
                        z
                    });
                acc
            },
        );
        // Now push the policies to the results
        let mut res = HashMap::new();

        for ((idx, privileges), grantees) in basic_policies {
            let _base_node = AssetAttributes::try_from(ag[idx].to_owned())?;
            let mut users = BTreeSet::new();
            let mut groups = BTreeSet::new();
            for g in grantees {
                match &ag[g] {
                    JettyNode::Group(attr) => groups.insert(attr.name.to_string()),
                    JettyNode::User(attr) => users.insert(attr.name.to_string()),
                    _ => panic!("wrong node type as grantee: expected user or group"),
                };
            }

            let yaml_policy = YamlPolicy {
                privileges: if privileges.is_empty() {
                    None
                } else {
                    Some(privileges)
                },
                users: if users.is_empty() { None } else { Some(users) },
                groups: if groups.is_empty() {
                    None
                } else {
                    Some(groups)
                },
                ..Default::default()
            };

            res.entry(idx)
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

        // Pull in default policies. Not really going to clean these up, for now at least.
        let default_policies = &ag.graph.nodes.default_policies;
        // Push the future policies to the results
        for (_, &idx) in default_policies {
            let default_policy = DefaultPolicyAttributes::try_from(ag[idx].to_owned())?;
            // There should only be one base_node
            let binding = ag.get_matching_children(
                idx,
                |e| matches!(e, EdgeType::ProvidedDefaultForChildren),
                |_| false,
                |n| matches!(n, JettyNode::Asset(_)),
                Some(1),
                Some(1),
            );
            let &base_idx = binding.first().unwrap();

            let grantees = ag.get_matching_children(
                idx,
                |e| matches!(e, EdgeType::GrantedTo),
                |_| false,
                |n| matches!(n, JettyNode::Group(_)) || matches!(n, JettyNode::User(_)),
                Some(1),
                Some(1),
            );

            let mut users = BTreeSet::new();
            let mut groups = BTreeSet::new();

            for g in grantees {
                match &ag[g] {
                    JettyNode::Group(attr) => groups.insert(attr.name.to_string()),
                    JettyNode::User(attr) => users.insert(attr.name.to_string()),
                    _ => panic!("wrong node type as grantee: expected user or group"),
                };
            }

            let yaml_default_policy = YamlDefaultPolicy {
                privileges: if default_policy.privileges.is_empty() {
                    None
                } else {
                    Some(default_policy.privileges.to_owned().into_iter().collect())
                },
                users: if users.is_empty() { None } else { Some(users) },
                groups: if groups.is_empty() {
                    None
                } else {
                    Some(groups)
                },
                path: default_policy.matching_path.to_owned(),
                types: default_policy.types.to_owned(),
                ..Default::default()
            };

            res.entry(base_idx)
                .and_modify(
                    |(_policies, default_policies): &mut (
                        BTreeSet<YamlPolicy>,
                        BTreeSet<YamlDefaultPolicy>,
                    )| {
                        default_policies.insert(yaml_default_policy.to_owned());
                    },
                )
                .or_insert(([].into(), [yaml_default_policy].into()));
        }

        Ok(res)
    }

    /// Generate the yaml configs for each asset, as well as the proper path for them in the file system
    pub fn generate_bootstrapped_policy_yaml(&self) -> Result<HashMap<PathBuf, String>> {
        let ag = self.try_access_graph()?;
        self.build_bootstrapped_policy_config()?
            .into_iter()
            .map(
                |(idx, (policies, default_policies))| -> Result<(PathBuf, String)> {
                    let node_name = ag[idx].get_node_name();
                    match node_name {
                        NodeName::Asset {
                            connector,
                            asset_type,
                            path,
                        } => Ok((
                            self.asset_index_to_file_path(idx),
                            yaml_peg::serde::to_string(&YamlAssetDoc {
                                identifier: YamlAssetIdentifier {
                                    name: path.to_string(),
                                    asset_type,
                                    connector,
                                },
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
        let binding = ag.get_matching_children(
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
                    Some(t) => format!("@{}", t.0),
                    None => "".to_string(),
                }
            )),
            _ => panic!("expected an asset nodename"),
        }
    }
}

/// This cleans out illegal characters so that asset names can be used in paths
fn clean_string_for_path(val: String) -> String {
    // Remove illegal characters
    let no_stinky_chars = val
        .split(
            &[
                '/', '\\', '?', '|', '<', '>', ':', '*', '"', '+', ',', ';', '=', '[', ']',
            ][..],
        )
        .collect::<Vec<_>>()
        .join("_");
    // Can't end in a period
    if no_stinky_chars.ends_with('.') {
        format!("{no_stinky_chars}_")
    } else {
        no_stinky_chars
    }
}

/// Write the output of generate_bootstrapped_policy_yaml to the proper directories
pub fn write_bootstrapped_asset_yaml(assets: HashMap<PathBuf, String>) -> Result<()> {
    for (path, policy_doc) in assets {
        let parent_path = project::assets_cfg_root_path().join(path);
        // make sure the parent directories exist
        fs::create_dir_all(&parent_path)?;
        fs::write(parent_path.join(project::assets_cfg_filename()), policy_doc)?;
    }
    Ok(())
}
