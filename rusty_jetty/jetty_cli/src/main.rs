use anyhow::Result;
use jetty_core::{
    access_graph::AccessGraph, connectors::ConnectorClient, fetch_credentials, Connector, Jetty,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let jetty = Jetty::new()?;
    let creds = fetch_credentials()?;
    let snow = jetty_snowflake::SnowflakeConnector::new(
        &jetty.config.connectors[0],
        &creds["snow"],
        Some(ConnectorClient::Core),
    )?;
    println!("checking for connection...");
    println!("working? {}", snow.check().await);

    let mut dbt = jetty_dbt::DbtConnector::new(
        &jetty.config.connectors[1],
        &creds["dbt"],
        Some(ConnectorClient::Core),
    )?;
    let dbt_data = dbt.get_data().await;
    println!("dbt data: {:#?}", dbt_data);
    let pcd = jetty_core::access_graph::ProcessedConnectorData {
        connector: "dbt".to_owned(),
        data: dbt_data,
    };
    AccessGraph::new(vec![pcd])?;

    Ok(())
}
