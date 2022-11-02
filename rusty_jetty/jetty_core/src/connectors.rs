//! Connectors module.
//!

pub mod nodes;
pub mod processed_nodes;

use std::path::{Path, PathBuf};

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    connectors::nodes::ConnectorData,
    jetty::{ConnectorConfig, CredentialsMap},
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
