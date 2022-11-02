//! Full CLI library for Jetty Core
//!

#![deny(missing_docs)]

mod ascii;
mod init;
mod project;
mod tui;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};

use anyhow::{anyhow, bail, Context, Result};
use clap::{Parser, Subcommand};
use human_panic::setup_panic;

use jetty_core::{
    access_graph::AccessGraph,
    connectors::{ConnectorClient, NewConnector},
    fetch_credentials,
    jetty::ConnectorNamespace,
    logging::{self, debug, info, warn, LevelFilter},
    Connector, Jetty,
};

/// Jetty CLI: Open-source data access control for modern teams
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None, arg_required_else_help = true)]
struct Args {
    #[clap(subcommand)]
    command: JettyCommand,
    #[clap(short, long)]
    log_level: Option<LevelFilter>,
}

#[derive(Subcommand, Debug)]
enum JettyCommand {
    /// Initialize a Jetty project.
    Init {
        /// Project name
        project_name: Option<String>,
        /// Initialize from an existing config (as a shortcut).
        #[clap(short, long, hide = true)]
        from: Option<PathBuf>,
        /// Overwrite project directory if it exists
        #[clap(short, long, value_parser, default_value = "false")]
        overwrite: bool,
    },
    Fetch {
        /// Visualize the graph in an SVG file.
        #[clap(short, long, value_parser, default_value = "false")]
        visualize: bool,
        /// Connectors to collect for.
        #[clap(short, long, use_value_delimiter = true, value_delimiter = ',')]
        connectors: Option<Vec<String>>,
    },
    Explore {
        #[clap(short, long, value_parser, default_value = "false")]
        fetch: bool,

        /// Select the ip and port to bind the server to (e.g. 127.0.0.1:3000)
        #[clap(short, long, value_parser)]
        bind: Option<String>,
    },
}

/// Main CLI entrypoint.
pub async fn cli() -> Result<()> {
    setup_panic!(Metadata {
        name: env!("CARGO_PKG_NAME").into(),
        version: env!("CARGO_PKG_VERSION").into(),
        authors: "Jetty Support <support@get-jetty.com>".into(),
        homepage: "get-jetty.com".into(),
    });
    let args = Args::parse();
    logging::setup(args.log_level);

    match &args.command {
        JettyCommand::Init {
            from,
            project_name,
            overwrite,
        } => {
            init::init(from, *overwrite, project_name).await?;
        }

        JettyCommand::Fetch {
            visualize,
            connectors,
        } => {
            fetch(connectors, visualize).await?;
        }

        JettyCommand::Explore {
            fetch: fetch_first,
            bind,
        } => {
            if *fetch_first {
                info!("Fetching all data first.");
                fetch(&None, &false).await?;
            }
            match AccessGraph::deserialize_graph(
                project::data_dir().join(project::graph_filename()),
            ) {
                Ok(mut ag) => {
                    let tags_path = project::tags_cfg_path_local();
                    if tags_path.exists() {
                        debug!("Getting tags from config.");
                        let tag_config = std::fs::read_to_string(&tags_path);
                        match tag_config {
                            Ok(c) => {
                                ag.add_tags(&c)?;
                            }
                            Err(e) => {
                                bail!(
                                    "found, but was unable to read {:?}\nerror: {}",
                                    tags_path,
                                    e
                                )
                            }
                        }
                    } else {
                        debug!("No tags file found. Skipping ingestion.")
                    }

                    jetty_explore::explore_web_ui(Arc::new(ag), bind).await;
                }
                Err(e) => info!(
                    "Unable to find saved graph. Try running `jetty fetch`\nError: {}",
                    e
                ),
            }
        }
    }

    Ok(())
}

