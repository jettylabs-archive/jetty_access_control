use std::collections::{HashMap};
use std::boxed::Box;
use crate::{ConnectorConfig, ConnectorCredentials};


pub struct JettyConfig{
    version: String,
    name: String,
    connectors: Vec<ConnectorConfig>
}

impl JettyConfig{
    pub fn new(connector_config:&str)->JettyConfig{
        JettyConfig { version: "2".to_string(), name: "name".to_string(), connectors: vec![]}
    }
}

pub struct Jetty{
    config:JettyConfig,
    connector_config: HashMap<String, Box<dyn ConnectorCredentials>>
}