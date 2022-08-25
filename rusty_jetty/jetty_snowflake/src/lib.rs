//! Snowflake Connector
//!
//! Everything needed for connection and interaction with Snowflake.&
//!
//! ```
//! use jetty_core::snowflake::Snowflake;
//! use jetty_core::connectors::Connector;
//! use jetty_core::jetty::{ConnectorConfig, CredentialsBlob};
//!
//! let config = ConnectorConfig::default();
//! let credentials = CredentialsBlob::default();
//! let snow = Snowflake::new(&config, &credentials);
//! ```

mod consts;
mod database;
mod grant;
mod role;
mod schema;
mod table;
mod user;
mod view;
mod warehouse;

pub use database::Database;
pub use grant::Grant;
pub use role::Role;
pub use schema::Schema;
pub use table::Table;
pub use user::User;
pub use view::View;
pub use warehouse::Warehouse;

use jetty_core::{
    connectors::Connector,
    jetty::{ConnectorConfig, CredentialsBlob},
};

use std::collections::{HashMap, HashSet};
use std::iter::zip;

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use jsonwebtoken::{encode, get_current_timestamp, Algorithm, EncodingKey, Header};
use reqwest::RequestBuilder;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use structmap::{value::Value, FromMap, GenericMap};

#[derive(Deserialize, Debug)]
struct SnowflakeField {
    #[serde(default)]
    name: String,
}

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

/// Main connector implementation.
#[async_trait]
impl Connector for Snowflake {
    /// Validates the configs and bootstraps a Snowflake connection.
    ///
    /// Validates that the required fields are present to authenticate to
    /// Snowflake. Stashes the credentials in the struct for use when
    /// connecting.
    fn new(_config: &ConnectorConfig, credentials: &CredentialsBlob) -> Result<Box<Self>> {
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
        let res = self.execute("SELECT 1").await;
        return match res {
            Err(e) => {
                println!("{:?}", e);
                false
            }
            Ok(_) => true,
        };
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
        let claims = JwtClaims {
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
                    .replace(' ', "")
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
            .header(consts::AUTH_HEADER, format!["Bearer {}", token])
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("X-Snowflake-Authorization-Token-Type", "KEYPAIR_JWT")
            .header("User-Agent", "jetty-labs"))
    }

    /// Execute a query, dropping the result.
    ///
    /// `execute` should only be used for
    /// SQL statements that don't expect results,
    /// such as those that are used to update
    /// state in Snowflake.
    async fn execute(&self, sql: &str) -> Result<()> {
        let request = self.get_request(sql)?;
        request.send().await?.text().await?;
        Ok(())
    }

    async fn query(&self, sql: &str) -> Result<String> {
        let request = self.get_request(sql)?;

        let res = request.send().await?.text().await?;
        Ok(res)
    }

    /// Get all grants to a user
    pub async fn get_grants_to_user(&self, user_name: &str) -> Result<Vec<Grant>> {
        self.query_to_obj::<Grant>(&format!("SHOW GRANTS TO USER {}", user_name))
            .await
    }

    /// Get all grants to a role
    pub async fn get_grants_to_role(&self, role_name: &str) -> Result<Vec<Grant>> {
        self.query_to_obj::<Grant>(&format!("SHOW GRANTS TO ROLE {}", role_name))
            .await
    }

    /// Get all users.
    pub async fn get_users(&self) -> Result<Vec<User>> {
        self.query_to_obj::<User>("SHOW USERS").await
    }

    /// Get all roles.
    pub async fn get_roles(&self) -> Result<Vec<Role>> {
        self.query_to_obj::<Role>("SHOW ROLES").await
    }

    /// Get all databases.
    pub async fn get_databases(&self) -> Result<Vec<Database>> {
        self.query_to_obj::<Database>("SHOW DATABASES").await
    }

    /// Get all warehouses.
    pub async fn get_warehouses(&self) -> Result<Vec<Warehouse>> {
        self.query_to_obj::<Warehouse>("SHOW WAREHOUSES").await
    }

    /// Get all schemas.
    pub async fn get_schemas(&self) -> Result<Vec<Schema>> {
        self.query_to_obj::<Schema>("SHOW SCHEMAS").await
    }

    /// Get all views.
    pub async fn get_views(&self) -> Result<Vec<View>> {
        self.query_to_obj::<View>("SHOW VIEWS").await
    }

    /// Get all tables.
    pub async fn get_tables(&self) -> Result<Vec<Table>> {
        self.query_to_obj::<Table>("SHOW TABLES").await
    }

    /// Execute the given query and deserialize the result into the given type.
    pub async fn query_to_obj<T>(&self, query: &str) -> Result<Vec<T>>
    where
        T: FromMap,
    {
        let result = self.query(query).await.context("query failed")?;
        let rows_value: JsonValue =
            serde_json::from_str(&result).context("failed to deserialize")?;
        let rows_data = rows_value["data"].clone();
        let rows: Vec<Vec<Value>> = serde_json::from_value::<Vec<Vec<Option<String>>>>(rows_data)
            .context("failed to deserialize rows")?
            .iter()
            .map(|i| {
                i.iter()
                    .map(|x| Value::new(x.clone().unwrap_or_default()))
                    .collect()
            })
            .collect();
        let fields_intermediate: Vec<SnowflakeField> =
            serde_json::from_value(rows_value["resultSetMetaData"]["rowType"].clone())
                .context("failed to deserialize fields")?;
        let fields: Vec<String> = fields_intermediate.iter().map(|i| i.name.clone()).collect();
        println!("fields: {:?}", fields);
        Ok(rows
            .iter()
            .map(|i| {
                // Zip field - i
                let map: GenericMap = zip(fields.clone(), i.clone()).collect();
                map
            })
            .map(|i| T::from_genericmap(i))
            .collect())
    }
}
