//! Jetty CLI
//!

#![deny(missing_docs)]

use std::{path::Path, sync::Arc, time::Instant};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};

use jetty_core::{
    access_graph::AccessGraph,
    connectors::ConnectorClient,
    fetch_credentials,
    jetty::ConnectorNamespace,
    logging::{self, error, info, warn, LevelFilter},
    Connector, Jetty,
};

const TAGS_PATH: &str = "tags.yaml";

/// Jetty CLI: Open-source data access control for modern teams
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: JettyCommand,
    #[clap(short, long)]
    log_level: Option<LevelFilter>,
}

#[derive(Subcommand, Debug)]
enum JettyCommand {
    Fetch {
        /// Visualize the graph in an SVG file.
        #[clap(short, long, value_parser, default_value = "false")]
        visualize: bool,
        /// Connectors to collect for.
        #[clap(short, long, use_value_delimiter=true, value_delimiter=',', default_values_t = vec!["snowflake".to_owned(),"tableau".to_owned(),"dbt".to_owned()])]
        connectors: Vec<String>,
    },
    Explore,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    logging::setup(args.log_level);

    match &args.command {
        JettyCommand::Fetch {
            visualize,
            connectors,
        } => {
            fetch(connectors, visualize).await?;
        }

        JettyCommand::Explore => match AccessGraph::deserialize_graph() {
            Ok(mut ag) => {
                if Path::new(TAGS_PATH).exists() {
                    let tag_config = std::fs::read_to_string("tags.yaml");
                    match tag_config {
                        Ok(c) => {
                            ag.add_tags(&c)?;
                        }
                        Err(e) => {
                            bail!("found, but was unable to read {}\nerror: {}", TAGS_PATH, e)
                        }
                    }
                }

                jetty_explore::explore_web_ui(Arc::new(ag)).await;
            }
            Err(e) => info!(
                "Unable to find saved graph. Try running `jetty fetch`\nError: {}",
                e
            ),
        },
    }

    Ok(())
}

async fn fetch(connectors: &Vec<String>, visualize: &bool) -> Result<()> {
    let jetty = Jetty::new()?;
    let creds = fetch_credentials()?;

    if connectors.is_empty() {
        warn!("No connectors, huh?");
        bail!("Select a connector");
    }

    let mut data_from_connectors = vec![];

    if connectors.contains(&"dbt".to_owned()) {
        info!("initializing dbt");
        let now = Instant::now();
        // Initialize connectors
        let mut dbt = jetty_dbt::DbtConnector::new(
            &jetty.config.connectors[&ConnectorNamespace("dbt".to_owned())],
            &creds["dbt"],
            Some(ConnectorClient::Core),
        )
        .await?;
        info!("dbt took {} seconds", now.elapsed().as_secs_f32());

        info!("getting dbt data");
        let now = Instant::now();
        let dbt_data = dbt.get_data().await;
        let dbt_pcd = jetty_core::access_graph::ProcessedConnectorData {
            connector: "dbt".to_owned(),
            data: dbt_data,
        };
        info!("dbt data took {} seconds", now.elapsed().as_secs_f32());
        data_from_connectors.push(dbt_pcd);
    }

    if connectors.contains(&"snowflake".to_owned()) {
        info!("intializing snowflake");
        let now = Instant::now();
        let mut snow = jetty_snowflake::SnowflakeConnector::new(
            &jetty.config.connectors[&ConnectorNamespace("snow".to_owned())],
            &creds["snow"],
            Some(ConnectorClient::Core),
        )
        .await?;
        info!("snowflake took {} seconds", now.elapsed().as_secs_f32());

        info!("getting snowflake data");
        let now = Instant::now();
        let snow_data = snow.get_data().await;
        let snow_pcd = jetty_core::access_graph::ProcessedConnectorData {
            connector: "snowflake".to_owned(),
            data: snow_data,
        };
        info!(
            "snowflake data took {} seconds",
            now.elapsed().as_secs_f32()
        );
        data_from_connectors.push(snow_pcd);
    }

    if connectors.contains(&"tableau".to_owned()) {
        info!("initializing tableau");
        let now = Instant::now();
        let mut tab = jetty_tableau::TableauConnector::new(
            &jetty.config.connectors[&ConnectorNamespace("tableau".to_owned())],
            &creds["tableau"],
            Some(ConnectorClient::Core),
        )
        .await?;
        info!("tableau took {} seconds", now.elapsed().as_secs_f32());

        info!("getting tableau data");
        let now = Instant::now();
        tab.setup().await?;
        let tab_data = tab.get_data().await;
        let tab_pcd = jetty_core::access_graph::ProcessedConnectorData {
            connector: "tableau".to_owned(),
            data: tab_data,
        };
        info!("tableau data took {} seconds", now.elapsed().as_secs_f32());
        data_from_connectors.push(tab_pcd);
    }

    info!("creating access graph");
    let now = Instant::now();
    let ag = AccessGraph::new(data_from_connectors)?;
    info!(
        "access graph creation took {} seconds",
        now.elapsed().as_secs_f32()
    );
    ag.serialize_graph()?;

    if *visualize {
        info!("visualizing access graph");
        let now = Instant::now();
        ag.visualize("/tmp/graph.svg")
            .context("failed to visualize")?;
        info!(
            "access graph creation took {} seconds",
            now.elapsed().as_secs_f32()
        );
    } else {
        info!("Skipping visualization.")
    };

    Ok(())
}
