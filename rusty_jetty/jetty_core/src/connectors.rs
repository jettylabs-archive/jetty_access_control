use std::collections::HashMap;

pub struct ConnectorConfig{
    namespace: String, 
    connector_type: String,
    config: HashMap<String, String>,
}

pub trait ConnectorCredentials{}