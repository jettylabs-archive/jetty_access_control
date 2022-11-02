//! Snowflake Connector
//!
//! Everything needed for connection and interaction with Snowflake.&
//!
//! ```
//! use std::path::PathBuf;
//! use jetty_core::connectors::{Connector, ConnectorClient, NewConnector};
//! use jetty_core::jetty::{ConnectorConfig, CredentialsMap};
//! use jetty_snowflake::SnowflakeConnector;
//!
//! let config = ConnectorConfig::default();
//! let credentials = CredentialsMap::default();
//! let connector_client = ConnectorClient::Core;
//! let snow = SnowflakeConnector::new(&config, &credentials, Some(connector_client), None);
//! ```

mod consts;
mod coordinator;
mod creds;
mod cual;
mod efperm;
mod entry_types;
mod rest;

use cual::set_cual_account_name;
pub use entry_types::{
    Asset, Database, Entry, FutureGrant, Grant, GrantOf, GrantType, Object, Role, RoleName, Schema,
    StandardGrant, Table, User, View, Warehouse,
};
use jetty_core::connectors::NewConnector;
use jetty_core::logging::error;
use rest::{SnowflakeRequestConfig, SnowflakeRestClient, SnowflakeRestConfig};
use serde::de::value::MapDeserializer;

use std::collections::{HashMap, HashSet};
use std::iter::zip;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use jetty_core::{
    connectors,
    connectors::{nodes, Connector},
    jetty::{ConnectorConfig, CredentialsMap},
};

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value as JsonValue;

/// The main Snowflake Connector struct.
///
/// Use this connector to access Snowflake data.
pub struct SnowflakeConnector {
    rest_client: SnowflakeRestClient,
    client: connectors::ConnectorClient,
}

#[derive(Deserialize, Debug)]
struct SnowflakeField {
    #[serde(default)]
    name: String,
}

#[async_trait]
impl NewConnector for SnowflakeConnector {
    /// Validates the configs and bootstraps a Snowflake connection.
    ///
    /// Validates that the required fields are present to authenticate to
    /// Snowflake. Stashes the credentials in the struct for use when
    /// connecting.
    async fn new(
        _config: &ConnectorConfig,
        credentials: &CredentialsMap,
        connector_client: Option<connectors::ConnectorClient>,
        _data_dir: Option<PathBuf>,
    ) -> Result<Box<Self>> {
        let mut conn = creds::SnowflakeCredentials::default();
        let mut required_fields: HashSet<_> = vec![
            "account",
            "role",
            "user",
            "warehouse",
            "private_key",
            "public_key_fp",
            // "url" // URL not required – defaults to typical account URL.
        ]
        .into_iter()
        .collect();

        for (k, v) in credentials.iter() {
            match k.as_ref() {
                "account" => conn.account = v.to_string(),
                "role" => conn.role = v.to_string(),
                "user" => conn.user = v.to_string(),
                "warehouse" => conn.warehouse = v.to_string(),
                "private_key" => conn.private_key = v.to_string(),
                "public_key_fp" => conn.public_key_fp = v.to_string(),
                "url" => conn.url = Some(v.to_string()),
                _ => (),
            }

            required_fields.remove::<str>(k);
        }
        set_cual_account_name(&conn.account);

        if !required_fields.is_empty() {
            Err(anyhow![
                "Snowflake config missing required fields: {:#?}",
                required_fields
            ])
        } else {
            let client = connector_client.unwrap_or(connectors::ConnectorClient::Core);
            Ok(Box::new(SnowflakeConnector {
                client,
                rest_client: SnowflakeRestClient::new(conn, SnowflakeRestConfig { retry: true })?,
            }))
        }
    }
}

