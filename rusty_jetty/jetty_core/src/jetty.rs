use std::collections::HashMap;
use std::fs;

use anyhow::{anyhow, Result};
use dirs::home_dir;
use serde::Deserialize;
use yaml_peg::serde as yaml;

#[derive(Deserialize, Debug)]
pub struct JettyConfig {
    version: String,
    name: String,
    pub connectors: Vec<ConnectorConfig>,
}

impl JettyConfig {
    pub fn new() -> Result<JettyConfig> {
        let config_raw = fs::read_to_string("./jetty_config.yaml")?;
        let mut config = yaml::from_str::<JettyConfig>(&config_raw)?;

        Ok(config.pop().ok_or(anyhow!["failed"])?)
    }
}

#[derive(Deserialize, Debug)]
pub struct ConnectorConfig {
    namespace: String,
    #[serde(rename = "type")]
    connector_type: String,
    pub config: HashMap<String, String>,
}

pub(crate) type CredentialsBlob = HashMap<String, String>;

pub fn fetch_credentials() -> Result<HashMap<String, CredentialsBlob>> {
    let mut default_path = home_dir().unwrap();

    default_path.push(".jetty/connectors.yaml");

    println!("{:?}", default_path);

    let credentials_raw = fs::read_to_string(default_path)?;
    let mut config = yaml::from_str::<HashMap<String, CredentialsBlob>>(&credentials_raw)?;

    Ok(config.pop().ok_or(anyhow!["failed"])?)
}

pub struct Jetty {
    pub config: JettyConfig,
    // connector_config: HashMap<String, ConnectorCredentials>,
}

impl Jetty {}
