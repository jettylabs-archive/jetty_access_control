//! Connectors module.
//!

pub mod nodes;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    connectors::nodes::ConnectorData,
    jetty::{ConnectorConfig, CredentialsBlob},
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
    /// Instantiate a Connector from configuration.
    async fn new(
        config: &ConnectorConfig,
        credentials: &CredentialsBlob,
        client: Option<ConnectorClient>,
    ) -> Result<Box<Self>>;
    /// Check if the Connector is properly set up and return the connection
    /// status (true for connected, false for not).
    async fn check(&self) -> bool;
    /// Get all data in one container for the connector to supply to the graph.
    async fn get_data(&mut self) -> ConnectorData;
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
    /// Shouldn't be used other than as a default.
    #[default]
    Unknown,
}

/// The kind of asset within a connector
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetType(pub String);
