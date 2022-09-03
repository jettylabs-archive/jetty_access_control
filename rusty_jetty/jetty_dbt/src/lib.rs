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
        AssetType as JettyAssetType,
        {
            self,
            nodes::{Asset as JettyAsset, ConnectorData},
        },
    },
    jetty::{ConnectorConfig, CredentialsBlob},
    Connector,
};

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use manifest::{DbtManifest, DbtProjectManifest};

/// Main connector struct
/// Used by Jetty to get the data that resides
/// within dbt
pub struct DbtConnector {
    manifest: Box<dyn DbtProjectManifest + Send + Sync>,
}

impl DbtConnector {
    /// Enhanced new method to inject a DbtManifest into the connector.
    fn new_with_manifest(
        config: &ConnectorConfig,
        credentials: &CredentialsBlob,
        client: Option<connectors::ConnectorClient>,
        manifest: impl DbtProjectManifest + Send + Sync + 'static,
    ) -> Result<Box<Self>> {
        Ok(Box::new(DbtConnector {
            manifest: Box::new(manifest),
        }))
    }

    fn get_data_from_manifest(&self, manifeset: &impl DbtProjectManifest) {
        // manifest.get_tables();
    }
}

#[async_trait]
impl Connector for DbtConnector {
    fn new(
        config: &ConnectorConfig,
        credentials: &CredentialsBlob,
        client: Option<connectors::ConnectorClient>,
    ) -> Result<Box<Self>> {
        if !credentials.contains_key("project_dir") {
            bail!("missing project_dir key in connectors.yaml");
        }
        let manifest = DbtManifest::new(&credentials["project_dir"])
            .context("creating dbt manifest object")?;
        Self::new_with_manifest(config, credentials, client, manifest)
    }

    async fn check(&self) -> bool {
        // Check that the manifest file exists and is valid json
        true
    }

    async fn get_data(&self) -> ConnectorData {
        let all_nodes_as_assets: Vec<JettyAsset> = self
            .manifest
            .get_nodes()
            .iter()
            .map(|node| {
                let node_dependencies = self.manifest.get_dependencies(&node.name).unwrap();
                let asset = JettyAsset::new(
                    node.name.to_owned(),
                    node.materialized_as,
                    node.get_metadata(),
                    // No policies in dbt.
                    HashSet::new(),
                    // We just put the immediate parent schema here, which will be
                    // resolved by Jetty with other sources
                    HashSet::from([node.get_parent()]),
                    // No children in dbt. Adult only zone.
                    HashSet::new(),
                    // This is the lineage!
                    node_dependencies,
                    // Handled by the lineage children nodes.
                    HashSet::new(),
                    // TODO?
                    HashSet::new(),
                );
                asset
            })
            .collect();
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
    use super::*;
    use manifest::MockDbtProjectManifest;
    use std::collections::HashSet;

    #[test]
    fn creating_connector_works() -> Result<()> {
        let manifest_mock = MockDbtProjectManifest::new();
        DbtConnector::new_with_manifest(
            &ConnectorConfig::default(),
            &CredentialsBlob::from([("project_dir".to_owned(), "/not/a/dir".to_owned())]),
            Some(connectors::ConnectorClient::Test),
            manifest_mock,
        )
        .context("creating dbt manifest object in creating_connector_works")?;
        Ok(())
    }

    #[test]
    #[should_panic]
    fn missing_config_fails() {
        let result = DbtConnector::new(
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

        manifest_mock
            .expect_get_nodes()
            .times(1)
            .returning(HashSet::new);
        let connector = DbtConnector::new_with_manifest(
            &ConnectorConfig::default(),
            &CredentialsBlob::from([("project_dir".to_owned(), "/not/a/dir".to_owned())]),
            Some(connectors::ConnectorClient::Test),
            manifest_mock,
        )
        .context("creating connector")?;
        let data = connector.get_data().await;
        assert_eq!(data, ConnectorData::default());
        Ok(())
    }
}
