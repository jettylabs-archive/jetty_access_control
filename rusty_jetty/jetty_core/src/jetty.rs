use std::fs::read_to_string;
use std::collections::{HashMap};
use std::boxed::Box;
use crate::{ConnectorConfig, ConnectorCredentials};

use serde::Deserialize;
use anyhow::{anyhow, Result};
use yaml_peg::serde::from_str;
use dirs::home_dir;

#[derive(Deserialize, Debug)]
pub struct JettyConfig{
    version: String,
    name: String,
    connectors: Vec<ConnectorConfig>
}

impl JettyConfig{
    pub fn new()->Result<JettyConfig>{
        let config_raw = read_to_string("./jetty_config.yaml")?;
        let mut config= from_str::<JettyConfig>(&config_raw)?;
        
        
        Ok(config.pop().ok_or(anyhow!["failed"])?)
        // let mut default_path = home_dir().unwrap();

        // default_path.push(".jetty/connectors.yaml");
        // let config_raw = read_to_string(default_path)?;
        // let mut config= from_str::<Vec<dyn ConnectorCredentials>>(&config_raw)?;
        // println!("config:");
        // println!("{:#?}", config);
        
        
        // Ok(config.pop().ok_or(anyhow!["failed"])?)
    }
}

pub struct Jetty{
    config:JettyConfig,
    connector_config: HashMap<String, Box<dyn ConnectorCredentials>>
}