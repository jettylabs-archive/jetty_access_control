use std::time::Instant;

use anyhow::{bail, Context, Result};
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
    /// Connectors to collect for.
    #[clap(short, long, default_value = "snowflake,tableau,dbt")]
    connectors: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let jetty = Jetty::new()?;
    let creds = fetch_credentials()?;

    if args.connectors.is_empty() {
        println!("No connectors, huh?");
        bail!("Select a connector");
    }

    let mut data_from_connectors = vec![];

    if args.connectors.contains(&"dbt".to_owned()) {
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

        println!("getting dbt data");
        let now = Instant::now();
        let dbt_data = dbt.get_data().await;
        let dbt_pcd = jetty_core::access_graph::ProcessedConnectorData {
            connector: "dbt".to_owned(),
            data: dbt_data,
        };
        println!("dbt data took {} seconds", now.elapsed().as_secs_f32());
        data_from_connectors.push(dbt_pcd);
    }

    if args.connectors.contains(&"snowflake".to_owned()) {
        println!("intializing snowflake");
        let now = Instant::now();
        let mut snow = jetty_snowflake::SnowflakeConnector::new(
            &jetty.config.connectors[0],
            &creds["snow"],
            Some(ConnectorClient::Core),
        )
        .await?;
        println!("snowflake took {} seconds", now.elapsed().as_secs_f32());

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
        data_from_connectors.push(snow_pcd);
    }

    if args.connectors.contains(&"tableau".to_owned()) {
        println!("initializing tableau");
        let now = Instant::now();
        let mut tab = jetty_tableau::TableauConnector::new(
            &jetty.config.connectors[2],
            &creds["tableau"],
            Some(ConnectorClient::Core),
        )
        .await?;
        println!("tableau took {} seconds", now.elapsed().as_secs_f32());

        println!("getting tableau data");
        let now = Instant::now();
        tab.setup().await?;
        let tab_data = tab.get_data().await;
        let tab_pcd = jetty_core::access_graph::ProcessedConnectorData {
            connector: "tableau".to_owned(),
            data: tab_data,
        };
        println!("tableau data took {} seconds", now.elapsed().as_secs_f32());
        data_from_connectors.push(tab_pcd);
    }

    println!("creating access graph");
    let now = Instant::now();
    let ag = AccessGraph::new(data_from_connectors)?;
    println!(
        "access graph creation took {} seconds",
        now.elapsed().as_secs_f32()
    );

    if args.visualize {
        println!("visualizing access graph");
        let now = Instant::now();
        ag.visualize("/tmp/graph.svg")
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
