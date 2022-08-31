mod nodes;
mod rest;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use jetty_core::{
    connectors::nodes::ConnectorData,
    jetty::{ConnectorConfig, CredentialsBlob},
    Connector,
};
use rest::TableauRestClient;
use serde::Deserialize;
use serde_json::json;
use std::collections::{HashMap, HashSet};

type TableauConfig = HashMap<String, String>;

/// Credentials for authenticating with Tableau.
///
/// The user sets these up by following Jetty documentation
/// and pasting ther connection info into their connector config.
#[derive(Deserialize, Debug, Default)]
struct TableauCredentials {
    username: String,
    password: String,
    server_name: String,
    site_name: String,
}

#[allow(dead_code)]
#[derive(Default)]
struct TableauConnector {
    client: TableauRestClient,
    config: TableauConfig,
}

#[async_trait]
impl Connector for TableauConnector {
    /// Validates the configs and bootstraps a Tableu connection.
    ///
    /// Validates that the required fields are present to authenticate to
    /// Tableau. Stashes the credentials in the struct for use when
    /// connecting.
    fn new(config: &ConnectorConfig, credentials: &CredentialsBlob) -> Result<Box<Self>> {
        let mut creds = TableauCredentials::default();
        let mut required_fields =
            HashSet::from(["server_name", "site_name", "username", "password"]);

        for (k, v) in credentials.iter() {
            match k.as_ref() {
                "server_name" => creds.server_name = v.to_string(),
                "site_name" => creds.site_name = v.to_string(),
                "username" => creds.username = v.to_string(),
                "password" => creds.password = v.to_string(),
                _ => (),
            }

            required_fields.remove::<str>(k);
        }

        if !required_fields.is_empty() {
            return Err(anyhow![
                "Snowflake config missing required fields: {:#?}",
                required_fields
            ]);
        }

        let tableau_connector = TableauConnector {
            config: config.config.to_owned(),
            client: TableauRestClient::new(creds),
        };

        Ok(Box::new(tableau_connector))
    }

    async fn check(&self) -> bool {
        todo!()
    }

    async fn get_data(&self) -> ConnectorData {
        todo!()
    }
}
