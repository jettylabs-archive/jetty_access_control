//! Jetty Module
//!
use std::fs;
use std::path::{Path, PathBuf};
use std::{collections::HashMap, fmt::Display};

use anyhow::{anyhow, Result};

use log::debug;
use serde::{Deserialize, Serialize};
use yaml_peg::serde as yaml;

/// The user-defined namespace corresponding to the connector.
#[derive(Deserialize, Debug, Hash, PartialEq, Eq, Clone, Default, PartialOrd, Ord, Serialize)]
pub struct ConnectorNamespace(pub String);

impl Display for ConnectorNamespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
/// Struct representing the jetty_config.yaml file.
#[allow(dead_code)]
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct JettyConfig {
    version: String,
    name: String,
    /// All connector configs defined.
    pub connectors: HashMap<ConnectorNamespace, ConnectorConfig>,
}

impl JettyConfig {
    /// New === default for this simple constructor.
    pub fn new() -> Self {
        Self {
            version: "0.0.1".to_owned(),
            ..Default::default()
        }
    }

    /// Use the default filepath to ingest the Jetty config.
    pub fn read_from_file<P: AsRef<Path>>(path: P) -> Result<JettyConfig> {
        let config_raw = fs::read_to_string(path)?;
        let mut config = yaml::from_str::<JettyConfig>(&config_raw)?;

        config.pop().ok_or_else(|| anyhow!["failed"])
    }

    /// Set the project name.
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// Get the name
    pub fn get_name(&self) -> String {
        self.name.to_owned()
    }

    /// Convert this config to a yaml string.
    pub fn to_yaml(&self) -> Result<String> {
        yaml::to_string(self).map_err(anyhow::Error::from)
    }
}

/// Config for all connectors in this project.
#[allow(dead_code)]
#[derive(Deserialize, Serialize, Default, Debug)]
pub struct ConnectorConfig {
    /// The connector type
    #[serde(rename = "type")]
    pub connector_type: String,
    /// Additional configuration, specific to the connector
    #[serde(flatten)]
    pub config: HashMap<String, String>,
}

impl ConnectorConfig {
    /// Basic constructor
    pub fn new(connector_type: String, config: HashMap<String, String>) -> Self {
        Self {
            connector_type,
            config,
        }
    }
}

/// Alias for HashMap to hold credentials information.
pub type CredentialsMap = HashMap<String, String>;

/// Fetch the credentials from the Jetty connectors config.
pub fn fetch_credentials(path: PathBuf) -> Result<HashMap<String, CredentialsMap>> {
    debug!("Trying to read credentials from {:?}", path);
    let credentials_raw = fs::read_to_string(path)?;
    let mut config = yaml::from_str::<HashMap<String, CredentialsMap>>(&credentials_raw)?;

    config
        .pop()
        .ok_or_else(|| anyhow!["failed to generate credentials"])
}

/// Represents Jetty Core in its entirety.
pub struct Jetty {
    /// The main jetty_config.yaml
    pub config: JettyConfig,
    // connector_config: HashMap<String, ConnectorCredentials>,
    /// The directory where data (such as the materialized graph) should be stored
    data_dir: PathBuf,
}

impl Jetty {
    /// Convenience method for struct creation. Uses the default location for
    /// config files.
    pub fn new<P: AsRef<Path>>(jetty_config_path: P, data_dir: PathBuf) -> Result<Self> {
        // load a saved access graph or create an empty one

        Ok(Jetty {
            config: JettyConfig::read_from_file(jetty_config_path)?,
            data_dir,
        })
    }
}
