use super::*;
use reqwest;

/// Wrapper struct for http functionality
#[derive(Default)]
pub(crate) struct TableauRestClient {
    /// The credentials used to authenticate into Snowflake.
    credentials: TableauCredentials,
    http_client: reqwest::Client,
    token: Option<String>,
}

impl TableauRestClient {
    pub fn new(credentials: TableauCredentials) -> Self {
        TableauRestClient {
            credentials,
            http_client: reqwest::Client::new(),
            token: None,
        }
    }

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
        let resp = self
            .http_client
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
        tc.client.get_token().await?;
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
