//! Snowflake Connector
//!
//! Everything needed for connection and interaction with Snowflake.
//!
//! ```
//! use jetty_core::snowflake::Snowflake;
//! use jetty_core::connectors::Connector;
//! use jetty_core::jetty::{ConnectorConfig, CredentialsBlob};
//!
//! fn main(){
//!     let config = ConnectorConfig::default();
//!     let credentials = CredentialsBlob::default();
//!     let snow = Snowflake::new(&config, &credentials);
//! }
//! ```

use crate::{
    connectors::Connector,
    jetty::{ConnectorConfig, CredentialsBlob},
};

use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use jsonwebtoken::{encode, get_current_timestamp, Algorithm, EncodingKey, Header};
use reqwest;
use reqwest::RequestBuilder;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const AUTH_HEADER: &str = "Authorization";

/// The main Snowflake Connector struct.
///
/// Use this connector to access Snowflake data.
pub struct Snowflake {
    /// The credentials used to authenticate into Snowflake.
    credentials: SnowflakeCredentials,
}

/// Credentials for authenticating to Snowflake.
///
/// The user sets these up by following Jetty documentation
/// and pasting their keys into their connector config.
#[derive(Deserialize, Debug, Default)]
struct SnowflakeCredentials {
    account: String,
    password: String,
    role: String,
    user: String,
    warehouse: String,
    private_key: String,
    public_key_fp: String,
}

/// Snowflake Role entry.
#[derive(Deserialize, Debug)]
pub struct Role {
    #[serde(skip_serializing)]
    _created_on_unused: String,
    /// The role name in Snowflake.
    pub name: String,
    #[serde(skip_serializing)]
    _is_default_unused: String,
    #[serde(skip_serializing)]
    _is_current_unused: String,
    #[serde(skip_serializing)]
    _is_inherited_unused: String,
    #[serde(skip_serializing)]
    _assigned_to_users_unused: String,
    #[serde(skip_serializing)]
    _granted_to_roles_unused: String,
    #[serde(skip_serializing)]
    _granted_roles_unused: String,
    #[serde(skip_serializing)]
    _owner_unused: String,
    #[serde(skip_serializing)]
    _comment_unused: String,
}

/// Claims for use with the `jsonwebtoken` crate when
/// creating a new JWT.
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    /// Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    exp: usize,
    /// Optional. Issued at (as UTC timestamp)
    iat: usize,
    /// Optional. Issuer
    iss: String,
    /// Optional. Subject (whom token refers to)
    sub: String,
}

/// Main connector implementation.
#[async_trait]
impl Connector for Snowflake {
    /// Validates the configs and bootstraps a Snowflake connection.
    ///
    /// Validates that the required fields are present to authenticate to
    /// Snowflake. Stashes the credentials in the struct for use when
    /// connecting.
    fn new(config: &ConnectorConfig, credentials: &CredentialsBlob) -> Result<Box<Self>> {
        let mut conn = SnowflakeCredentials::default();
        let mut required_fields: HashSet<_> = vec![
            "account",
            "password",
            "role",
            "user",
            "warehouse",
            "private_key",
            "public_key_fp",
        ]
        .into_iter()
        .collect();

        for (k, v) in credentials.iter() {
            match k.as_ref() {
                "account" => conn.account = v.to_string(),
                "password" => conn.password = v.to_string(),
                "role" => conn.role = v.to_string(),
                "user" => conn.user = v.to_string(),
                "warehouse" => conn.warehouse = v.to_string(),
                "private_key" => conn.private_key = v.to_string(),
                "public_key_fp" => conn.public_key_fp = v.to_string(),
                _ => (),
            }

            required_fields.remove::<str>(k);
        }

        if !required_fields.is_empty() {
            Err(anyhow![format![
                "Snowflake config missing required fields: {:#?}",
                required_fields
            ]])
        } else {
            Ok(Box::new(Snowflake { credentials: conn }))
        }
    }

    async fn check(&self) -> bool {
        let res = self.execute("SELECT 1").await.unwrap();
        true
        // return match res {
        //     Err(e) => {
        //         println!("{:?}", e);
        //         false
        //     }
        //     Ok(_) => true,
        // };
    }
}

impl Snowflake {
    fn get_jwt(&self) -> Result<String> {
        let qualified_username = format![
            "{}.{}",
            self.credentials.account.to_uppercase(),
            self.credentials.user.to_uppercase()
        ];

        // Generate jwt
        let claims = Claims {
            exp: (get_current_timestamp() + 3600) as usize,
            iat: get_current_timestamp() as usize,
            iss: format!["{}.{}", qualified_username, self.credentials.public_key_fp],
            sub: qualified_username,
        };

        // println!("{}", self.credentials.private_key.replace(r" ", ""));

        encode(
            &Header::new(Algorithm::RS256),
            &claims,
            &EncodingKey::from_rsa_pem(
                self.credentials
                    .private_key
                    .replace(r" ", "")
                    .replace("ENDPRIVATEKEY", "END PRIVATE KEY")
                    .replace("BEGINPRIVATEKEY", "BEGIN PRIVATE KEY")
                    .as_bytes(),
            )?,
        )
        .map_err(anyhow::Error::from)
    }

    fn get_body<'a>(&'a self, sql: &'a str) -> HashMap<&str, &'a str> {
        let mut body = HashMap::new();
        body.insert("statement", sql);
        body.insert("warehouse", "main");
        body.insert("role", &self.credentials.role);
        body
    }

    fn get_request(&self, sql: &str) -> Result<RequestBuilder> {
        let token = self.get_jwt()?;
        let body = self.get_body(sql);

        let client = reqwest::Client::new();

        Ok(client
            .post(format![
                "https://{}.snowflakecomputing.com/api/v2/statements",
                self.credentials.account
            ])
            .json(&body)
            .header(AUTH_HEADER, format!["Bearer {}", token])
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("X-Snowflake-Authorization-Token-Type", "KEYPAIR_JWT")
            .header("User-Agent", "test"))
    }

    /// Execute a query, dropping the result.
    ///
    /// `execute` should only be used for
    /// SQL statements that don't expect results,
    /// such as those that are used to update
    /// state in Snowflake.
    async fn execute(&self, sql: &str) -> Result<()> {
        let request = self.get_request(sql)?;

        let res = request.send().await?.text().await;
        println!["{:#?}", res];
        Ok(())
    }

    async fn query(&self, sql: &str) -> Result<String> {
        let request = self.get_request(sql)?;

        let res = request.send().await?.text().await?;
        Ok(res)
    }

    /// Get all roles contained within the account.
    pub async fn get_roles(&self) -> Result<Vec<Role>> {
        let result = self.query("SHOW ROLES").await?;
        println!("{:#?}", result);
        let roles_value: Value = serde_json::from_str(&result)?;
        let roles_data = roles_value["data"].clone();
        let roles = serde_json::from_value(roles_data)?;
        Ok(roles)
    }
}
