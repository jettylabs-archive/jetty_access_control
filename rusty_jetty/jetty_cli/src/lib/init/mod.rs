//! Utilities for initializing a Jetty project from scratch.
//!

mod fs;
mod inquiry;
mod pki;

use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::Result;

use jetty_core::{fetch_credentials, jetty::JettyConfig};
use tokio::io::AsyncWriteExt;

use crate::{
    init::fs::{create_dir_ignore_failure, create_file},
    project::{self, tags_cfg_path},
};

use self::inquiry::inquire_init;

struct ProjectStructure {
    jetty_config: JettyConfig,
    credentials: HashMap<String, HashMap<String, String>>,
}

impl ProjectStructure {
    fn new(
        jetty_config: JettyConfig,
        credentials: HashMap<String, HashMap<String, String>>,
    ) -> Self {
        Self {
            jetty_config,
            credentials,
        }
    }
}

/// Main initialization fn to ask the user for the necessary information and
/// create the relevant project structure.
pub async fn init(
    from: &Option<PathBuf>,
    overwrite_project_dir: bool,
    project_name: &Option<String>,
) -> Result<()> {
    let (jetty_config, credentials) = if let Some(from_config) = from {
        // This is a shortcut for debugging and reinitialization with an existing config.
        let jt = JettyConfig::read_from_file(from_config)?;
        let credentials = fetch_credentials(project::connector_cfg_path())?;
        (jt, credentials)
    } else {
        inquire_init(overwrite_project_dir, project_name).await?
    };

    initialize_project_structure(ProjectStructure::new(jetty_config, credentials)).await?;
    Ok(())
}

/// Create the project structure and relevant files.
///
/// Connector credentials belong in ~/.jetty/connectors.yaml.
/// Everything else is local.
///
/// The project structure currently looks like this:
///
/// pwd
///  └── {project_name}
///       ├── jetty_config.yaml
///       ├── .data
///       │    ├── jetty_graph
///       │    └── {connector}
///       │         └── {connector-specific data}
///       └── tags
///            └── tags.yaml
async fn initialize_project_structure(
    ProjectStructure {
        jetty_config: jt_config,
        credentials,
    }: ProjectStructure,
) -> Result<()> {
    // We assume the configs don't exist yet. Create them and then the project.
    println!("Creating project files...");

    let project_path = jt_config.get_name();
    create_dir_ignore_failure(&project_path).await;
    let jetty_config = create_file(project::jetty_cfg_path(&project_path)).await;
    let home_dir = dirs::home_dir().expect("Couldn't find your home directory.");
    let jetty_config_dir = home_dir.join("./.jetty");
    create_dir_ignore_failure(jetty_config_dir).await;

    let connectors_config = OpenOptions::new()
        .create(true)
        .append(true)
        .open(project::connector_cfg_path());

    if let Ok(mut cfg) = jetty_config {
        cfg.write_all(jt_config.to_yaml()?.as_bytes()).await?;
    }

    let connectors_yaml = yaml_peg::serde::to_string(&credentials).map_err(anyhow::Error::from)?;
    if let Ok(mut cfg) = connectors_config {
        cfg.write_all(connectors_yaml.as_bytes())?;
    }
    // create tags parent dir if needed
    let tags_parent_dir = tags_cfg_path(project_path.clone())
        .parent()
        .unwrap()
        .to_owned();
    create_dir_ignore_failure(tags_parent_dir).await;
    let mut tags_config = create_file(project::tags_cfg_path(project_path.clone())).await?;

    tags_config
        .write_all(
            "
# This file is tagging assets by attribute.
# 
# For example, you may want to identify a Snowflake table as personally 
# identifiable information (PII). 
# See more at docs.get-jetty.com/docs/getting-started
#
# pii:
#   # Optional - description of the tag
#   description: Includes sensitive information 
#   # Optional - whether the tag should be inherited by assets in the downstream hierarchy (default: false)
#   pass_through_hierarchy: false
#   # Optional - whether the tag should be inherited by assets derived from these assets (default: false)
#   pass_through_lineage: true
#   # List of assets to be tagged
#   apply_to:
#       # Can be a full asset name or a unique fragment of an asset name
#       - snow::JETTY_TEST_DB/RAW/IRIS
#       # Can also be an object that specifies the name and type of an asset
#       - name: snow::JETTY_TEST_DB/RAW/CUSTOMERS
#         type: table
#   # Optional - list of assets to have the tag removed from
#   remove_from:
#       - tableau::My Project/Iris Dashboard

"
            .as_bytes(),
        )
        .await?;

    println!("\n\nCongratulations, your jetty project has been created and configured!");
    println!("To get started, run the following commands:\n");
    println!("\t$ cd {}", project_path);
    println!("\t$ jetty explore --fetch\n\n");
    Ok(())
}
