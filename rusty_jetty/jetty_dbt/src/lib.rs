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

mod consts;
mod cual;
mod manifest;

use std::{collections::HashMap, path::Path};

use cual::set_cual_account_name;
use jetty_core::{
    connectors::{
        self,
        nodes::{ConnectorData, RawAssetReference as JettyAssetReference},
    },
    jetty::{ConnectorConfig, CredentialsMap},
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
        manifest: impl DbtProjectManifest + Send + Sync + 'static,
    ) -> Result<Box<Self>> {
        Ok(Box::new(DbtConnector {
            manifest: Box::new(manifest),
        }))
    }
}

#[async_trait]
impl Connector for DbtConnector {
    async fn new(
        _config: &ConnectorConfig,
        credentials: &CredentialsMap,
        _client: Option<connectors::ConnectorClient>,
    ) -> Result<Box<Self>> {
        if !credentials.contains_key("project_dir") {
            bail!("missing project_dir key in connectors.yaml");
        }
        if !credentials.contains_key("snowflake_account") {
            bail!("missing `snowflake_account` dbt configuration (connectors.yaml)");
        }
        set_cual_account_name(&credentials["snowflake_account"]);
        let manifest = DbtManifest::new(&credentials["project_dir"])
            .context("creating dbt manifest object")?;
        Self::new_with_manifest(manifest)
    }

    async fn check(&self) -> bool {
        // Check that the manifest file exists and is valid json
        let project_dir = &self.manifest.get_project_dir();
        let project_path = Path::new(project_dir);
        let is_file = project_path.is_file();
        let f = std::fs::File::open(project_path);
        if f.is_err() {
            // Problem reading the file
            return false;
        }
        let reader = std::io::BufReader::new(f.unwrap());
        let valid_json = serde_json::from_reader::<_, serde_json::Value>(reader).is_ok();
        is_file && valid_json
    }

