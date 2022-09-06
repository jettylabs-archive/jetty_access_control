//! Connector library for dbt!
//!  
//! We use dbt for lineage only right now.
//!
//! That means we get relationships and models
//! from dbt and bind those with the assets declared
//! in other connectors to inform policy and
//! give us table-level lineage-based policy.
//!
#![deny(missing_docs)]

mod manifest;

use std::collections::{HashMap, HashSet};

use jetty_core::{
    connectors::{
        self,
        nodes::{Asset as JettyAsset, ConnectorData},
        AssetType,
    },
    jetty::{ConnectorConfig, CredentialsBlob},
    Connector,
};

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use manifest::{node::DbtNode, DbtManifest, DbtProjectManifest};

/// Main connector struct
/// Used by Jetty to get the data that resides
/// within dbt
pub struct DbtConnector {
    manifest: Box<dyn DbtProjectManifest + Send + Sync>,
}

impl DbtConnector {
    /// Enhanced new method to inject a DbtManifest into the connector.
    fn new_with_manifest(
        manifest: impl DbtProjectManifest + Send + Sync + 'static,
    ) -> Result<Box<Self>> {
        Ok(Box::new(DbtConnector {
            manifest: Box::new(manifest),
        }))
    }
}

#[async_trait]
impl Connector for DbtConnector {
    fn new(
        _config: &ConnectorConfig,
        credentials: &CredentialsBlob,
        _client: Option<connectors::ConnectorClient>,
    ) -> Result<Box<Self>> {
        if !credentials.contains_key("project_dir") {
            bail!("missing project_dir key in connectors.yaml");
        }
        let manifest = DbtManifest::new(&credentials["project_dir"])
            .context("creating dbt manifest object")?;
        Self::new_with_manifest(manifest)
    }

    async fn check(&self) -> bool {
        // Check that the manifest file exists and is valid json
        true
    }

    async fn get_data(&mut self) -> ConnectorData {
        self.manifest.init(&None).unwrap();
        let all_nodes_as_assets: Vec<JettyAsset> = self
            .manifest
            .get_nodes()
            .unwrap()
            .iter()
            .map(|node| {
                match node {
                    DbtNode::ModelNode(m_node) => {
                        let node_dependencies = self
                            .manifest
                            .get_dependencies(&m_node.name)
                            .unwrap()
                            .unwrap_or_default();
                        JettyAsset::new(
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
                            node_dependencies,
                            // TODO?
                            HashSet::new(),
                        )
                    }
                    DbtNode::SourceNode(s_node) => {
                        let node_dependencies = self
                            .manifest
                            .get_dependencies(&s_node.name)
                            .unwrap()
                            .unwrap_or_default();
                        JettyAsset::new(
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
                            node_dependencies,
                            HashSet::new(),
                        )
                    }
                }
            })
            .collect();
        println!("got assets {:#?}", all_nodes_as_assets);
        ConnectorData {
            // No groups in dbt.
            groups: vec![],
            // No users in dbt.
            users: vec![],
            // No policies in dbt.
            policies: vec![],
            assets: all_nodes_as_assets,
            // TODO?
            tags: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::manifest::node::DbtModelNode;

    use super::*;
    use jetty_core::connectors::{nodes::Asset, AssetType};
    use manifest::{node::DbtNode, MockDbtProjectManifest};
    use std::collections::{HashMap, HashSet};

    #[test]
    fn creating_connector_works() -> Result<()> {
        let manifest_mock = MockDbtProjectManifest::new();
        DbtConnector::new_with_manifest(manifest_mock)
            .context("creating dbt manifest object in creating_connector_works")?;
        Ok(())
    }

    #[test]
    #[should_panic]
    fn missing_config_fails() {
        DbtConnector::new(
            &ConnectorConfig::default(),
            &CredentialsBlob::new(),
            Some(connectors::ConnectorClient::Test),
        )
        .unwrap();
    }

    #[test]
    fn check_with_no_manifest_fails() {
        // Mock manifest file
        let manifest_mock = MockDbtProjectManifest::new();

        // panic!();
    }

    #[tokio::test]
    async fn get_data_returns_empty() -> Result<()> {
        // Create mocked manifest
        let mut manifest_mock = MockDbtProjectManifest::new();

        manifest_mock.expect_init().times(1).returning(|_| Ok(()));
        manifest_mock
            .expect_get_nodes()
            .times(1)
            .returning(|| Ok(HashSet::new()));

        let mut connector =
            DbtConnector::new_with_manifest(manifest_mock).context("creating connector")?;
        let data = connector.get_data().await;
        assert_eq!(data, ConnectorData::default());
        Ok(())
    }

    #[tokio::test]
    async fn get_data_returns_valid_assets() -> Result<()> {
        // Create mocked manifest
        let mut manifest_mock = MockDbtProjectManifest::new();

        manifest_mock.expect_init().times(1).returning(|_| Ok(()));
        manifest_mock
            .expect_get_dependencies()
            .times(1)
            .returning(|_| Ok(None));
        manifest_mock
            .expect_get_nodes()
            .times(1)
            .returning(|| Ok(HashSet::from([DbtNode::ModelNode(DbtModelNode::default())])));
        let mut connector =
            DbtConnector::new_with_manifest(manifest_mock).context("creating connector")?;

        let data = connector.get_data().await;
        assert_eq!(
            data,
            ConnectorData {
                assets: vec![Asset {
                    name: "".to_owned(),
                    asset_type: AssetType::Other,
                    metadata: HashMap::from([("enabled".to_owned(), "false".to_owned())]),
                    governed_by: HashSet::new(),
                    child_of: HashSet::from([".".to_owned()]),
                    parent_of: HashSet::new(),
                    derived_from: HashSet::new(),
                    derived_to: HashSet::new(),
                    tagged_as: HashSet::new()
                }],
                ..Default::default()
            }
        );
        Ok(())
    }
}