async fn fetch(connectors: &Option<Vec<String>>, &visualize: &bool) -> Result<()> {
    let jetty = Jetty::new(project::jetty_cfg_path_local(), project::data_dir()).map_err(|_| {
        anyhow!(
            "unable to find {} - make sure you are in a \
        Jetty project directory, or create a new project by running `jetty init`",
            project::jetty_cfg_path_local().display()
        )
    })?;
    let creds = fetch_credentials(project::connector_cfg_path()).map_err(|_| {
        anyhow!(
            "unable to find {} - you can set this up by running `jetty init`",
            project::connector_cfg_path().display()
        )
    })?;

    let mut data_from_connectors = vec![];

    let selected_connectors;

    selected_connectors = if let Some(conns) = connectors {
        jetty
            .config
            .connectors
            .into_iter()
            .filter(|(name, _config)| conns.contains(&name.to_string()))
            .collect::<HashMap<_, _>>()
    } else {
        jetty.config.connectors
    };

    for (namespace, config) in &selected_connectors {
        match config.connector_type.as_str() {
            "dbt" => {
                let mut dbt = jetty_dbt::DbtConnector::new(
                    &selected_connectors[namespace],
                    &creds
                        .get(namespace.to_string().as_str())
                        .ok_or(anyhow!(
                            "unable to find a connector called {} in {}",
                            namespace,
                            project::connector_cfg_path().display()
                        ))?
                        .to_owned(),
                    Some(ConnectorClient::Core),
                    Some(project::data_dir().join(namespace.to_string())),
                )
                .await?;

                info!("getting {} data", namespace);
                let now = Instant::now();
                let dbt_data = dbt.get_data().await;
                let dbt_pcd = (dbt_data, namespace.to_owned());
                info!(
                    "{} data took {:.1} seconds",
                    namespace,
                    now.elapsed().as_secs_f32()
                );
                data_from_connectors.push(dbt_pcd);
            }
            "snowflake" => {
                let mut snow = jetty_snowflake::SnowflakeConnector::new(
                    &selected_connectors[namespace],
                    &creds
                        .get(namespace.to_string().as_str())
                        .ok_or(anyhow!(
                            "unable to find a connector called {} in {}",
                            namespace,
                            project::connector_cfg_path().display()
                        ))?
                        .to_owned(),
                    Some(ConnectorClient::Core),
                    Some(project::data_dir().join(namespace.to_string())),
                )
                .await?;

                info!("getting {} data", namespace);
                let now = Instant::now();
                let snow_data = snow.get_data().await;
                let snow_pcd = (snow_data, namespace.to_owned());
                info!(
                    "{} data took {:.1} seconds",
                    namespace,
                    now.elapsed().as_secs_f32()
                );
                data_from_connectors.push(snow_pcd);
            }
            "tableau" => {
                let mut tab = jetty_tableau::TableauConnector::new(
                    &selected_connectors[namespace],
                    &creds
                        .get(namespace.to_string().as_str())
                        .ok_or(anyhow!(
                            "unable to find a connector called {} in {}",
                            namespace,
                            project::connector_cfg_path().display()
                        ))?
                        .to_owned(),
                    Some(ConnectorClient::Core),
                    Some(project::data_dir().join(namespace.to_string())),
                )
                .await?;

                info!("getting {} data", namespace);
                let now = Instant::now();
                tab.setup().await?;
                let tab_data = tab.get_data().await;
                let tab_pcd = (tab_data, namespace.to_owned());
                info!(
                    "{} data took {:.1} seconds",
                    namespace,
                    now.elapsed().as_secs_f32()
                );
                data_from_connectors.push(tab_pcd);
            }
            o => bail!("unknown connector type: {o}"),
        }
    }

    info!("creating access graph");
    let now = Instant::now();

    let ag = AccessGraph::new_from_connector_data(data_from_connectors)?;

    info!(
        "access graph creation took {:.1} seconds",
        now.elapsed().as_secs_f32()
    );
    ag.serialize_graph(project::data_dir().join(project::graph_filename()))?;

    if visualize {
        info!("visualizing access graph");
        let now = Instant::now();
        ag.visualize("/tmp/graph.svg")
            .context("failed to visualize")?;
        info!(
            "access graph creation took {:.1} seconds",
            now.elapsed().as_secs_f32()
        );
    } else {
        info!("Skipping visualization.")
    };

    Ok(())
}
