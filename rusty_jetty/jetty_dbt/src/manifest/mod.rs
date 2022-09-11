pub(crate) mod node;
mod to_asset_type;

use jetty_core::cual::Cual;
use node::{DbtModelNode, DbtNode, DbtSourceNode};
use to_asset_type::ToAssetType;

use anyhow::{bail, Context, Result};
use mockall::automock;
use serde::Deserialize;
use std::f32::consts::E;
use std::fs::read_to_string;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::cual::cual_from_dbt_node;

pub(crate) type DbtNodeName = String;

/// Trait to make mocking behavior easier.
#[automock]
pub(crate) trait DbtProjectManifest {
    fn init(&mut self, file_path: &Option<PathBuf>) -> Result<()>;
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
        #[derive(Deserialize)]
        struct Config {
            enabled: bool,
            database: Option<String>,
            schema: Option<String>,
            // TODO: Use this for asset type determination
            materialized: String,
        }

        #[derive(Deserialize)]
        struct DbtManifestNode {
            resource_type: String,
            config: Config,
        }

        #[derive(Deserialize, Debug)]
        struct DbtManifestSourceNode {
            database: String,
            schema: String,
            unique_id: String,
        }

        #[derive(Deserialize)]
        struct DbtManifestJson {
            nodes: HashMap<String, DbtManifestNode>,
            sources: HashMap<String, DbtManifestSourceNode>,
            child_map: HashMap<String, HashSet<String>>,
        }

        // Initialization only happens once.
        if self.initialized {
            return Ok(());
        }

        let manifest_path = file_path.clone().unwrap_or_else(|| self.path());

        let file_contents =
            read_to_string(&manifest_path).context(format!("reading file {:?}", file_path))?;
        let json_manifest: DbtManifestJson = serde_json::from_str(&file_contents).context(
            format!("deserializing manifest json from {:?}", manifest_path),
        )?;
        // First we will ingest the nodes.
        for (node_name, node) in json_manifest.nodes {
            let asset_type = node.resource_type.try_to_asset_type()?;
            if let Some(ty) = asset_type {
                self.nodes.insert(
                    node_name.to_owned(),
                    DbtNode::ModelNode(DbtModelNode {
                        name: node_name.to_owned(),
                        enabled: node.config.enabled.to_owned(),
                        database: node.config.database.to_owned().unwrap_or_default(),
                        schema: node.config.schema.to_owned().unwrap_or_default(),
                        materialized_as: ty,
                    }),
                );
            } else {
                // Asset type not usable.
                println!("asset type for {:?} not found", node.resource_type);
                continue;
            }
        }
        // Now we'll ingest sources.
        for (_source_name, source) in json_manifest.sources {
            self.nodes.insert(
                source.unique_id.to_owned(),
                DbtNode::SourceNode(DbtSourceNode {
                    name: source.unique_id.to_owned(),
                    database: source.database,
                    schema: source.schema,
                }),
            );
        }
        // Now we'll record the dependencies between nodes.
        for (name, new_deps) in json_manifest.child_map {
            // We only ignore test nodes right now.
            if name.starts_with("test") {
                continue;
            }
            let new_deps = new_deps
                .iter()
                .cloned()
                // Filter out test nodes.
                .filter(|d| !d.starts_with("test"))
                .collect();
            println!("assigning node {:?} deps {:?}", name, new_deps);
            if let Some(deps) = self.dependencies.get_mut(&name) {
                // Combine the new deps with the existing ones.
                *deps = deps.union(&new_deps).map(|i| i.to_owned()).collect();
            } else {
                // Model not yet in map. Add it.
                self.dependencies.insert(name, new_deps);
            }
        }

        self.initialized = true;
        Ok(())
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
            Ok(cual_from_dbt_node(node))
        } else {
            bail!("couldn't get node for name {}", node_name);
        }
    }
}
