//! Full CLI library for Jetty Core
//!

#![deny(missing_docs)]

mod ascii;
mod cmd;
mod init;
mod tui;
mod usage_stats;

use std::{collections::HashMap, env, fs, sync::Arc, time::Instant};

use anyhow::{anyhow, bail, Context, Result};

use clap::Parser;

use human_panic::setup_panic;

use jetty_core::{
    access_graph::AccessGraph,
    connectors::{ConnectorClient, NewConnector},
    fetch_credentials,
    jetty::{ConnectorNamespace, CredentialsMap, JettyConfig},
    logging::{self, debug, error, info},
    project::{self, groups_cfg_path_local},
    write::Diffs,
    Connector, Jetty,
};

use crate::{
    cmd::{JettyArgs, JettyCommand},
    usage_stats::{record_usage, UsageEvent},
};

/// Main CLI entrypoint.
pub async fn cli() -> Result<()> {
    // Setup logging
    let reload_handle = logging::setup(None);

    // Setup panic handler
    setup_panic!(Metadata {
        name: env!("CARGO_PKG_NAME").into(),
        version: env!("CARGO_PKG_VERSION").into(),
        authors: "Jetty Support <support@get-jetty.com>".into(),
        homepage: "get-jetty.com".into(),
    });
    // Get Jetty Config
    let jetty_config = JettyConfig::read_from_file(project::jetty_cfg_path_local()).ok();
    // Get args
    let args = if env::args().collect::<Vec<_>>().len() == 1 {
        // Invoke telemetry for empty args. If we executed `JettyArgs::parse()` first,
        // the program would exit before we got to publish usage.
        record_usage(UsageEvent::InvokedDefault, &jetty_config)
            .await
            .unwrap_or_else(|_| debug!("Failed to publish usage."));
        JettyArgs::parse()
    } else {
        let args = JettyArgs::parse();
        // Invoke telemetry
        let event = match args.command {
            JettyCommand::Init { .. } => UsageEvent::InvokedInit,
            JettyCommand::Fetch { .. } => UsageEvent::InvokedFetch {
                connector_types: if let Some(c) = &jetty_config {
                    c.connectors
                        .values()
                        .map(|c| c.connector_type.to_owned())
                        .collect()
                } else {
                    vec![]
                },
            },
            JettyCommand::Explore { .. } => UsageEvent::InvokedExplore,
            JettyCommand::Add => UsageEvent::InvokedAdd,
            JettyCommand::Bootstrap {
                no_fetch,
                overwrite,
            } => UsageEvent::InvokedBootstrap {
                no_fetch,
                overwrite,
            },
            JettyCommand::Diff { fetch } => UsageEvent::InvokedDiff { fetch },
            JettyCommand::Plan { fetch } => UsageEvent::InvokedPlan { fetch },
            JettyCommand::Apply { no_fetch } => UsageEvent::InvokedApply { no_fetch },
        };
        record_usage(event, &jetty_config)
            .await
            .unwrap_or_else(|_| debug!("Failed to publish usage."));
        args
    };
    // Adjust logging levels based on args
    logging::update_filter_level(reload_handle, args.log_level);

    // ...and we're off!
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
            let jetty = new_jetty_with_connectors().await.map_err(|_| {
                anyhow!(
                    "unable to find {} - make sure you are in a \
                Jetty project directory, or create a new project by running `jetty init`",
                    project::jetty_cfg_path_local().display()
                )
            })?;

            jetty.try_access_graph()?;
            jetty_explore::explore_web_ui(Arc::from(jetty.access_graph.unwrap()), bind).await;
        }
        JettyCommand::Add => {
            init::add().await?;
        }
        JettyCommand::Bootstrap {
            no_fetch,
            overwrite,
        } => {
            if !*no_fetch {
                info!("Fetching data before bootstrap");
                fetch(&None, &false).await?;
            };
            bootstrap(*overwrite).await?;
        }
        JettyCommand::Diff { fetch: fetch_first } => {
            if *fetch_first {
                info!("Fetching data before bootstrap");
                fetch(&None, &false).await?;
            };
            diff().await?;
        }
        JettyCommand::Plan { fetch: fetch_first } => {
            if *fetch_first {
                info!("Fetching data before bootstrap");
                fetch(&None, &false).await?;
            };
            plan().await?;
        }
        JettyCommand::Apply { no_fetch } => {
            if !*no_fetch {
                info!("Fetching data before bootstrap");
                fetch(&None, &false).await?;
            };
            apply().await?;
        }
    }

    Ok(())
}

