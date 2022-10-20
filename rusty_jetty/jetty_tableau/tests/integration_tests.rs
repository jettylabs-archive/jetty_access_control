use jetty_core::{
    connectors::ConnectorClient,
    fetch_credentials,
    jetty::ConnectorNamespace,
    logging::{info},
    Connector, Jetty,
};

use anyhow::Result;

#[tokio::test]
async fn test_fetch_data_works() -> Result<()> {
    let jetty = Jetty::new()?;
    let creds = fetch_credentials()?;
    let mut tab = jetty_tableau::TableauConnector::new(
        &jetty.config.connectors[&ConnectorNamespace("tableau".to_owned())],
        &creds["tableau"],
        Some(ConnectorClient::Core),
    )
    .await?;

    info!("getting tableau data");
    tab.setup().await?;
    tab.get_data().await;
    Ok(())
}
