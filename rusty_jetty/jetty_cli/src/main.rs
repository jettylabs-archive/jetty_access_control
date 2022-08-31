use anyhow::Result;
use jetty_core::{
    access_graph::AccessGraph, connectors::ConnectorClient, fetch_credentials, Connector, Jetty,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let jetty = Jetty::new()?;
    // println!("{:#?}", jetty.config);
    let creds = fetch_credentials()?;
    let snow = jetty_snowflake::SnowflakeConnector::new(
        &jetty.config.connectors[0],
        &creds["snow"],
        Some(ConnectorClient::Core),
    )?;
    println!("checking for connection...");
    println!("working? {}", snow.check().await);

    let snow_data = snow.get_data().await;
    println!("{:#?}", snow_data);
    let pcd = jetty_core::access_graph::ProcessedConnectorData {
        connector: "Snowflake".to_owned(),
        data: snow_data,
    };
    AccessGraph::new(vec![pcd])?;
    // let ag = AccessGraph::new(vec![pcd])?;
    // let res = ag
    //     .graph
    //     .visualize("/tmp/graph.svg".to_owned())
    //     .context("failed to visualize")?;
    // println!("{}", res);

    Ok(())
}
