//! Snowflake Connector
//!
//! Everything needed for connection and interaction with Snowflake.&
//!
//! ```
//! use jetty_core::connectors::Connector;
//! use jetty_core::jetty::{ConnectorConfig, CredentialsBlob};
//! use jetty_snowflake::Snowflake;
//!
//! let config = ConnectorConfig::default();
//! let credentials = CredentialsBlob::default();
//! let snow = Snowflake::new(&config, &credentials);
//! ```

mod consts;
mod entry;

pub use entry::*;

use futures::{stream::FuturesUnordered, StreamExt};
use std::collections::{HashMap, HashSet};
use std::iter::zip;

use jetty_core::{
    connectors,
    connectors::{nodes, Connector},
    jetty::{ConnectorConfig, CredentialsBlob},
};

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use jsonwebtoken::{encode, get_current_timestamp, Algorithm, EncodingKey, Header};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, RequestBuilder};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use structmap::{value::Value, FromMap, GenericMap};

#[macro_use]
extern crate maplit;

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
    http_client: ClientWithMiddleware,
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

    async fn get_data(&self) -> nodes::ConnectorData {
        nodes::ConnectorData {
            groups: self.get_jetty_groups().await.unwrap(),
            users: self.get_jetty_users().await.unwrap(),
            assets: self.get_jetty_assets().await.unwrap(),
            tags: self.get_jetty_tags().await.unwrap(),
            policies: self.get_jetty_policies().await.unwrap(),
        }
    }

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
            Err(anyhow![
                "Snowflake config missing required fields: {:#?}",
                required_fields
            ])
        } else {
            let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
            let client = ClientBuilder::new(reqwest::Client::new())
                .with(RetryTransientMiddleware::new_with_policy(retry_policy))
                .build();
            Ok(Box::new(Snowflake {
                credentials: conn,
                http_client: client,
            }))
        }
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

        Ok(self
            .http_client
            .post(format![
                "https://{}.snowflakecomputing.com/api/v2/statements",
                self.credentials.account
            ])
            .json(&body)
            .header(consts::AUTH_HEADER, format!["Bearer {}", token])
            .header(consts::CONTENT_TYPE_HEADER, "application/json")
            .header(consts::ACCEPT_HEADER, "application/json")
            .header(consts::SNOWFLAKE_AUTH_HEADER, "KEYPAIR_JWT")
            .header(consts::USER_AGENT_HEADER, "jetty-labs"))
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
    pub async fn get_grants_to_user(&self, user_name: &str) -> Result<Vec<RoleGrant>> {
        self.query_to_obj::<RoleGrant>(&format!("SHOW GRANTS TO USER {}", user_name))
            .await
    }

    /// Get all grants to a role – the privileges and "children" roles.
    pub async fn get_grants_to_role(&self, role_name: &str) -> Result<Vec<Grant>> {
        self.query_to_obj::<Grant>(&format!("SHOW GRANTS TO ROLE {}", role_name))
            .await
    }

    /// Get all grants on a role – the "parent" roles.
    pub async fn get_grants_on_role(&self, role_name: &str) -> Result<Vec<Grant>> {
        self.query_to_obj::<Grant>(&format!("SHOW GRANTS ON ROLE {}", role_name))
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
        if result.is_empty() {
            // TODO: Determine whether this is actually okay behavior.
            return Ok(vec![]);
        }
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

    async fn grant_to_policy(&self, role_name: &str, grant: &Grant) -> Result<nodes::Policy> {
        let _granted_to_groups = self
            .get_grants_on_role(role_name)
            .await
            .context(format!("failed to get grants on role {}", &role_name))?;
        Ok(nodes::Policy::new(
            role_name.to_owned(),
            hashset![grant.privilege.clone()],
            // This
            hashset![grant.name.to_owned()],
            hashset![],
            hashset![],
            hashset![],
            false,
            false,
        ))
    }
    async fn get_jetty_policies(&self) -> Result<Vec<nodes::Policy>> {
        let mut res = vec![];
        for role in self.get_roles().await? {
            let mut grants_to_role = self
                .get_grants_to_role(&role.name)
                .await?
                .iter()
                // Ignored types here:
                // ACCOUNT, FUNCTION, WAREHOUSE: These are TODOs for a future iteration.
                // ROLE: We don't need children groups. Those relationships will be taken care of
                // as parent roles.
                .filter(|g| consts::ASSET_TYPES.contains(&g.granted_on.as_str()))
                .map(|g| self.grant_to_policy(&role.name, g))
                .collect::<FuturesUnordered<_>>()
                .collect::<Vec<_>>()
                .await;
            res.append(&mut grants_to_role)
        }
        let res = res.iter().map(|x| x.as_ref().unwrap()).cloned().collect();
        Ok(res)
    }

    async fn get_jetty_tags(&self) -> Result<Vec<nodes::Tag>> {
        Ok(vec![])
    }

    async fn get_jetty_groups(&self) -> Result<Vec<nodes::Group>> {
        let mut res = vec![];
        for role in self.get_roles().await? {
            // TODO: Get members for role.
            res.push(nodes::Group::new(
                role.name,
                hashmap![],
                hashset![],
                hashset![],
                hashset![],
                hashset![],
            ));
        }
        Ok(res)
    }

    async fn get_jetty_users(&self) -> Result<Vec<nodes::User>> {
        let mut res = vec![];
        for user in self.get_users().await? {
            let user_roles = self.get_grants_to_user(&user.name).await?;
            res.push(nodes::User::new(
                user.name,
                hashmap![],
                hashset![],
                hashmap![],
                user_roles.iter().map(|role| role.role.clone()).collect(),
                hashset![],
            ));
        }
        Ok(res)
    }

    async fn get_jetty_assets(&self) -> Result<Vec<nodes::Asset>> {
        let mut res = vec![];
        for table in self.get_tables().await? {
            res.push(nodes::Asset::new(
                format!(
                    "{}.{}.{}",
                    table.database_name, table.schema_name, table.name
                ),
                connectors::AssetType::DBTable,
                hashmap![],
                hashset![],
                hashset![format!("{}.{}", table.database_name, table.schema_name)],
                hashset![],
                hashset![],
                hashset![],
                hashset![],
            ));
        }

        for view in self.get_views().await? {
            res.push(nodes::Asset::new(
                format!("{}.{}.{}", view.database_name, view.schema_name, view.name),
                connectors::AssetType::DBView,
                hashmap![],
                hashset![],
                hashset![format!("{}.{}", view.database_name, view.schema_name)],
                hashset![],
                hashset![],
                hashset![],
                hashset![],
            ));
        }

        let schemas = self.get_schemas().await?;

        for schema in schemas {
            // TODO: Get subassets
            res.push(nodes::Asset::new(
                format!("{}.{}", schema.database_name, schema.name),
                connectors::AssetType::DBSchema,
                hashmap![],
                hashset![],
                hashset![schema.database_name],
                hashset![],
                hashset![],
                hashset![],
                hashset![],
            ));
        }

        for db in self.get_databases().await? {
            // TODO: Get subassets
            res.push(nodes::Asset::new(
                db.name,
                connectors::AssetType::DBDB,
                hashmap![],
                hashset![],
                hashset![],
                hashset![],
                hashset![],
                hashset![],
                hashset![],
            ));
        }

        Ok(res)
    }
}
