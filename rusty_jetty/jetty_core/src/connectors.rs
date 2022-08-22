use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ConnectorConfig{
    namespace: String, 
    #[serde(rename = "type")]
    connector_type: String,
    config: HashMap<String, String>,
}

pub trait ConnectorCredentials{}