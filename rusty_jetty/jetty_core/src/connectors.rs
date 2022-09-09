//! Connectors module.
//!

pub mod nodes;

use anyhow::Result;
use async_trait::async_trait;

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
    fn new(
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

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum UserIdentifier {
    /// User's first name
    FirstName,
    /// User's last name
    LastName,
    /// User's full name
    FullName,
    /// User's email address
    Email,
}

/// Enum of known asset types

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetType {
    /// Database Table
    DBTable,
    /// Database View
    DBView,
    /// Database Schema
    DBSchema,
    /// Database
    DBDB,
    /// Database Warehouse
    DBWarehouse,
    /// A catch-all that can be used by connector implementors
    #[default]
    Other,
}