/// Main connector implementation.
#[async_trait]
impl Connector for SnowflakeConnector {
    async fn check(&self) -> bool {
        let res = self
            .rest_client
            .execute(&SnowflakeRequestConfig {
                sql: "SELECT 1".to_string(),
                use_jwt: true,
            })
            .await;
        return match res {
            Err(e) => {
                error!("{:?}", e);
                false
            }
            Ok(_) => true,
        };
    }

    async fn get_data(&mut self) -> nodes::ConnectorData {
        // Fetch Snowflake Environment
        let mut c = coordinator::Coordinator::new(self);
        c.get_data().await
    }
}

impl SnowflakeConnector {
    /// Get all grants to a role – the privileges and "children" roles.
    pub(crate) async fn get_grants_to_role_future(
        &self,
        role: &Role,
        target: Arc<Mutex<&mut Vec<StandardGrant>>>,
    ) -> Result<()> {
        let RoleName(role_name) = &role.name;
        let res = self
            .query_to_obj::<StandardGrant>(&format!("SHOW GRANTS TO ROLE {}", &role_name))
            .await
            .context("failed to get grants to role")?;

        let mut target = target.lock().unwrap();
        target.extend(res);
        Ok(())
    }

    /// Get all grants of a role
    pub(crate) async fn get_grants_of_role_future(
        &self,
        role: &Role,
        target: Arc<Mutex<&mut Vec<GrantOf>>>,
    ) -> Result<()> {
        let RoleName(role_name) = &role.name;
        let res = self
            .query_to_obj::<GrantOf>(&format!("SHOW GRANTS OF ROLE {}", &role_name))
            .await
            .context("failed to get grants of role")?;

        let mut target = target.lock().unwrap();
        target.extend(res);
        Ok(())
    }

    /// Get all future grants for a schema
    pub async fn get_future_grants_of_schema_future(
        &self,
        schema: &Schema,
        target: Arc<Mutex<&mut Vec<FutureGrant>>>,
    ) -> Result<()> {
        let res = self
            .query_to_obj::<FutureGrant>(&format!(
                "SHOW FUTURE GRANTS IN SCHEMA {}.{}",
                &schema.database_name, &schema.name
            ))
            .await
            .context(format!(
                "failed to get future grants on schema {}",
                &schema.name
            ))?;

        let mut target = target.lock().unwrap();
        target.extend(res);
        Ok(())
    }

    /// Get all future grants for a database
    pub async fn get_future_grants_of_database_future(
        &self,
        database: &Database,
        target: Arc<Mutex<&mut Vec<FutureGrant>>>,
    ) -> Result<()> {
        let res = self
            .query_to_obj::<FutureGrant>(&format!(
                "SHOW FUTURE GRANTS IN DATABASE {}",
                &database.name
            ))
            .await
            .context(format!(
                "failed to get future grants on database {}",
                &database.name
            ))?;

        let mut target = target.lock().unwrap();
        target.extend(res);
        Ok(())
    }

    /// Get all users.
    pub async fn get_users_future(&self, target: &mut Vec<User>) -> Result<()> {
        *target = self
            .query_to_obj::<User>("SHOW USERS")
            .await
            .context("failed to get users")?;
        Ok(())
    }

    /// Get all roles.
    pub(crate) async fn get_roles_future(&self, target: &mut Vec<Role>) -> Result<()> {
        *target = self
            .query_to_obj::<Role>("SHOW ROLES")
            .await
            .context("failed to get roles")?;
        Ok(())
    }

    /// Get all databases.
    pub async fn get_databases_future(&self, target: &mut Vec<Database>) -> Result<()> {
        *target = self
            .query_to_obj::<Database>("SHOW DATABASES")
            .await
            .context("failed to get databases")?;
        Ok(())
    }

    /// Get all warehouses.
    pub async fn get_warehouses(&self) -> Result<Vec<Warehouse>> {
        self.query_to_obj::<Warehouse>("SHOW WAREHOUSES")
            .await
            .context("failed to get warehouses")
    }