async fn fetch(connectors: &Option<Vec<String>>, &visualize: &bool) -> Result<()> {
    let jetty = new_jetty_with_connectors().await.map_err(|_| {
        anyhow!(
            "unable to find {} - make sure you are in a \
        Jetty project directory, or create a new project by running `jetty init`",
            project::jetty_cfg_path_local().display()
        )
    })?;

    let mut data_from_connectors = vec![];

    // Handle optionally fetching for only a few connectors
    let selected_connectors = if let Some(conns) = connectors {
        jetty
            .connectors
            .into_iter()
            .filter(|(name, _config)| conns.contains(&name.to_string()))
            .collect::<HashMap<_, _>>()
    } else {
        jetty.connectors
    };

    for (namespace, mut conn) in selected_connectors {
        info!("getting {} data", namespace);
        let now = Instant::now();
        let data = conn.get_data().await;
        let pcd = (data, namespace.to_owned());
        info!(
            "{} data took {:.1} seconds",
            namespace,
            now.elapsed().as_secs_f32()
        );
        data_from_connectors.push(pcd);
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

async fn bootstrap(overwrite: bool) -> Result<()> {
    let mut jetty = new_jetty_with_connectors().await.map_err(|_| {
        anyhow!(
            "unable to find {} - make sure you are in a \
        Jetty project directory, or create a new project by running `jetty init`",
            project::jetty_cfg_path_local().display()
        )
    })?;

    // make sure there's an existing access graph
    jetty.try_access_graph()?;

    // Build all the yaml first
    let group_yaml = jetty.build_bootstrapped_group_yaml()?;
    // TODO: Add Policy yaml

    // Now check for all the files
    if !overwrite {
        if groups_cfg_path_local().exists() {
            bail!("{} already exists; run `jetty bootstrap --overwrite` to overwrite the existing configuration", groups_cfg_path_local().to_string_lossy())
        }
    }

    // Now write the yaml files

    // groups
    fs::create_dir_all(groups_cfg_path_local().parent().unwrap()).unwrap(); // Create the parent dir, if needed
    fs::write(groups_cfg_path_local(), group_yaml)?; // write the contents
                                                     // sanity check - the diff should be empty at this point
    if jetty_core::write::get_group_diff(&jetty)
        .context("checking the generated group configuration")?
        .len()
        != 0
    {
        bail!("something went wrong - the configuration generated doesn't fully match the true state; please contact support: support@get-jetty.com")
    }

    // TODO: Policies

    Ok(())
}

async fn diff() -> Result<()> {
    let mut jetty = new_jetty_with_connectors().await.map_err(|_| {
        anyhow!(
            "unable to find {} - make sure you are in a \
        Jetty project directory, or create a new project by running `jetty init`",
            project::jetty_cfg_path_local().display()
        )
    })?;

    // make sure there's an existing access graph
    jetty.try_access_graph()?;

    // For now, we're just looking at group diffs
    let group_diff = jetty_core::write::get_group_diff(&jetty)?;
    if !group_diff.is_empty() {
        group_diff.iter().for_each(|diff| println!("{diff}"));
    } else {
        println!("No changes found");
    };

    Ok(())
}

async fn plan() -> Result<()> {
    let mut jetty = new_jetty_with_connectors().await.map_err(|_| {
        anyhow!(
            "unable to find {} - make sure you are in a \
        Jetty project directory, or create a new project by running `jetty init`",
            project::jetty_cfg_path_local().display()
        )
    })?;

    // make sure there's an existing access graph
    let ag = jetty.try_access_graph()?;

    let diffs = Diffs {
        groups: jetty_core::write::get_group_diff(&jetty)?,
    };

    let connector_specific_diffs = diffs.split_by_connector();

    let tr = ag.translator();

    let local_diffs = connector_specific_diffs
        .iter()
        .map(|(k, v)| (k.to_owned(), tr.translate_diffs_to_local(v)))
        .collect::<HashMap<_, _>>();

    let plans: HashMap<_, _> = local_diffs
        .iter()
        .map(|(k, v)| (k.to_owned(), jetty.connectors[k].plan_changes(v)))
        .collect();

    for (c, plan) in plans {
        println!("{c}:");
        plan.iter().for_each(|s| println!("  {s}"));
        println!("\n")
    }
    Ok(())
}

async fn apply() -> Result<()> {
    let mut jetty = new_jetty_with_connectors().await.map_err(|_| {
        anyhow!(
            "unable to find {} - make sure you are in a \
        Jetty project directory, or create a new project by running `jetty init`",
            project::jetty_cfg_path_local().display()
        )
    })?;

    // make sure there's an existing access graph
    let ag = jetty.try_access_graph()?;

    let diffs = Diffs {
        groups: jetty_core::write::get_group_diff(&jetty)?,
    };

    let connector_specific_diffs = diffs.split_by_connector();

    let tr = ag.translator();

    let local_diffs = connector_specific_diffs
        .iter()
        .map(|(k, v)| (k.to_owned(), tr.translate_diffs_to_local(v)))
        .collect::<HashMap<_, _>>();

    let mut results: HashMap<_, _> = HashMap::new();

    for (conn, diff) in local_diffs {
        results.insert(
            conn.to_owned(),
            jetty.connectors[&conn].apply_changes(&diff).await?,
        );
    }
    println!("Fetching updated access information");
    match fetch(&None, &false).await {
        Ok(_) => {
            println!("In some cases, applied changes may have side effects. Here is the current diff based on your configuration:");
            // For now, we're just looking at group diffs
            let group_diff = jetty_core::write::get_group_diff(&jetty)?;
            if !group_diff.is_empty() {
                group_diff.iter().for_each(|diff| println!("{diff}"));
            } else {
                println!("No changes found");
            };
        }
        Err(_) => {
            error!("unable to perform fetch");
            println!("In some cases, applied changes may have side effects. We recommend running `jetty diff -f` to see if you should run apply again");
        }
    };

    for (c, result) in results {
        println!("{c}:\n{result}\n");
    }

    Ok(())
}

async fn get_connectors(
    creds: &HashMap<String, CredentialsMap>,
    selected_connectors: &HashMap<ConnectorNamespace, jetty_core::jetty::ConnectorConfig>,
) -> Result<HashMap<ConnectorNamespace, Box<dyn Connector>>> {
    let mut connector_map: HashMap<ConnectorNamespace, Box<dyn Connector>> = HashMap::new();

    for (namespace, config) in selected_connectors {
        connector_map.insert(
            namespace.to_owned(),
            match config.connector_type.as_str() {
                "dbt" => {
                    jetty_dbt::DbtConnector::new(
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
                    .await?
                }
                "snowflake" => {
                    jetty_snowflake::SnowflakeConnector::new(
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
                    .await?
                }
                "tableau" => {
                    jetty_tableau::TableauConnector::new(
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
                    .await?
                }
                _ => panic!(
                    "unknown connector type: {}",
                    config.connector_type.to_owned()
                ),
            },
        );
    }
    Ok(connector_map)
}

/// Create a new Jetty struct with all the connectors. Uses default locations for everything
pub async fn new_jetty_with_connectors() -> Result<Jetty> {
    let config = JettyConfig::read_from_file(project::jetty_cfg_path_local())
        .context("Reading Jetty Config file")?;

    let creds = fetch_credentials(project::connector_cfg_path()).map_err(|_| {
        anyhow!(
            "unable to find {} - you can set this up by running `jetty init`",
            project::connector_cfg_path().display()
        )
    })?;

    let connectors = get_connectors(&creds, &config.connectors).await?;

    Jetty::new_with_config(config, project::data_dir(), connectors)
}
