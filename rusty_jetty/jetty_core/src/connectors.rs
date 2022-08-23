use crate::jetty::{ConnectorConfig, CredentialsBlob};
use anyhow::Result;

pub trait Connector {
    fn new(config: &ConnectorConfig, credentials: &CredentialsBlob) -> Result<Box<Self>>;
    // fn check() -> bool;
}
