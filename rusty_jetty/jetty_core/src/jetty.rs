//! Jetty Module
//!
use std::fs;
use std::{collections::HashMap, fmt::Display};

use anyhow::{anyhow, Result};
use dirs::home_dir;
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
#[derive(Deserialize, Debug)]
pub struct JettyConfig {
    version: String,
    name: String,
    /// All connector configs defined.
    pub connectors: HashMap<ConnectorNamespace, ConnectorConfig>,
}

impl JettyConfig {
    /// Use the default filepath to ingest the Jetty config.
    pub fn new() -> Result<JettyConfig> {
        let config_raw = fs::read_to_string("./jetty_config.yaml")?;
        let mut config = yaml::from_str::<JettyConfig>(&config_raw)?;

        config.pop().ok_or_else(|| anyhow!["failed"])
    }
}

/// Config for all connectors in this project.
#[allow(dead_code)]
#[derive(Deserialize, Default, Debug)]
pub struct ConnectorConfig {
    #[serde(rename = "type")]
    connector_type: String,
    /// Additional configuration, specific to the connector
    #[serde(flatten)]
    pub config: HashMap<String, String>,
}

/// Alias for HashMap to hold credentials information.
pub type CredentialsBlob = HashMap<String, String>;

/// Fetch the credentials from the Jetty connectors config.
pub fn fetch_credentials() -> Result<HashMap<String, CredentialsBlob>> {
    let mut default_path = home_dir().unwrap();

    default_path.push(".jetty/connectors.yaml");

    let credentials_raw = fs::read_to_string(default_path)?;
    let mut config = yaml::from_str::<HashMap<String, CredentialsBlob>>(&credentials_raw)?;

    config
        .pop()
        .ok_or_else(|| anyhow!["failed to generate credentials"])
}

/// Represents Jetty Core in its entirety.
pub struct Jetty {
    /// The main jetty_config.yaml
    pub config: JettyConfig,
    // connector_config: HashMap<String, ConnectorCredentials>,
}

impl Jetty {
    /// Convenience method for struct creation. Uses the default location for
    /// config files.
    pub fn new() -> Result<Self> {
        // load a saved access graph or create an empty one

        Ok(Jetty {
            config: JettyConfig::new()?,
        })
    }
}
