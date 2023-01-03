use serde::Deserialize;

use jetty_core::connectors::AssetType;
use jetty_core::cual::Cual;
use jetty_core::{connectors::nodes::RawAssetReference as JettyAssetReference, cual::Cualable};

use std::collections::{HashMap, HashSet};

use super::DbtProjectManifest;

use crate::cual::{cual, get_cual_account_name};

pub(crate) trait NamePartable {
    // Get the relation name for the object.
    fn name(&self) -> &str;
    /// Get the parts of the name in a format eligible for making a CUAL.
    fn name_parts(&self) -> Vec<String> {
        self.name()
            .split('.')
            .map(|p| {
                if p.starts_with(r#"\""#) {
                    // Remove the quotes and return the contained part as-is.
                    p.trim_start_matches(r#"\""#)
                        .trim_end_matches(r#"\""#)
                        .to_owned()
                } else {
                    // Not quoted – we can just capitalize it (only for
                    // Snowflake).
                    p.to_uppercase()
                }
            })
            .collect()
    }

    // Get the cual based on the parts of the relational name.
    fn dbt_cual(&self) -> Cual {
        let name_parts = self.name_parts();
        match name_parts.len() {
            1 => cual!(name_parts[0]),
            2 => cual!(name_parts[0], name_parts[1]),
            3 => cual!(name_parts[0], name_parts[1], name_parts[2]),
            num => panic!("{num} name parts is too many for a dbt CUAL"),
        }
    }
}

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
}

/// A node within Dbt that represents a model.
#[derive(Clone, PartialEq, Eq, Hash, Default)]
pub(crate) struct DbtModelNode {
    pub(crate) name: String,
    pub(crate) enabled: bool,
    pub(crate) materialized_as: AssetType,
}

impl NamePartable for DbtNode {
    fn name(&self) -> &str {
        match self {
            Self::ModelNode(DbtModelNode { name, .. }) => name,
            Self::SourceNode(DbtSourceNode { name, .. }) => name,
        }
    }
}

impl DbtNode {
    #[allow(clippy::borrowed_box)]
    pub(crate) fn to_jetty_asset(
        &self,
        manifest: &Box<dyn DbtProjectManifest + Send + Sync>,
    ) -> JettyAssetReference {
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
                JettyAssetReference::new(
                    (m_node as &dyn NamePartable).cual(),
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
                JettyAssetReference::new(
                    (s_node as &dyn NamePartable).cual(),
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

impl NamePartable for DbtModelNode {
    fn name(&self) -> &str {
        &self.name
    }
}

impl NamePartable for DbtSourceNode {
    fn name(&self) -> &str {
        &self.name
    }
}

impl DbtModelNode {
    pub(crate) fn get_metadata(&self) -> HashMap<String, String> {
        HashMap::from([("enabled".to_owned(), self.enabled.to_string())])
    }
}
