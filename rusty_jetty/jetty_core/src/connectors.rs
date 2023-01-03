//! Connectors module.
//!

pub mod nodes;
pub mod processed_nodes;

use std::{collections::HashSet, path::PathBuf};

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    access_graph::translate::diffs::LocalConnectorDiffs,
    connectors::nodes::ConnectorData,
    jetty::{ConnectorConfig, ConnectorManifest, CredentialsMap},
};

/// Client using the connector
#[derive(PartialEq, Eq)]
pub enum ConnectorClient {
    /// Automated tests
    Test,
    /// Jetty Core
    Core,
    /// Something else
    Other,
}

/// The trait all connectors are expected to implement.
#[async_trait]
pub trait Connector {
    /// Check if the Connector is properly set up and return the connection
    /// status (true for connected, false for not).
    async fn check(&self) -> bool;
    /// Get all data in one container for the connector to supply to the graph.
    async fn get_data(&mut self) -> ConnectorData;
    /// Get the capabilities of a given connector. These can include
    fn get_manifest(&self) -> ConnectorManifest;
    /// Plan changes, based on a set of diffs. Can have a todo!() implementation if a connector doesn't have
    /// write capabilities
    fn plan_changes(&self, diffs: &LocalConnectorDiffs) -> Vec<String>;
    /// Apply changes, based on a set of diffs. Can have a todo!() implementation if a connector doesn't have
    /// write capabilities
    async fn apply_changes(&self, diffs: &LocalConnectorDiffs) -> Result<String>;
}

/// The trait all connectors are expected to implement.
#[async_trait]
pub trait NewConnector {
    /// Instantiate a Connector from configuration.
    async fn new(
        config: &ConnectorConfig,
        credentials: &CredentialsMap,
        client: Option<ConnectorClient>,
        // A connector is allowed to create and write to this directory.
        data_dir: Option<PathBuf>,
    ) -> Result<Box<Self>>;
}

#[derive(Default, Debug)]
/// The capabilities of a connector
pub struct ConnectorCapabilities {
    /// The write capabilities of the connector. Right now these can include:
    /// groups, policies
    pub write: HashSet<WriteCapabilities>,
    /// The read capabilities of the connector. These could include:
    /// asset_lineage, assets, groups, users, policies
    pub read: HashSet<ReadCapabilities>,
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
/// Available read capabilities for connectors
pub enum ReadCapabilities {
    /// Read asset lineage
    AssetLineage,
    /// Read assets
    Assets,
    /// Read groups
    Groups,
    /// Read users
    Users,
    /// Read policies
    Policies {
        /// Connector support for default/wildcard policies
        default_policies: bool,
    },
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
/// Available write capabilities for connectors
pub enum WriteCapabilities {
    /// Write groups, and whether groups can be nested inside groups
    Groups {
        /// Whether other groups can be nested in groups
        nested: bool,
    },
    /// Write Policies
    Policies {
        /// Connector support for default/wildcard policies
        default_policies: bool,
    },
    /// Add Users
    Users,
}

/// Enum of identifiers used to resolve user identities
#[derive(Debug, Default, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserIdentifier {
    /// User's first name
    FirstName(String),
    /// User's last name
    LastName(String),
    /// User's full name
    FullName(String),
    /// User's email address
    Email(String),
    /// Other identifiers that can be used for matching
    Other(String),
    /// Shouldn't be used other than as a default.
    #[default]
    Unknown,
}

/// The kind of asset within a connector
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct AssetType(pub String);

impl ToString for AssetType {
    fn to_string(&self) -> String {
        self.0.to_owned()
    }
}
