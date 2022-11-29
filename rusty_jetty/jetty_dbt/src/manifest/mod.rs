pub(crate) mod node;
mod to_asset_type;

use jetty_core::cual::{Cual, Cualable};
use node::{DbtModelNode, DbtNode, DbtSourceNode};
use to_asset_type::ToAssetType;

use anyhow::{bail, Context, Result};
use mockall::automock;
use serde::Deserialize;
use std::fs::read_to_string;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};


use self::node::NamePartable;

pub(crate) type DbtNodeName = String;

/// Trait to make mocking behavior easier.
#[automock]
pub(crate) trait DbtProjectManifest {
    fn init(&mut self, file_path: &Option<PathBuf>) -> Result<()>;
    fn get_project_dir(&self) -> String;
    /// List all nodes
    fn get_nodes(&self) -> Result<HashMap<String, DbtNode>>;
    /// List all nodes that depend on the given node.
    fn get_dependencies(&self, node_name: &str) -> Result<Option<HashSet<String>>>;
    /// Get the CUAL for a given node name.
    fn cual_for_node(&self, node_name: DbtNodeName) -> Result<Cual>;
}

#[derive(Default)]
pub(crate) struct DbtManifest {
    initialized: bool,
    project_dir: String,
    /// All models
    nodes: HashMap<String, DbtNode>,
    /// Map of model relationships from node name to childrens' names
    dependencies: HashMap<String, HashSet<String>>,
}

impl DbtManifest {
    pub(crate) fn new(project_dir: &str) -> Result<Self> {
        Ok(DbtManifest {
            project_dir: project_dir.to_owned(),
            ..Default::default()
        })
    }

    #[inline(always)]
    fn path(&self) -> PathBuf {
        Path::new(&self.project_dir).join(Path::new("target/manifest.json"))
    }

    #[inline(always)]
    fn check_initialized(&self) -> Result<()> {
        if !self.initialized {
            bail!("manifest was not initialized")
        }
        Ok(())
    }
}