    /// Get all schemas.
    pub async fn get_schemas_future(&self, target: &mut Vec<Schema>) -> Result<()> {
        *target = self
            .query_to_obj::<Schema>("SHOW SCHEMAS IN ACCOUNT")
            .await
            .context("failed to get schemas")?;
        Ok(())
    }

    /// Get all tables.
    pub async fn get_objects_futures(
        &self,
        schema: &Schema,
        target: Arc<Mutex<&mut Vec<Object>>>,
    ) -> Result<()> {
        let query = format!(
            "SHOW OBJECTS IN SCHEMA {}.{}",
            &schema.database_name, &schema.name
        );
        let res = self
            .query_to_obj::<Object>(&query)
            .await
            .context("failed to get tables")?;
        let mut target = target.lock().unwrap();
        target.extend(res);
        Ok(())
    }

    /// Execute the given query and deserialize the result into the given type.
    pub async fn query_to_obj<T>(&self, query: &str) -> Result<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let result = self
            .rest_client
            .query(&SnowflakeRequestConfig {
                sql: query.to_string(),
                use_jwt: self.client != connectors::ConnectorClient::Test,
            })
            .await
            .context("query failed")?;
        if result.is_empty() {
            // TODO: Determine whether this is actually okay behavior.
            return Ok(vec![]);
        }
        let rows_value: JsonValue =
            serde_json::from_str(&result).context("failed to deserialize")?;
        if let Some(info) = rows_value.get("partitionInfo") {
            panic!("Unexpected partitioned return value: {info}");
        }
        let rows_data = rows_value["data"].clone();
        let rows = serde_json::from_value::<Vec<Vec<Option<String>>>>(rows_data)
            .context("failed to deserialize rows")?
            .into_iter()
            .map(|v| v.iter().map(|f| f.clone().unwrap_or_default()).collect());
        let fields_intermediate: Vec<SnowflakeField> =
            serde_json::from_value(rows_value["resultSetMetaData"]["rowType"].clone())
                .context("failed to deserialize fields")?;
        let fields: Vec<String> = fields_intermediate.iter().map(|i| i.name.clone()).collect();
        Ok(rows
            .map(|i: Vec<_>| {
                // Zip field - i
                let vals: HashMap<String, String> = zip(fields.clone(), i).collect();
                T::deserialize(MapDeserializer::<
                    std::collections::hash_map::IntoIter<std::string::String, std::string::String>,
                    serde::de::value::Error,
                >::new(vals.into_iter()))
                .context("couldn't deserialize")
                .unwrap()
            })
            .collect())
    }

    fn grants_to_policies(&self, grants: &[GrantType]) -> Vec<nodes::RawPolicy> {
        grants
            .iter()
            .filter(|g| consts::ASSET_TYPES.contains(&g.granted_on()))
            // Collect roles by asset name so the role:asset ratio is 1:1.
            .fold(
                HashMap::new(),
                |mut asset_map: HashMap<String, HashSet<GrantType>>, g| {
                    if let Some(asset_privileges) = asset_map.get_mut(g.granted_on_name()) {
                        asset_privileges.insert(g.clone());
                    } else {
                        asset_map
                            .insert(g.granted_on_name().to_owned(), HashSet::from([g.clone()]));
                    }
                    asset_map
                },
            )
            .iter()
            .filter_map(|(_asset_name, grants)| {
                // When we read, a policy will get created for each unique
                // role/user, asset combination. All privileges will be bunched together
                // for that combination.
                if grants.is_empty() {
                    // No privileges.
                    return None;
                }
                // Each set of grants should be exactly the same except for privileges.
                // We will take the first one...
                let final_grant = grants.iter().next().cloned().unwrap();
                // ...and now we'll combine all of the privileges from the
                // grants into one policy.
                let privileges: HashSet<String> =
                    grants.iter().map(|g| g.privilege().to_owned()).collect();
                Some(final_grant.into_policy(privileges))
            })
            .collect::<Vec<_>>()
    }
}
