use anyhow::{Context, Result};
use jetty_core::{
    access_graph::AccessGraph, connectors::ConnectorClient, fetch_credentials, Connector, Jetty,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let jetty = Jetty::new()?;
    let creds = fetch_credentials()?;
    let mut snow = jetty_snowflake::SnowflakeConnector::new(
        &jetty.config.connectors[0],
        &creds["snow"],
        Some(ConnectorClient::Core),
    )
    .await?;
    println!("checking for connection...");
    println!("working? {}", snow.check().await);

    let mut dbt = jetty_dbt::DbtConnector::new(
        &jetty.config.connectors[1],
        &creds["dbt"],
        Some(ConnectorClient::Core),
    )
    .await?;
    let dbt_data = dbt.get_data().await;
    println!("dbt data: {:#?}", dbt_data);
    let pcd = jetty_core::access_graph::ProcessedConnectorData {
        connector: "dbt".to_owned(),
        data: dbt_data,
    };

    let snow_data = snow.get_data().await;
    let snow_pcd = jetty_core::access_graph::ProcessedConnectorData {
        connector: "snowflake".to_owned(),
        data: snow_data,
    };
    let ag = AccessGraph::new(vec![pcd, snow_pcd])?;
    ag.graph
        .visualize("/tmp/graph.svg".to_owned())
        .context("failed to visualize")?;

    Ok(())
}
