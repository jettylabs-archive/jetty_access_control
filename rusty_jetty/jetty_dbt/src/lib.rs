use jetty_core::{
    connectors::nodes::ConnectorData,
    jetty::{ConnectorConfig, CredentialsBlob},
    Connector,
};

use anyhow::Result;
use async_trait::async_trait;

struct DbtConnector {}

impl DbtConnector {}

#[async_trait]
impl Connector for DbtConnector {
    fn new(
        config: &ConnectorConfig,
        credentials: &CredentialsBlob,
        client: Option<jetty_core::connectors::ConnectorClient>,
    ) -> Result<Box<Self>> {
        Ok(Box::new(DbtConnector {}))
    }

    async fn check(&self) -> bool {
        true
    }

    fn get_data<'life0, 'async_trait>(
        &'life0 self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = ConnectorData> + core::marker::Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
