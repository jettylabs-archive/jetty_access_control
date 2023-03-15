//! Rest API interface for Snowflake
//!

use crate::{consts, creds::SnowflakeCredentials};

use anyhow::{Context, Result};
use jetty_core::logging::{debug, error};
use jsonwebtoken::{encode, get_current_timestamp, Algorithm, EncodingKey, Header};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, RequestBuilder};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::{Deserialize, Serialize};

use std::{collections::HashMap, thread, time::Duration};

/// Claims for use with the `jsonwebtoken` crate when
/// creating a new JWT.
#[derive(Debug, Serialize, Deserialize)]
struct JwtClaims {
    /// Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    exp: usize,
    /// Optional. Issued at (as UTC timestamp)
    iat: usize,
    /// Optional. Issuer
    iss: String,
    /// Optional. Subject (whom token refers to)
    sub: String,
}

#[derive(Default)]
pub struct SnowflakeRequestConfig {
    pub sql: String,
    /// Only used to bypass JWT logic in testing
    pub use_jwt: bool,
}

#[derive(Default)]
pub(crate) struct SnowflakeRestConfig {
    /// Enable/disable retry logic.
    pub(crate) retry: bool,
}
/// Wrapper struct for http functionality
pub(crate) struct SnowflakeRestClient {
    /// The credentials used to authenticate into Snowflake.
    credentials: SnowflakeCredentials,
    http_client: ClientWithMiddleware,
}

impl SnowflakeRestClient {
    pub(crate) fn new(
        credentials: SnowflakeCredentials,
        config: SnowflakeRestConfig,
    ) -> Result<Self> {
        credentials.validate()?;
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(4);
        let mut client_builder = ClientBuilder::new(reqwest::Client::new());
        if config.retry {
            client_builder =
                client_builder.with(RetryTransientMiddleware::new_with_policy(retry_policy))
        }
        let client = client_builder.build();
        Ok(Self {
            credentials,
            http_client: client,
        })
    }
    /// Execute a query, dropping the result.
    ///
    /// `execute` should only be used for
    /// SQL statements that don't expect results,
    /// such as those that are used to update
    /// state in Snowflake.
    pub(crate) async fn execute(&self, config: &SnowflakeRequestConfig) -> Result<()> {
        let request = self.get_request(config)?;
        request
            .send()
            .await
            .context("couldn't send request")?
            .error_for_status()?;
        Ok(())
    }

    pub(crate) async fn query(&self, config: &SnowflakeRequestConfig) -> Result<String> {
        debug!("starting query: {:?}", &config.sql);
        #[derive(Deserialize)]
        struct AcceptedResponse {
            #[serde(rename = "statementHandle")]
            statement_handle: String,
            code: String,
        }
        let request = self
            .get_request(config)
            .context(format!("failed to get request for query {:?}", &config.sql))?;

        let response = request
            .send()
            .await
            .context("couldn't send request")?
            .error_for_status()
            .map_err(|e| {
                error!("error status for query: {} -- error: {}", &config.sql, &e);
                e
            })?;

        let mut res = response.text().await.context("couldn't get body text")?;

        while serde_json::from_str::<AcceptedResponse>(&res)
            .map(|r| r.code == "333334")
            .unwrap_or(false)
        {
            thread::sleep(Duration::from_millis(1500));
            let statement_handle = serde_json::from_str::<AcceptedResponse>(&res)?.statement_handle;

            let request = self.get_status_check_request(config, statement_handle)?;
            res = request
                .send()
                .await
                .context("couldn't send request")?
                .error_for_status()?
                .text()
                .await
                .context("couldn't get body text")?;
        }

        debug!("completed query: {:?}", &config.sql);
        Ok(res)
    }

    /// If the URL is explicitly defined, that's used first.
    /// Otherwise, the standard account configuration
    /// is used
    fn get_url(&self) -> String {
        self.credentials.url.to_owned().unwrap_or_else(|| {
            format![
                "https://{}.snowflakecomputing.com/api/v2/statements",
                self.credentials.account
            ]
        })
    }

