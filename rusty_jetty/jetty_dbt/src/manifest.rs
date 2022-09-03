use anyhow::{Context, Result};
use jetty_core::connectors::AssetType;
use mockall::automock;
use serde::Deserialize;
use std::fs::read_to_string;

use std::collections::{HashMap, HashSet};
use std::path::Path;

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct DbtNode {
    pub(crate) name: String,
    pub(crate) enabled: bool,
    pub(crate) database: String,
    pub(crate) schema: String,
    pub(crate) materialized_as: AssetType,
}

impl DbtNode {
    pub(crate) fn get_metadata(&self) -> HashMap<String, String> {
        HashMap::from([("enabled".to_owned(), self.enabled.to_string())])
    }

    pub(crate) fn get_parent(&self) -> String {
        format!("{}.{}", self.database, self.schema)
    }
}

/// Trait to make mocking behavior easier.
#[automock]
pub(crate) trait DbtProjectManifest {
    fn init(&mut self, file_path: &Path) -> Result<()>;
    /// List all nodes
    fn get_nodes(&self) -> HashSet<DbtNode>;
    /// List all nodes that the given node depends on.
    fn get_dependencies(&self, node_name: &str) -> Option<HashSet<String>>;
}

#[derive(Default)]
pub(crate) struct DbtManifest {
    // /Users/jk/jetty/vendor/jaffle_shop/target/manifest.json
    /// All models
    nodes: HashSet<DbtNode>,
    /// Map of model relationships from node name to dependents' names
    dependencies: HashMap<String, HashSet<String>>,
}

impl DbtManifest {
    pub(crate) fn new(project_dir: &str) -> Result<Self> {
        let mut manifest = DbtManifest::default();
        manifest
            .init(&Path::new(project_dir).join(Path::new("target/manifest.json")))
            .context("initializing manifest")?;
        Ok(manifest)
    }
}

impl DbtProjectManifest for DbtManifest {
    fn init(&mut self, file_path: &Path) -> Result<()> {
        #[derive(Deserialize)]
        struct DependsOn {
            nodes: HashSet<String>,
        }

        #[derive(Deserialize)]
        struct Config {
            enabled: bool,
            database: Option<String>,
            schema: Option<String>,
            materialized: String,
        }

        #[derive(Deserialize)]
        struct DbtManifestNode {
            depends_on: DependsOn,
            config: Config,
        }

        #[derive(Deserialize)]
        struct DbtManifestJson {
            nodes: HashMap<String, DbtManifestNode>,
        }

        let contents =
            read_to_string(file_path).context(format!("reading file {:?}", file_path))?;
        let json_manifest: DbtManifestJson =
            serde_json::from_str(&contents).context("deserializing json")?;
        for (node_name, node) in json_manifest.nodes {
            let asset_type = match node.config.materialized.as_str() {
                "view" => AssetType::DBView,
                "table" => AssetType::DBTable,
                // TODO figure out what we want to do with seeds
                "seed" => AssetType::Other,
                "test" => AssetType::Other,
                x => {
                    println!("unexpected asset type {:?}", x);
                    AssetType::Other
                }
            };
            self.nodes.insert(DbtNode {
                name: node_name.to_owned(),
                enabled: node.config.enabled.to_owned(),
                database: node.config.database.to_owned().unwrap_or_default(),
                schema: node.config.schema.to_owned().unwrap_or_default(),
                materialized_as: asset_type,
            });
            if let Some(depended_nodes) = self.dependencies.get_mut(&node_name.to_owned()) {
                // Node already in the map. Add its dependents.
                *depended_nodes = HashSet::from_iter(
                    depended_nodes
                        .union(&node.depends_on.nodes)
                        .map(|s| s.to_owned()),
                );
            } else {
                // Model not yet in map. Add it.
                self.dependencies.insert(node_name, node.depends_on.nodes);
            }
        }
        Ok(())
    }

    fn get_nodes(&self) -> HashSet<DbtNode> {
        self.nodes.clone()
    }

    fn get_dependencies(&self, node_name: &str) -> Option<HashSet<String>> {
        self.dependencies.get(node_name).cloned()
    }
}
