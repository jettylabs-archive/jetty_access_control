use anyhow::Result;
use async_trait::async_trait;

use crate::jetty::{ConnectorConfig, CredentialsBlob};

/// The trait all connectors are expected to implement.
#[async_trait]
pub trait Connector {
    /// Instantiate a Connector from configuration.
    fn new(config: &ConnectorConfig, credentials: &CredentialsBlob) -> Result<Box<Self>>;
    /// Check if the Connector is properly set up and return the connection 
    /// status (true for connected, false for not).
    async fn check(&self) -> bool;
}
