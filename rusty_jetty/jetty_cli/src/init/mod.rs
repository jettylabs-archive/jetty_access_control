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

pub(crate) async fn init(from: &Option<PathBuf>, overwrite_project_dir: bool) -> Result<()> {
    let (jetty_config, credentials) = if let Some(from_config) = from {
        // This is a shortcut for debugging and reinitialization with an existing config.
        let jt = JettyConfig::read_from_file(from_config)?;
        let credentials = fetch_credentials(project::connector_cfg_path())?;
        (jt, credentials)
    } else {
        inquire_init(overwrite_project_dir).await?
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
///  └──{project_name}
///       ├── jetty_config.yaml
///       └── src
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
    create_dir_ignore_failure(tags_cfg_path(project_path.clone()).parent().unwrap()).await;
    let mut tags_config = create_file(project::tags_cfg_path(project_path)).await?;

    tags_config
        .write_all(
            "
# This file is tagging assets by attribute.
# 
# For example, you may want to identify a Snowflake table as personally 
# identifiable information (PII). 
# See more at docs.get-jetty.com/docs/getting-started/assets#tagging-assets
#
# pii:
#   description: This data contains pii from ppis
#   apply_to:
#       - snowflake://cea26391.snowflakecomputing.com/JETTY_TEST_DB2/RAW/IRIS

    "
            .as_bytes(),
        )
        .await?;

    println!("Project created!");
    Ok(())
}
