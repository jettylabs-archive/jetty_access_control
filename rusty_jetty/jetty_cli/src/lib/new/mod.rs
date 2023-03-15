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

use anyhow::{bail, Context, Result};

use jetty_core::{
    fetch_credentials,
    jetty::{CredentialsMap, JettyConfig},
    project::{self, tags_cfg_path},
    write,
};
use tokio::io::AsyncWriteExt;

use crate::{
    new::fs::{create_dir_ignore_failure, create_file},
    new_jetty_with_connectors,
};

use self::inquiry::{inquire_add, inquire_init};

struct ProjectStructure {
    jetty_config: JettyConfig,
    credentials: HashMap<String, HashMap<String, String>>,
}

impl ProjectStructure {
    fn new(
        jetty_config: &JettyConfig,
        credentials: HashMap<String, HashMap<String, String>>,
    ) -> Self {
        Self {
            jetty_config: jetty_config.to_owned(),
            credentials,
        }
    }
}

/// Main initialization fn to ask the user for the necessary information and
/// create the relevant project structure.
pub async fn new(
    from: &Option<PathBuf>,
    overwrite_project_dir: bool,
    project_name: &Option<String>,
) -> Result<()> {
    let (jetty_config, credentials) = if let Some(from_config) = from {
        // This is a shortcut for debugging and re-initialization with an existing config.
        let jt = JettyConfig::read_from_file(from_config)?;
        let credentials = fetch_credentials(project::connector_cfg_path())?;
        (jt, credentials)
    } else {
        inquire_init(overwrite_project_dir, project_name).await?
    };

    if credentials.is_empty() {
        bail!("skipping project initialization - no connectors were configured");
    }

    initialize_project_structure(ProjectStructure::new(&jetty_config, credentials)).await?;

    // create a new repository in the directory specified by the project name
    create_git_repo(jetty_config.get_name())?;
    // add schemas and vs code settings
    let jetty = &new_jetty_with_connectors(jetty_config.get_name(), false).await?;
    write::config::write_settings_and_schema(jetty, jetty_config.get_name())?;
    Ok(())
}

/// Create a new gite repository, renaming the master branch to main
fn create_git_repo<P: AsRef<Path>>(project_path: P) -> Result<()> {
    if git2::Repository::init(&project_path).is_ok() {
        let git_head_path = PathBuf::from(project_path.as_ref()).join(".git/head");
        std::fs::write(git_head_path, "ref: refs/heads/main")
            .context("updating name of branch to main")?;
        std::fs::write(
            PathBuf::from(project_path.as_ref()).join(".gitignore"),
            "# Environment state data\n.data/\n",
        )?;

        Ok(())
    } else {
        bail!("Unable to create git repository for project")
    }
}

/// Add connectors to an existing project
pub async fn add() -> Result<()> {
    let (jetty_config, credentials) = inquire_add().await?;
    update_project_configs(jetty_config, credentials).await
}

async fn update_project_configs(
    jetty_config: JettyConfig,
    credentials: HashMap<String, CredentialsMap>,
) -> Result<()> {
    // Open in the existing config file.
    let mut config_file = OpenOptions::new()
        .write(true)
        //Use open rather than create to avoid truncating the file before we're sure if both files are open-able.
        .open(project::jetty_cfg_path_local())
        .context(format!(
            "Opening Jetty Config file at ({})",
            project::jetty_cfg_path_local().to_string_lossy()
        ))?;

    // Read in the existing credentials.
    let mut credentials_file = OpenOptions::new()
        .write(true)
        //Use open rather than create to avoid truncating the file before we're sure if both files are open-able.
        .open(project::connector_cfg_path())
        .context(format!(
            "Opening Jetty Connectors file at ({})",
            project::connector_cfg_path().to_string_lossy()
        ))?;

    // Convert config to bytes, then write
    let config_yaml = jetty_config.to_yaml()?;
    let connectors_yaml = yaml_peg::serde::to_string(&credentials).map_err(anyhow::Error::from)?;

    // truncate the files - At this point we were able to open both files and serialize the config properly.
    // If things go south beyond this point, we risk deleting existing configurations.
    config_file.set_len(0)?;
    credentials_file.set_len(0)?;

    config_file.write_all(config_yaml.as_bytes())?;
    credentials_file.write_all(connectors_yaml.as_bytes())?;

    println!("\n\nSuccessfully added connectors to your project!");
    println!("To get started, run the following command:\n");
    println!("\t$ jetty bootstrap\n\n");

    Ok(())
}

/// Create the project structure and relevant files.
///
/// Connector credentials belong in ~/.jetty/connectors.yaml.
/// Everything else is local.
///
/// See the [project] module for a description of project layout.
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
    let jetty_config_dir = home_dir.join(".jetty");
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
    println!("\t$ cd {project_path}");
    println!("\t$ jetty bootstrap\n\n");
    Ok(())
}