    async fn get_data(&mut self) -> ConnectorData {
        self.manifest.init(&None).unwrap();
        let all_nodes_as_assets: Vec<JettyAssetReference> = self
            .manifest
            .get_nodes()
            .unwrap()
            .values()
            .map(|node| node.to_jetty_asset(&self.manifest))
            .collect();
        ConnectorData {
            // No groups in dbt.
            groups: vec![],
            // No users in dbt.
            users: vec![],
            // No policies in dbt.
            policies: vec![],
            assets: vec![],
            asset_references: all_nodes_as_assets,
            // TODO?
            tags: vec![],
            effective_permissions: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        consts::VIEW,
        manifest::node::{DbtModelNode, DbtSourceNode},
    };

    use super::*;
    use jetty_core::{
        connectors::{nodes::RawAsset, AssetType, ConnectorClient},
        cual::Cual,
    };
    use manifest::{node::DbtNode, MockDbtProjectManifest};
    use std::collections::{HashMap, HashSet};

    #[test]
    fn creating_connector_works() -> Result<()> {
        let manifest_mock = MockDbtProjectManifest::new();
        DbtConnector::new_with_manifest(manifest_mock)
            .context("creating dbt manifest object in creating_connector_works")?;
        Ok(())
    }

    #[tokio::test]
    #[should_panic]
    async fn missing_config_fails() {
        DbtConnector::new(
            &ConnectorConfig::default(),
            &CredentialsMap::new(),
            Some(connectors::ConnectorClient::Test),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn check_with_no_manifest_fails() {
        let connector = DbtConnector::new(
            &ConnectorConfig::default(),
            &HashMap::from([("project_dir".to_owned(), "something/not/a/path".to_owned())]),
            Some(ConnectorClient::Test),
        )
        .await;
        assert!(connector.is_err());
    }

    #[tokio::test]
    async fn get_data_returns_empty() -> Result<()> {
        set_cual_account_name("account");
        // Create mocked manifest
        let mut manifest_mock = MockDbtProjectManifest::new();

        manifest_mock.expect_init().times(1).returning(|_| Ok(()));
        manifest_mock
            .expect_get_nodes()
            .times(1)
            .returning(|| Ok(HashMap::new()));

        let mut connector =
            DbtConnector::new_with_manifest(manifest_mock).context("creating connector")?;
        let data = connector.get_data().await;
        assert_eq!(data, ConnectorData::default());
        Ok(())
    }

    #[tokio::test]
    async fn get_data_returns_valid_dbt_assets() -> Result<()> {
        set_cual_account_name("account");
        // Create mocked manifest
        let mut manifest_mock = MockDbtProjectManifest::new();

        manifest_mock.expect_init().times(1).returning(|_| Ok(()));
        manifest_mock
            .expect_get_dependencies()
            .times(1)
            .returning(|_| Ok(None));
        manifest_mock.expect_get_nodes().times(1).returning(|| {
            Ok(HashMap::from([(
                "".to_owned(),
                DbtNode::ModelNode(DbtModelNode {
                    materialized_as: AssetType(VIEW.to_owned()),
                    name: "db.schema.model".to_owned(),
                    ..Default::default()
                }),
            )]))
        });
        let mut connector =
            DbtConnector::new_with_manifest(manifest_mock).context("creating connector")?;

        let data = connector.get_data().await;
        assert_eq!(
            data,
            ConnectorData {
                asset_references: vec![JettyAssetReference {
                    cual: Cual::new("snowflake://account.snowflakecomputing.com/DB/SCHEMA/MODEL"),
                    metadata: HashMap::from([("enabled".to_owned(), "false".to_owned())]),
                    governed_by: HashSet::new(),
                    child_of: HashSet::new(),
                    parent_of: HashSet::new(),
                    derived_from: HashSet::new(),
                    derived_to: HashSet::new(),
                    tagged_as: HashSet::new(),
                }],
                ..Default::default()
            }
        );
        Ok(())
    }

    #[tokio::test]
    async fn get_data_returns_valid_dbt_assets_and_lineage() -> Result<()> {
        // Create mocked manifest
        let mut manifest_mock = MockDbtProjectManifest::new();

        manifest_mock.expect_init().times(1).returning(|_| Ok(()));
        manifest_mock
            .expect_get_dependencies()
            .times(3)
            .returning(|_| Ok(Some(HashSet::from(["test".to_owned()]))));
        manifest_mock
            .expect_cual_for_node()
            .times(3)
            .returning(|_| Ok(Cual::new("cual://a")));
        manifest_mock.expect_get_nodes().times(1).returning(|| {
            Ok(HashMap::from([
                (
                    "".to_owned(),
                    DbtNode::ModelNode(DbtModelNode {
                        materialized_as: AssetType(VIEW.to_owned()),
                        ..Default::default()
                    }),
                ),
                (
                    "test".to_owned(),
                    DbtNode::ModelNode(DbtModelNode {
                        materialized_as: AssetType(VIEW.to_owned()),
                        name: "test".to_owned(),
                        ..Default::default()
                    }),
                ),
                (
                    "test2".to_owned(),
                    DbtNode::SourceNode(DbtSourceNode {
                        name: "test2".to_owned(),
                        ..Default::default()
                    }),
                ),
            ]))
        });
        let mut connector =
            DbtConnector::new_with_manifest(manifest_mock).context("creating connector")?;

        let mut assets = connector.get_data().await.assets;
        dbg!(&assets);
        assert_eq!(
            assets.sort(),
            vec![
                RawAsset {
                    cual: Cual::new("snowflake:////"),
                    name: "".to_owned(),
                    asset_type: AssetType(VIEW.to_owned()),
                    metadata: HashMap::from([("enabled".to_owned(), "false".to_owned())]),
                    governed_by: HashSet::new(),
                    child_of: HashSet::new(),
                    parent_of: HashSet::new(),
                    derived_from: HashSet::new(),
                    derived_to: HashSet::from(["".to_owned()]),
                    tagged_as: HashSet::new()
                },
                RawAsset {
                    cual: Cual::new("snowflake:////test"),
                    name: "test".to_owned(),
                    asset_type: AssetType(VIEW.to_owned()),
                    metadata: HashMap::from([("enabled".to_owned(), "false".to_owned())]),
                    governed_by: HashSet::new(),
                    child_of: HashSet::new(),
                    parent_of: HashSet::new(),
                    derived_from: HashSet::new(),
                    derived_to: HashSet::from(["".to_owned()]),
                    tagged_as: HashSet::new()
                },
                RawAsset {
                    cual: Cual::new("snowflake://test2db/test2schema/test2"),
                    name: "test2".to_owned(),
                    asset_type: AssetType(VIEW.to_owned()),
                    metadata: HashMap::from([("enabled".to_owned(), "false".to_owned())]),
                    governed_by: HashSet::new(),
                    child_of: HashSet::new(),
                    parent_of: HashSet::new(),
                    derived_from: HashSet::new(),
                    derived_to: HashSet::from(["".to_owned()]),
                    tagged_as: HashSet::new()
                },
            ]
            .sort(),
        );
        Ok(())
    }
}
