use std::time::Instant;

use anyhow::{Context, Result};
use clap::Parser;

use jetty_core::{
    access_graph::AccessGraph, connectors::ConnectorClient, fetch_credentials, Connector, Jetty,
};

/// Jetty CLI: Open-source data access control for modern teams
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Visualize the graph in an SVG file.
    #[clap(short, long, value_parser, default_value = "false")]
    visualize: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let jetty = Jetty::new()?;
    let creds = fetch_credentials()?;

    println!("initializing dbt");
    let now = Instant::now();
    // Initialize connectors
    let mut dbt = jetty_dbt::DbtConnector::new(
        &jetty.config.connectors[1],
        &creds["dbt"],
        Some(ConnectorClient::Core),
    )
    .await?;
    println!("dbt took {} seconds", now.elapsed().as_secs_f32());

    println!("intializing snowflake");
    let now = Instant::now();
    let mut snow = jetty_snowflake::SnowflakeConnector::new(
        &jetty.config.connectors[0],
        &creds["snow"],
        Some(ConnectorClient::Core),
    )
    .await?;
    println!("snowflake took {} seconds", now.elapsed().as_secs_f32());

    println!("initializing tableau");
    let now = Instant::now();
    let mut tab = jetty_tableau::TableauConnector::new(
        &jetty.config.connectors[2],
        &creds["tableau"],
        Some(ConnectorClient::Core),
    )
    .await?;
    println!("tableau took {} seconds", now.elapsed().as_secs_f32());

    // Collect data from each connector.
    println!("getting dbt data");
    let now = Instant::now();
    let dbt_data = dbt.get_data().await;
    let dbt_pcd = jetty_core::access_graph::ProcessedConnectorData {
        connector: "dbt".to_owned(),
        data: dbt_data,
    };
    println!("dbt data took {} seconds", now.elapsed().as_secs_f32());

    println!("getting snowflake data");
    let now = Instant::now();
    let snow_data = snow.get_data().await;
    let snow_pcd = jetty_core::access_graph::ProcessedConnectorData {
        connector: "snowflake".to_owned(),
        data: snow_data,
    };
    println!(
        "snowflake data took {} seconds",
        now.elapsed().as_secs_f32()
    );

    println!("getting tableau data");
    let now = Instant::now();
    let tab_data = tab.get_data().await;
    let tab_pcd = jetty_core::access_graph::ProcessedConnectorData {
        connector: "tableau".to_owned(),
        data: tab_data,
    };
    println!("tableau data took {} seconds", now.elapsed().as_secs_f32());

    println!("creating access graph");
    let now = Instant::now();
    let ag = AccessGraph::new(vec![dbt_pcd, snow_pcd, tab_pcd])?;
    println!(
        "access graph creation took {} seconds",
        now.elapsed().as_secs_f32()
    );

    if args.visualize {
        println!("visualizing access graph");
        let now = Instant::now();
        ag.graph
            .visualize("/tmp/graph.svg".to_owned())
            .context("failed to visualize")?;
        println!(
            "access graph creation took {} seconds",
            now.elapsed().as_secs_f32()
        );
    } else {
        println!("Skipping visualization.")
    }

    Ok(())
}
