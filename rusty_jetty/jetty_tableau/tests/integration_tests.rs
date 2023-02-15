use std::path::Path;

use dirs::home_dir;
use jetty_core::{
    connectors::{ConnectorClient, NewConnector},
    fetch_credentials,
    jetty::ConnectorNamespace,
    logging::info,
    Connector, Jetty,
};

use anyhow::Result;
use jetty_tableau::TableauConnector;

async fn basic_tableau_connector() -> Result<Box<TableauConnector>> {
    let jetty = Jetty::new(
        "jetty_config.yaml",
        Path::new("data").into(),
        Default::default(),
    )?;
    let creds = fetch_credentials(home_dir().unwrap().join(".jetty/connectors.yaml"))?;

    jetty_tableau::TableauConnector::new(
        &jetty.config.connectors[&ConnectorNamespace("tableau".to_owned())],
        &creds["tableau"],
        Some(ConnectorClient::Core),
        None,
    )
    .await
}

#[tokio::test]
async fn test_fetch_data_works() -> Result<()> {
    let mut tab = basic_tableau_connector().await?;

    info!("getting tableau data");
    tab.setup().await?;
    tab.get_data().await;
    Ok(())
}
