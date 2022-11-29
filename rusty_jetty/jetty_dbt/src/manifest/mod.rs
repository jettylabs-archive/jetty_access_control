mod filtered_asset;
mod ingestion;
pub(crate) mod node;
mod to_asset_type;

use jetty_core::cual::Cual;
use node::DbtNode;

use anyhow::{bail, Result};
use mockall::automock;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

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