    fn get_request(&self, config: &SnowflakeRequestConfig) -> Result<RequestBuilder> {
        let body = self.get_body(&config.sql);

        let mut builder = self
            .http_client
            .post(self.get_url())
            .json(&body)
            .header(consts::CONTENT_TYPE_HEADER, "application/json")
            .header(consts::ACCEPT_HEADER, "application/json")
            .header(consts::SNOWFLAKE_AUTH_HEADER, "KEYPAIR_JWT")
            .header(consts::USER_AGENT_HEADER, "jetty-labs");
        if config.use_jwt {
            let token = self.get_jwt().context("failed to get jwt")?;
            builder = builder.header(consts::AUTH_HEADER, format!["Bearer {token}"]);
        }
        Ok(builder)
    }

    fn get_status_check_request(
        &self,
        config: &SnowflakeRequestConfig,
        statement_handle: String,
    ) -> Result<RequestBuilder> {
        let mut builder = self
            .http_client
            .get(format!("{}/{statement_handle}", self.get_url()))
            .header(consts::CONTENT_TYPE_HEADER, "application/json")
            .header(consts::ACCEPT_HEADER, "application/json")
            .header(consts::SNOWFLAKE_AUTH_HEADER, "KEYPAIR_JWT")
            .header(consts::USER_AGENT_HEADER, "jetty-labs");
        if config.use_jwt {
            let token = self.get_jwt().context("failed to get jwt")?;
            builder = builder.header(consts::AUTH_HEADER, format!["Bearer {token}"]);
        }
        Ok(builder)
    }

    pub(crate) fn get_partition(
        &self,
        config: &SnowflakeRequestConfig,
        statement_handle: &str,
        partition_number: usize,
    ) -> Result<RequestBuilder> {
        let mut builder = self
            .http_client
            .get(format!("{}/{statement_handle}", self.get_url()))
            .query(&[("partition", partition_number)])
            .header(consts::CONTENT_TYPE_HEADER, "application/json")
            .header(consts::ACCEPT_HEADER, "application/json")
            .header(consts::SNOWFLAKE_AUTH_HEADER, "KEYPAIR_JWT")
            .header(consts::USER_AGENT_HEADER, "jetty-labs");
        if config.use_jwt {
            let token = self.get_jwt().context("failed to get jwt")?;
            builder = builder.header(consts::AUTH_HEADER, format!["Bearer {token}"]);
        }
        Ok(builder)
    }

    fn get_body<'a>(&'a self, sql: &'a str) -> HashMap<&str, &'a str> {
        let mut body = HashMap::new();
        body.insert("statement", sql);
        body.insert("warehouse", &self.credentials.warehouse);
        body.insert("role", &self.credentials.role);
        body
    }

    fn get_jwt(&self) -> Result<String> {
        {
            let qualified_username = format![
                "{}.{}",
                self.credentials.account.split('.').collect::<Vec<_>>()[0].to_uppercase(),
                self.credentials.user.to_uppercase()
            ];

            // Generate jwt
            let claims = JwtClaims {
                exp: (get_current_timestamp() + 3600) as usize,
                iat: get_current_timestamp() as usize,
                iss: format!["{qualified_username}.{}", self.credentials.public_key_fp],
                sub: qualified_username,
            };

            encode(
                &Header::new(Algorithm::RS256),
                &claims,
                &EncodingKey::from_rsa_pem(
                    self.credentials
                        .private_key
                        .replace(' ', "")
                        .replace("ENDPRIVATEKEY", "END PRIVATE KEY")
                        .replace("BEGINPRIVATEKEY", "BEGIN PRIVATE KEY")
                        .as_bytes(),
                )?,
            )
            .map_err(anyhow::Error::from)
        }
    }

    /// Get the snowflake user used for queries
    pub(crate) fn get_snowflake_role(&self) -> String {
        self.credentials.role.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use jetty_core::logging::debug;
    use wiremock::matchers::{body_string_contains, method, path};
    use wiremock::{Mock, MockGuard, MockServer, ResponseTemplate};

    pub struct WiremockServer {
        pub server: Option<MockServer>,
    }

    impl WiremockServer {
        pub fn new() -> Self {
            Self { server: None }
        }

        pub async fn init(&mut self) {
            let mock_server = MockServer::start().await;
            self.server = Some(mock_server);
        }
    }

    async fn mount_default_guard(server: &WiremockServer) -> MockGuard {
        Mock::given(method("POST"))
            .and(path("/api/v2/statements"))
            .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"text": "wiremock"}"#))
            .named("execute_does_not_panic")
            .mount_as_scoped(server.server.as_ref().unwrap())
            .await
    }

