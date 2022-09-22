use serde::Deserialize;

use jetty_core::connectors::AssetType;
use jetty_core::{connectors::nodes::Asset as JettyAsset, cual::Cualable};

use std::collections::{HashMap, HashSet};

use super::DbtProjectManifest;

/// A node within Dbt, representing either a model
/// or a source.
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) enum DbtNode {
    ModelNode(DbtModelNode),
    SourceNode(DbtSourceNode),
}

/// A node within Dbt that represents a data source.
#[derive(Default, Clone, Deserialize, PartialEq, Eq, Hash)]
pub(crate) struct DbtSourceNode {
    pub(crate) name: String,
    pub(crate) database: String,
    pub(crate) schema: String,
}

/// A node within Dbt that represents a model.
#[derive(Clone, PartialEq, Eq, Hash, Default)]
pub(crate) struct DbtModelNode {
    pub(crate) name: String,
    pub(crate) enabled: bool,
    pub(crate) database: String,
    pub(crate) schema: String,
    pub(crate) materialized_as: AssetType,
}

impl DbtNode {
    pub(crate) fn to_jetty_asset(
        &self,
        manifest: &Box<dyn DbtProjectManifest + Send + Sync>,
    ) -> JettyAsset {
        match self {
            Self::ModelNode(m_node) => {
                let node_dependencies = manifest
                    .get_dependencies(&m_node.name)
                    .unwrap()
                    .unwrap_or_default();
                let dependency_cuals = node_dependencies
                    .iter()
                    .map(|dep_name| manifest.cual_for_node(dep_name.to_owned()).unwrap().uri())
                    .collect();
                JettyAsset::new(
                    m_node.cual(),
                    m_node.name.to_owned(),
                    m_node.materialized_as,
                    m_node.get_metadata(),
                    // No policies in dbt.
                    HashSet::new(),
                    // We won't put the schema here, since it originates in Snowflake.
                    HashSet::new(),
                    // No children in dbt. Adult only zone.
                    HashSet::new(),
                    // Handled by the lineage derived_to nodes.
                    HashSet::new(),
                    // This is the lineage!
                    dependency_cuals,
                    // TODO?
                    HashSet::new(),
                )
            }
            DbtNode::SourceNode(s_node) => {
                let node_dependencies = manifest
                    .get_dependencies(&s_node.name)
                    .unwrap()
                    .unwrap_or_default();
                let dependency_cuals = node_dependencies
                    .iter()
                    .map(|dep_name| manifest.cual_for_node(dep_name.to_owned()).unwrap().uri())
                    .collect();
                JettyAsset::new(
                    s_node.cual(),
                    s_node.name.to_owned(),
                    AssetType::DBTable,
                    HashMap::new(),
                    // No policies in dbt.
                    HashSet::new(),
                    // We won't put the schema here, since it originates in Snowflake.
                    HashSet::new(),
                    // No children in dbt. Adult only zone.
                    HashSet::new(),
                    // No lineage parents here since this is a source.
                    HashSet::new(),
                    // Models derived from this source.
                    dependency_cuals,
                    HashSet::new(),
                )
            }
        }
    }
}
impl DbtModelNode {
    pub(crate) fn get_metadata(&self) -> HashMap<String, String> {
        HashMap::from([("enabled".to_owned(), self.enabled.to_string())])
    }
}
