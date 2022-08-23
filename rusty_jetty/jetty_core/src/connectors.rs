use anyhow::Result;
use async_trait::async_trait;

use crate::jetty::{ConnectorConfig, CredentialsBlob};

#[async_trait]
pub trait Connector {
    fn new(config: &ConnectorConfig, credentials: &CredentialsBlob) -> Result<Box<Self>>;
    async fn check(&self) -> bool;
}
