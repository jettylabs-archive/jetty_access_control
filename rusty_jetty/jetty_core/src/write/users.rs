//! Functionality to manage the write path for users

pub mod bootstrap;
mod parser;

use std::collections::HashMap;

use anyhow::{Context, Result};
use bimap::BiHashMap;
use glob::glob;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{access_graph::NodeName, jetty::ConnectorNamespace, project};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct UserYaml {
    name: String,
    identifiers: HashMap<ConnectorNamespace, String>,
    id: Uuid,
}

/// Get the paths of all asset config files
fn get_config_paths() -> Result<glob::Paths> {
    // collect the paths to all the config files
    glob(
        format!(
            // the user files can be in whatever directory the user would like
            "{}/**/*.y*ml",
            project::users_cfg_root_path_local().to_string_lossy()
        )
        .as_str(),
    )
    .context("trouble generating config file paths")
}
