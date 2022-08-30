mod rest;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use jetty_core::{
    connectors::nodes::ConnectorData,
    jetty::{ConnectorConfig, CredentialsBlob},
    Connector,
};
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
#[derive(Deserialize, Default)]
struct TableauConnector {
    config: TableauConfig,
    credentials: TableauCredentials,
    token: Option<String>,
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
            credentials: creds,
            ..Default::default()
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

impl TableauConnector {
    /// Tableau uses credentials to authenticate and then provides an auth
    /// token to authenticate subsequent requests. This fetches that token
    /// and updates it on the TableauConnector.
    #[allow(dead_code)]
    async fn get_token(&mut self) -> Result<String> {
        if let Some(t) = &self.token {
            return Ok(t.to_owned());
        }

        // Set API version. This may eventually belong in the credentials file
        let api_version = "3.4";
        // Set up the request body to get a request token
        let request_body = json!({
            "credentials": {
                "name" : &self.credentials.username,
                "password": &self.credentials.password,
                "site": {
                    "contentUrl": &self.credentials.site_name,
                }
            }
        });
        let client = reqwest::Client::new();
        let resp = client
            .post(format![
                "https://{}/api/{}/auth/signin",
                &self.credentials.server_name, api_version
            ])
            .json(&request_body)
            .header("Accept".to_string(), "application/json".to_string())
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let token = resp
            .get("credentials")
            .ok_or(anyhow!["unable to get token from response"])?
            .get("token")
            .ok_or(anyhow!["unable to get token from response"])?
            .as_str()
            .ok_or(anyhow!["unable to get token from response"])?
            .to_string();
        self.token = Some(token.to_owned());
        Ok(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;
    use jetty_core::jetty;

    #[tokio::test]
    async fn test_fetching_token() -> Result<()> {
        let mut tc = connector_setup().context("running tableau connector setup")?;
        tc.get_token().await?;
        Ok(())
    }

    fn connector_setup() -> Result<TableauConnector> {
        let j = jetty::Jetty::new().context("creating Jetty")?;
        let creds = jetty::fetch_credentials().context("fetching credentials from file")?;
        let config = &j.config.connectors[0];
        let tc = TableauConnector::new(config, &creds["tableau"])
            .context("reading tableau credentials")?;
        Ok(*tc)
    }
}
