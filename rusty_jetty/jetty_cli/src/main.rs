use anyhow::{Context, Result};
use jetty_core::{
    access_graph::AccessGraph, connectors::ConnectorClient, fetch_credentials, Connector, Jetty,
};

#[tokio::main]
async fn main() -> Result<()> {
    let jetty = Jetty::new()?;
    let creds = fetch_credentials()?;
    println!("checking for connection...");

    // Initialize connectors
    let mut dbt = jetty_dbt::DbtConnector::new(
        &jetty.config.connectors[1],
        &creds["dbt"],
        Some(ConnectorClient::Core),
    )
    .await?;
    let mut snow = jetty_snowflake::SnowflakeConnector::new(
        &jetty.config.connectors[0],
        &creds["snow"],
        Some(ConnectorClient::Core),
    )
    .await?;
    let mut tab = jetty_tableau::TableauConnector::new(
        &jetty.config.connectors[2],
        &creds["tableau"],
        Some(ConnectorClient::Core),
    )
    .await?;

    // Collect data from each connector.
    let dbt_data = dbt.get_data().await;
    println!("dbt data: {:#?}", dbt_data);
    let dbt_pcd = jetty_core::access_graph::ProcessedConnectorData {
        connector: "dbt".to_owned(),
        data: dbt_data,
    };

    let snow_data = snow.get_data().await;
    let snow_pcd = jetty_core::access_graph::ProcessedConnectorData {
        connector: "snowflake".to_owned(),
        data: snow_data,
    };

    let tab_data = tab.get_data().await;
    let tab_pcd = jetty_core::access_graph::ProcessedConnectorData {
        connector: "tableau".to_owned(),
        data: tab_data,
    };

    let ag = AccessGraph::new(vec![dbt_pcd, snow_pcd, tab_pcd])?;
    ag.graph
        .visualize("/tmp/graph.svg".to_owned())
        .context("failed to visualize")?;

    Ok(())
}
