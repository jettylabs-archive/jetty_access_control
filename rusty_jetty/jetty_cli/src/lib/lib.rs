//! Full CLI library for Jetty Core
//!

#![deny(missing_docs)]

mod ascii;
mod cmd;
mod init;
mod tui;
mod usage_stats;

use std::{
    collections::HashMap,
    env, fs,
    str::FromStr,
    sync::Arc,
    thread,
    time::{self, Instant},
};

use anyhow::{anyhow, bail, Context, Result};
use clap::Parser;
use human_panic::setup_panic;
use indicatif::{ProgressBar, ProgressStyle};

use jetty_core::{
    access_graph::AccessGraph,
    connectors::{ConnectorClient, NewConnector},
    fetch_credentials,
    jetty::{ConnectorNamespace, CredentialsMap, JettyConfig},
    logging::{self, debug, error, info},
    project::{self, groups_cfg_path_local},
    write::{
        assets::{bootstrap::write_bootstrapped_asset_yaml, get_policy_diffs},
        groups::parse_and_validate_groups,
        Diffs,
    },
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
            JettyCommand::Subgraph { depth, .. } => UsageEvent::InvokedSubgraph { depth },
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
                info!("Fetching data before diff");
                fetch(&None, &false).await?;
            } else {
                println!("Generating diff based off existing data. Run `jetty diff -f` to fetch before generating the diff.")
            };
            diff().await?;
        }
        JettyCommand::Plan { fetch: fetch_first } => {
            if *fetch_first {
                info!("Fetching data before plan");
                fetch(&None, &false).await?;
            } else {
                println!("Generating plan based off existing data. Run `jetty plan -f` to fetch before generating the plan.")
            };
            plan().await?;
        }
        JettyCommand::Apply { no_fetch } => {
            if !*no_fetch {
                info!("Fetching data before apply. You can run `jetty apply -n` to run apply based on a previous fetch.");
                fetch(&None, &false).await?;
            };
            apply().await?;
        }
        JettyCommand::Subgraph { id, depth } => {
            let jetty = new_jetty_with_connectors().await.map_err(|_| {
                anyhow!(
                    "unable to find {} - make sure you are in a \
                Jetty project directory, or create a new project by running `jetty init`",
                    project::jetty_cfg_path_local().display()
                )
            })?;

            let ag = jetty.try_access_graph()?;
            let parsed_uuid = uuid::Uuid::from_str(id)?;
            let binding = ag.extract_graph(
                ag.get_untyped_index_from_id(&parsed_uuid)
                    .ok_or(anyhow!("unable to find the right node"))?,
                *depth,
            );
            let generated_dot = binding.dot();
            println!("{generated_dot:?}");
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
        let pb = basic_progress_bar(format!("Fetching {} data", namespace).as_str());

        let now = Instant::now();
        let data = conn.get_data().await;
        let pcd = (data, namespace.to_owned());

        data_from_connectors.push(pcd);

        pb.finish_with_message(format!(
            "Fetching {} data took {:.1} seconds",
            namespace,
            now.elapsed().as_secs_f32()
        ));
    }

    let pb = basic_progress_bar("Creating access graph");
    let now = Instant::now();

    let ag = AccessGraph::new_from_connector_data(data_from_connectors)?;
    ag.serialize_graph(project::data_dir().join(project::graph_filename()))?;

    pb.finish_with_message(format!(
        "Access graph created in data took {:.1} seconds",
        now.elapsed().as_secs_f32()
    ));

    if visualize {
        info!("visualizing access graph");
        let now = Instant::now();
        ag.visualize("/tmp/graph.svg")
            .context("failed to visualize")?;
        info!(
            "Access graph visualized at \"/tmp/graph.svg\" (took {:.1} seconds)",
            now.elapsed().as_secs_f32()
        );
    } else {
        debug!("Skipping visualization.")
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
    let group_yaml = jetty.generate_bootstrapped_group_yaml()?;
    let asset_yaml = jetty.generate_bootstrapped_policy_yaml()?;

    // Now check for all the files
    if !overwrite {
        if groups_cfg_path_local().exists() {
            bail!("{} already exists; run `jetty bootstrap --overwrite` to overwrite the existing configuration", groups_cfg_path_local().to_string_lossy())
        }
        if project::assets_cfg_root_path().exists() {
            bail!("{} already exists; run `jetty bootstrap --overwrite` to overwrite the existing configuration", project::assets_cfg_root_path().to_string_lossy())
        }
    } else {
        match fs::remove_file(groups_cfg_path_local()) {
            Ok(_) => println!("removed existing groups file"),
            Err(_) => (),
        };
        match fs::remove_dir_all(project::assets_cfg_root_path()) {
            Ok(_) => println!("removed existing asset directory"),
            Err(_) => (),
        };
    }

    // Now write the yaml files

    // groups
    fs::create_dir_all(groups_cfg_path_local().parent().unwrap()).unwrap(); // Create the parent dir, if needed
    fs::write(groups_cfg_path_local(), group_yaml)?; // write the contents
                                                     // assets
    write_bootstrapped_asset_yaml(asset_yaml)?;

    // sanity check - the diff should be empty at this point
    let validated_group_config = parse_and_validate_groups(&jetty)?;
    if jetty_core::write::get_group_diff(&validated_group_config, &jetty)
        .context("checking the generated group configuration")?
        .len()
        != 0
    {
        bail!("something went wrong - the configuration generated doesn't fully match the true state of your environment; please contact support: support@get-jetty.com")
    }

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
    let validated_group_config = parse_and_validate_groups(&jetty)?;
    let group_diff = jetty_core::write::get_group_diff(&validated_group_config, &jetty)?;

    // now get the policy diff
    // need to get the group configs and all available connectors
    let policy_diff = get_policy_diffs(&jetty, &validated_group_config)?;
    dbg!(&policy_diff);

    // Now print out the diffs
    println!("\nGROUPS\n----------------");
    if !group_diff.is_empty() {
        group_diff.iter().for_each(|diff| println!("{diff}"));
    } else {
        println!("No changes found");
    };

    println!("\nPOLICIES\n----------------");
    if !policy_diff.is_empty() {
        policy_diff.iter().for_each(|diff| println!("{diff}"));
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
    let validated_group_config = parse_and_validate_groups(&jetty)?;

    let diffs = Diffs {
        groups: jetty_core::write::get_group_diff(&validated_group_config, &jetty)?,
    };

    let connector_specific_diffs = diffs.split_by_connector();

    let tr = ag.translator();

    let local_diffs = connector_specific_diffs
        .iter()
        .map(|(k, v)| (k.to_owned(), tr.translate_diffs_to_local(v)))
        .collect::<HashMap<_, _>>();

    // Exit early if there haven't been any changes
    if local_diffs.is_empty() {
        println!("No changes found");
        return Ok(());
    }

    let plans: HashMap<_, _> = local_diffs
        .iter()
        .map(|(k, v)| (k.to_owned(), jetty.connectors[k].plan_changes(v)))
        .collect();

    for (c, plan) in plans {
        println!("{c}:");
        if !plan.is_empty() {
            plan.iter()
                .for_each(|s| println!("{}\n", textwrap::indent(s, "  ")));
            println!("\n")
        } else {
            println!("  No changes planned\n");
        }
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
    let validated_group_config = parse_and_validate_groups(&jetty)?;

    let diffs = Diffs {
        groups: jetty_core::write::get_group_diff(&validated_group_config, &jetty)?,
    };

    let connector_specific_diffs = diffs.split_by_connector();

    let tr = ag.translator();

    let local_diffs = connector_specific_diffs
        .iter()
        .map(|(k, v)| (k.to_owned(), tr.translate_diffs_to_local(v)))
        .collect::<HashMap<_, _>>();

    let mut results: HashMap<_, _> = HashMap::new();

    let pb = basic_progress_bar("Applying changes");
    let now = Instant::now();
    for (conn, diff) in local_diffs {
        results.insert(
            conn.to_owned(),
            jetty.connectors[&conn].apply_changes(&diff).await?,
        );
    }
    pb.finish_with_message(format!(
        "Changes applied in {:.1} seconds",
        now.elapsed().as_secs_f32()
    ));

    // Look at the updated data to see if the apply was successful
    println!("Waiting 5 seconds then fetching updated access information");
    timer_with_spinner(
        5,
        "Giving your tools a chance to update",
        "Done - beginning fetch",
    );

    match fetch(&None, &false).await {
        Ok(_) => {
            // reload jetty to get the latest fetch
            let jetty = new_jetty_with_connectors().await.map_err(|_| {
                anyhow!(
                    "unable to find {} - make sure you are in a \
                Jetty project directory, or create a new project by running `jetty init`",
                    project::jetty_cfg_path_local().display()
                )
            })?;

            println!("Here is the current diff based on your configuration:");
            // For now, we're just looking at group diffs

            let validated_group_config = parse_and_validate_groups(&jetty)?;
            let group_diff = jetty_core::write::get_group_diff(&validated_group_config, &jetty)?;
            if !group_diff.is_empty() {
                group_diff.iter().for_each(|diff| println!("{diff}"));
                println!(
                    r#"You might see outstanding changes for a couple of reasons:
    1. The underlying system is still updating (this is often the case with Tableau)
    2. The changes had side-effects. This is expected for some changes.
       You can run `jetty apply` again to make the necessary updates"#
                )
            } else {
                println!("No changes found");
            };
        }
        Err(_) => {
            error!("unable to perform fetch");
            println!("We recommend running `jetty diff -f` to see if you should run apply again to correct any side-effects of your changes");
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

fn timer_with_spinner(secs: u64, msg: &str, completion_msg: &str) {
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(time::Duration::from_millis(120));

    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} {msg}")
            .unwrap()
            // For more spinners check out the cli-spinners project:
            // https://github.com/sindresorhus/cli-spinners/blob/master/spinners.json
            .tick_strings(&[
                "▹▹▹▹▹",
                "▸▹▹▹▹",
                "▹▸▹▹▹",
                "▹▹▸▹▹",
                "▹▹▹▸▹",
                "▹▹▹▹▸",
                "▪▪▪▪▪",
            ]),
    );
    pb.set_message(msg.to_owned());
    thread::sleep(time::Duration::from_secs(secs));
    pb.finish_with_message(completion_msg.to_owned());
}

fn basic_progress_bar(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(time::Duration::from_millis(120));

    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} {msg}")
            .unwrap()
            // For more spinners check out the cli-spinners project:
            // https://github.com/sindresorhus/cli-spinners/blob/master/spinners.json
            .tick_strings(&[
                "▹▹▹▹▹",
                "▸▹▹▹▹",
                "▹▸▹▹▹",
                "▹▹▸▹▹",
                "▹▹▹▸▹",
                "▹▹▹▹▸",
                "▪▪▪▪▪",
            ]),
    );
    pb.set_message(msg.to_owned());
    pb
}