    #[tokio::test]
    #[should_panic]
    async fn empty_creds_fails_to_load() {
        SnowflakeRestClient::new(
            SnowflakeCredentials::default(),
            SnowflakeRestConfig::default(),
        )
        .unwrap();
    }

    #[tokio::test]
    async fn filled_creds_create_client_successfully() {
        let creds = SnowflakeCredentials {
            account: "my_account".to_owned(),
            role: "role".to_owned(),
            user: "user".to_owned(),
            warehouse: "warehouse".to_owned(),
            private_key: "private_key".to_owned(),
            public_key_fp: "fp".to_owned(),
            url: None,
        };
        SnowflakeRestClient::new(creds, SnowflakeRestConfig::default()).unwrap();
    }

    #[tokio::test]
    async fn execute_does_not_panic() {
        let mut server = WiremockServer::new();
        server.init().await;
        let guard = mount_default_guard(&server).await;
        let creds = SnowflakeCredentials {
            account: "my_account".to_owned(),
            role: "role".to_owned(),
            user: "user".to_owned(),
            warehouse: "warehouse".to_owned(),
            private_key: "private_key".to_owned(),
            public_key_fp: "fp".to_owned(),
            url: Some(format!(
                "{}/api/v2/statements",
                server.server.as_ref().unwrap().uri()
            )),
        };
        let client = SnowflakeRestClient::new(creds, SnowflakeRestConfig::default()).unwrap();
        client
            .execute(&SnowflakeRequestConfig {
                sql: "select 1".to_owned(),
                use_jwt: false,
            })
            .await
            .unwrap();
        drop(guard);
    }

    #[tokio::test]
    async fn query_does_not_panic() {
        let mut server = WiremockServer::new();
        server.init().await;
        let guard = mount_default_guard(&server).await;
        let creds = SnowflakeCredentials {
            account: "my_account".to_owned(),
            role: "role".to_owned(),
            user: "user".to_owned(),
            warehouse: "warehouse".to_owned(),
            private_key: "private_key".to_owned(),
            public_key_fp: "fp".to_owned(),
            url: Some(format!(
                "{}/api/v2/statements",
                server.server.as_ref().unwrap().uri()
            )),
        };
        let client = SnowflakeRestClient::new(creds, SnowflakeRestConfig::default()).unwrap();
        client
            .query(&SnowflakeRequestConfig {
                sql: "select 1".to_owned(),
                use_jwt: false,
            })
            .await
            .unwrap();
        drop(guard);
    }

    #[tokio::test]
    #[should_panic]
    async fn server_error_panics() {
        let mut server = WiremockServer::new();
        server.init().await;
        // We will use a custom guard for this one to mock a bad response (500).
        let guard = Mock::given(method("POST"))
            .and(path("/api/v2/statements"))
            .and(body_string_contains("select 2"))
            .respond_with(ResponseTemplate::new(500).set_body_string(r#"{"text": "wiremock"}"#))
            .named("500 server error")
            .mount_as_scoped(server.server.as_ref().unwrap())
            .await;

        let creds = SnowflakeCredentials {
            account: "my_account".to_owned(),
            role: "role".to_owned(),
            user: "user".to_owned(),
            warehouse: "warehouse".to_owned(),
            private_key: "private_key".to_owned(),
            public_key_fp: "fp".to_owned(),
            url: Some(format!(
                "{}/api/v2/statements",
                server.server.as_ref().unwrap().uri()
            )),
        };
        let client = SnowflakeRestClient::new(creds, SnowflakeRestConfig::default()).unwrap();
        let res = client
            .query(&SnowflakeRequestConfig {
                sql: "select 2".to_owned(),
                use_jwt: false,
            })
            .await
            .context("query failed")
            .unwrap();
        debug!("query: {:?}", res);
        drop(guard);
    }
}
