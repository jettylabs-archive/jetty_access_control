use std::path::{Path, PathBuf};

use dirs::home_dir;
use jetty_core::{
    connectors::{ConnectorClient, NewConnector},
    fetch_credentials,
    jetty::ConnectorNamespace,
    logging::info,
    Connector, Jetty,
};

use anyhow::Result;

#[tokio::test]
async fn test_fetch_data_works() -> Result<()> {
    let jetty = Jetty::new("jetty_config.yaml", Path::new("data").into())?;
    let creds = fetch_credentials(home_dir().unwrap().join(".jetty/connectors.yaml"))?;
    let mut tab = jetty_tableau::TableauConnector::new(
        &jetty.config.connectors[&ConnectorNamespace("tableau".to_owned())],
        &creds["tableau"],
        Some(ConnectorClient::Core),
        None,
    )
    .await?;

    info!("getting tableau data");
    tab.setup().await?;
    tab.get_data().await;
    Ok(())
}
