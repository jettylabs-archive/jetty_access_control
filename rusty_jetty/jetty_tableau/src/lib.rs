mod fetch;
mod nodes;
mod nodes2;
mod rest;

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use jetty_core::{
    connectors::{nodes::ConnectorData, ConnectorClient},
    jetty::{ConnectorConfig, CredentialsBlob},
    Connector,
};
use rest::TableauRestClient;
use serde::Deserialize;
use serde_json::json;
use std::{
    collections::{HashMap, HashSet},
    fs, io,
};

type TableauConfig = HashMap<String, String>;

/// Credentials for authenticating with Tableau.
///
/// The user sets these up by following Jetty documentation
/// and pasting ther connection info into their connector config.
#[derive(Deserialize, Debug, Default)]
struct TableauCredentials {
    username: String,
    password: String,
    /// Tableau server name like 10ay.online.tableau.com *without* the `https://`
    server_name: String,
    site_name: String,
}

#[derive(Default, Deserialize)]
pub(crate) struct TableauEnvironment {
    pub users: HashMap<String, nodes2::User>,
    pub groups: HashMap<String, nodes2::Group>,
    pub projects: HashMap<String, nodes2::Project>,
    pub datasources: HashMap<String, nodes2::Datasource>,
    pub data_connections: HashMap<String, nodes2::DataConnection>,
    pub flows: HashMap<String, nodes2::Flow>,
    pub lenses: HashMap<String, nodes2::Lens>,
    pub metrics: HashMap<String, nodes2::Metric>,
    pub views: HashMap<String, nodes2::View>,
    pub workbooks: HashMap<String, nodes2::Workbook>,
}

#[allow(dead_code)]
#[derive(Default)]
struct TableauConnector {
    rest_client: TableauRestClient,
    config: TableauConfig,
    env: TableauEnvironment,
}

#[async_trait]
impl Connector for TableauConnector {
    /// Validates the configs and bootstraps a Tableu connection.
    ///
    /// Validates that the required fields are present to authenticate to
    /// Tableau. Stashes the credentials in the struct for use when
    /// connecting.
    fn new(
        config: &ConnectorConfig,
        credentials: &CredentialsBlob,
        _client: Option<ConnectorClient>,
    ) -> Result<Box<Self>> {
        dbg!(credentials);
        let mut creds = TableauCredentials::default();
        let mut required_fields = HashSet::from([
            "server_name".to_owned(),
            "site_name".to_owned(),
            "username".to_owned(),
            "password".to_owned(),
        ]);

        for (k, v) in credentials.iter() {
            match k.as_ref() {
                "server_name" => creds.server_name = v.to_string(),
                "site_name" => creds.site_name = v.to_string(),
                "username" => creds.username = v.to_string(),
                "password" => creds.password = v.to_string(),
                _ => (),
            }

            required_fields.remove(k);
        }

        if !required_fields.is_empty() {
            return Err(anyhow![
                "Snowflake config missing required fields: {:#?}",
                required_fields
            ]);
        }

        let tableau_connector = TableauConnector {
            config: config.config.to_owned(),
            rest_client: TableauRestClient::new(creds),
            env: read_env().unwrap_or_default(),
        };

        Ok(Box::new(tableau_connector))
    }

    async fn check(&self) -> bool {
        todo!()
    }

    async fn get_data(&mut self) -> ConnectorData {
        todo!()
    }
}

fn read_env() -> Result<TableauEnvironment> {
    // Open the file in read-only mode with buffer.
    let file = fs::File::open("tableau_env.json").context("opening environment file")?;
    let reader = io::BufReader::new(file);

    let e = serde_json::from_reader(reader).context("parsing environment")?;

    // Return the `Environment`.
    Ok(e)
}

#[cfg(test)]
pub(crate) fn connector_setup() -> Result<crate::TableauConnector> {
    use anyhow::Context;
    use jetty_core::Connector;

    let j = jetty_core::jetty::Jetty::new().context("creating Jetty")?;
    let creds = jetty_core::jetty::fetch_credentials().context("fetching credentials from file")?;
    let config = &j.config.connectors[0];
    let tc = crate::TableauConnector::new(config, &creds["tableau"], None)
        .context("reading tableau credentials")?;
    Ok(*tc)
}