impl DbtProjectManifest for DbtManifest {
    fn init(&mut self, file_path: &Option<PathBuf>) -> Result<()> {
        #[derive(Deserialize, Debug)]
        struct Config {
            enabled: bool,
            materialized: String,
        }

        #[derive(Deserialize, Debug)]
        struct DbtManifestNode {
            /// Used by Jetty only for ephemeral nodes.
            unique_id: String,
            relation_name: Option<String>,
            resource_type: String,
            config: Config,
        }

        #[derive(Deserialize, Debug)]
        struct DbtManifestSourceNode {
            relation_name: Option<String>,
        }

        #[derive(Deserialize, Debug)]
        struct DbtManifestJson {
            nodes: HashMap<String, DbtManifestNode>,
            sources: HashMap<String, DbtManifestSourceNode>,
            child_map: HashMap<String, HashSet<String>>,
        }

        #[derive(Hash, PartialEq, Eq, Clone, Debug)]
        enum RelationName {
            NodeRelationName(String),
            EphemeralNodeWithoutRelationName(String),
        }
        impl RelationName {
            fn inner(&self) -> &str {
                match self {
                    RelationName::NodeRelationName(name) => name,
                    RelationName::EphemeralNodeWithoutRelationName(_name) => panic!("not today sir"),
                }
            }
        }

        /// Get the relation name of the given node from the manifest, whether
        /// it's a source node or a model node.
        fn get_node_relation_name_from_mani(
            manifest: &DbtManifestJson,
            name: &str,
        ) -> RelationName {
            if name.starts_with("source") {
                RelationName::NodeRelationName(
                    manifest
                        .sources
                        .get(name)
                        .unwrap()
                        .relation_name
                        .as_ref()
                        .unwrap()
                        .to_owned(),
                )
            } else {
                let node = manifest.nodes.get(name).unwrap();
                node.relation_name
                    .as_ref()
                    .map(|rn| RelationName::NodeRelationName(rn.clone()))
                    .or_else(|| {
                        // No relation name, this is likely an ephemeral node
                        if node.config.materialized == "ephemeral" {
                            Some(RelationName::EphemeralNodeWithoutRelationName(
                                node.unique_id.clone(),
                            ))
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| panic!("trying to get {name} from {:#?}", &manifest.nodes))
            }
        }

        // Initialization only happens once.
        if self.initialized {
            return Ok(());
        }

        let manifest_path = file_path.clone().unwrap_or_else(|| self.path());

        let file_contents =
            read_to_string(&manifest_path).context(format!("reading file {file_path:?}"))?;
        let json_manifest: DbtManifestJson = serde_json::from_str(&file_contents).context(
            format!("deserializing manifest json from {manifest_path:?}"),
        )?;
        // First we will ingest the nodes.
        for node in json_manifest.nodes.values() {
            let asset_type = node.resource_type.try_to_asset_type()?;
            if let Some(ty) = asset_type {
                if node.config.materialized != "ephemeral" {
                    self.nodes.insert(
                        node.relation_name.clone().unwrap().to_owned(),
                        DbtNode::ModelNode(DbtModelNode {
                            // All model nodes should have relation names
                            name: node.relation_name.as_ref().unwrap().to_owned(),
                            enabled: node.config.enabled.to_owned(),
                            materialized_as: ty,
                        }),
                    );
                }
            } else {
                // Asset type not usable.
                continue;
            }
        }
        // Now we'll ingest sources.
        for (_source_name, source) in &json_manifest.sources {
            self.nodes.insert(
                source.relation_name.clone().unwrap().to_owned(),
                DbtNode::SourceNode(DbtSourceNode {
                    // All  source nodes should have relation names
                    name: source.relation_name.as_ref().unwrap().to_owned(),
                }),
            );
        }
        // Now we'll record the dependencies between nodes.
        // First we'll transform the child map ("source.x.y" -> "model.x.y")
        // to a relation child map ("db.schema.table" -> "db.schema.view")
        let mut relation_child_map = json_manifest
            .child_map
            .iter()
            .filter_map(|(name, new_deps)| {
                // Filter test nodes.
                if name.starts_with("test") {
                    None
                } else {
                    let relation_name = get_node_relation_name_from_mani(&json_manifest, name);
                    let new_relation_deps: HashSet<_> = new_deps
                        .iter()
                        .cloned()
                        .filter(|d| !d.starts_with("test"))
                        .map(|dep| get_node_relation_name_from_mani(&json_manifest, &dep))
                        .collect();
                    Some((relation_name, new_relation_deps))
                }
            })
            .collect::<HashMap<_, _>>();

        // Ephemeral nodes don't have permissions associated with them, so we need to
        // remove them and join the dependencies around them.
        // Get all ephemeral relationships
        let rcm_clone = relation_child_map.clone();
        let ephemeral_deps = rcm_clone
            .iter()
            .filter(|(n, _)| {
                // only keep ephemeral nodes
                matches!(n, RelationName::EphemeralNodeWithoutRelationName(_))
            })
            .collect::<HashMap<_, _>>();
        // For each one, connect the parent to the child
        for (ephemeral_dep_name, children) in ephemeral_deps {
            relation_child_map
                .iter_mut()
                .for_each(|(p, parent_children)| {
                    // Remove the connection to the ephemeral node and replace
                    // it with the ephemeral node's children
                    if parent_children.remove(ephemeral_dep_name) {
                        println!("adding {:?} to {:?}", children, p);
                        parent_children.extend(children.clone().into_iter());
                    }
                });
            relation_child_map.remove(ephemeral_dep_name);
        }

        for (name, new_deps) in relation_child_map {
            if let Some(deps) = self.dependencies.get_mut(name.inner()) {
                // Combine the new deps with the existing ones.
                deps.extend(new_deps.iter().map(|d| d.inner().to_owned()));
            } else {
                // Model not yet in dependency map. Add it.
                self.dependencies.insert(
                    name.inner().to_owned(),
                    new_deps.iter().map(|d| d.inner().to_owned()).collect(),
                );
            }
        }

        self.initialized = true;
        Ok(())
    }

    fn get_project_dir(&self) -> String {
        self.project_dir.to_owned()
    }

    fn get_nodes(&self) -> Result<HashMap<String, DbtNode>> {
        self.check_initialized()?;
        Ok(self.nodes.clone())
    }

    fn get_dependencies(&self, node_name: &str) -> Result<Option<HashSet<String>>> {
        self.check_initialized()?;
        Ok(self.dependencies.get(node_name).cloned())
    }

    fn cual_for_node(&self, node_name: DbtNodeName) -> Result<Cual> {
        if let Some(node) = self.nodes.get(&node_name) {
            Ok((node as &dyn NamePartable).cual())
        } else {
            bail!("couldn't get node for name {}", node_name);
        }
    }
}
