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
mod write;

use cual::set_cual_account_name;
pub use entry_types::{
    Asset, Database, Entry, FutureGrant, Grant, GrantOf, GrantType, Object, Role, RoleName, Schema,
    StandardGrant, User, Warehouse,
};
use futures::StreamExt;
use jetty_core::access_graph::translate::diffs::LocalConnectorDiffs;
use jetty_core::connectors::{
    AssetType, ConnectorCapabilities, NewConnector, ReadCapabilities, WriteCapabilities,
};
use jetty_core::jetty::ConnectorManifest;
use jetty_core::logging::error;

use rest::{SnowflakeRequestConfig, SnowflakeRestClient, SnowflakeRestConfig};
use serde::de::value::MapDeserializer;

use std::collections::{HashMap, HashSet};
use std::iter::zip;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use jetty_core::{
    connectors,
    connectors::{nodes, Connector},
    jetty::{ConnectorConfig, CredentialsMap},
};

use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Deserializer};
use serde_json::Value as JsonValue;

const CONCURRENT_WAREHOUSE_QUERIES: usize = 5;

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

    fn get_manifest(&self) -> ConnectorManifest {
        ConnectorManifest {
            capabilities: ConnectorCapabilities {
                read: HashSet::from([
                    ReadCapabilities::Assets,
                    ReadCapabilities::Groups,
                    ReadCapabilities::Policies {
                        default_policies: true,
                    },
                    ReadCapabilities::Users,
                ]),
                write: HashSet::from([
                    WriteCapabilities::Groups { nested: true },
                    WriteCapabilities::Policies {
                        default_policies: true,
                    },
                ]),
            },
            asset_privileges: [
                (
                    AssetType(consts::DATABASE.to_owned()),
                    [
                        "MODIFY",
                        "MONITOR",
                        "USAGE",
                        "CREATE SCHEMA",
                        "OWNERSHIP",
                        "IMPORTED PRIVILEGES",
                        "REFERENCE_USAGE",
                    ]
                    .into_iter()
                    .map(|p| p.to_owned())
                    .collect(),
                ),
                (
                    AssetType(consts::SCHEMA.to_owned()),
                    [
                        "OWNERSHIP",
                        "MODIFY",
                        "MONITOR",
                        "USAGE",
                        "CREATE EXTERNAL TABLE",
                        "CREATE FILE FORMAT",
                        "CREATE FUNCTION",
                        "CREATE MASKING POLICY",
                        "CREATE MATERIALIZED VIEW",
                        "CREATE PASSWORD POLICY",
                        "CREATE PIPE",
                        "CREATE PROCEDURE",
                        "CREATE ROW ACCESS POLICY",
                        "CREATE SESSION POLICY",
                        "CREATE SEQUENCE",
                        "CREATE STAGE",
                        "CREATE STREAM",
                        "CREATE TAG",
                        "CREATE TABLE",
                        "CREATE TASK",
                        "CREATE VIEW",
                        "ADD SEARCH OPTIMIZATION",
                        "CREATE TEMPORARY TABLE",
                    ]
                    .into_iter()
                    .map(|p| p.to_owned())
                    .collect(),
                ),
                (
                    AssetType(consts::TABLE.to_owned()),
                    [
                        "OWNERSHIP",
                        "SELECT",
                        "INSERT",
                        "UPDATE",
                        "DELETE",
                        "TRUNCATE",
                        "REFERENCES",
                        "REBUILD",
                    ]
                    .into_iter()
                    .map(|p| p.to_owned())
                    .collect(),
                ),
                (
                    AssetType(consts::VIEW.to_owned()),
                    [
                        "OWNERSHIP",
                        "SELECT",
                        "REFERENCES",
                        "DELETE",
                        "INSERT",
                        "REBUILD",
                        "TRUNCATE",
                        "UPDATE",
                    ]
                    .into_iter()
                    .map(|p| p.to_owned())
                    .collect(),
                ),
            ]
            .into(),
        }
    }
    fn plan_changes(&self, diffs: &LocalConnectorDiffs) -> Vec<std::string::String> {
        self.generate_diff_queries(diffs).flatten()
    }

    async fn apply_changes(&self, diffs: &LocalConnectorDiffs) -> Result<String> {
        let mut success_counter = 0;
        let mut failure_counter = 0;
        // This is designed in such a way that each query_set may be run concurrently.
        let prepared_queries = self.generate_diff_queries(diffs);
        for query_set in [prepared_queries.0, prepared_queries.1, prepared_queries.2] {
            let query_set_configs = query_set
                .iter()
                .map(|q| SnowflakeRequestConfig {
                    sql: q.to_owned(),
                    use_jwt: true,
                })
                .collect::<Vec<_>>();

            let query_futures = query_set_configs
                .iter()
                .map(|q| self.rest_client.execute(q))
                .collect::<Vec<_>>();

            let results = futures::stream::iter(query_futures)
                .buffered(CONCURRENT_WAREHOUSE_QUERIES)
                .collect::<Vec<_>>()
                .await;

            for result in results {
                match result {
                    Err(e) => {
                        error!("{:?}", e);
                        failure_counter += 1;
                    }
                    Ok(_) => {
                        success_counter += 1;
                    }
                }
            }
        }
        Ok(format!(
            "{success_counter} successful queries\n{failure_counter} failed queries"
        ))
    }
}

impl SnowflakeConnector {
    /// Get all grants to a role – the privileges and "children" roles.
    pub(crate) async fn get_privilege_grants_future(
        &self,
        target: Arc<Mutex<&mut Vec<StandardGrant>>>,
    ) -> Result<()> {
        let res = self
            .query_to_obj::<StandardGrant>("select * from snowflake.account_usage.grants_to_roles where deleted_on is null and granted_on in ('TABLE', 'DATABASE', 'SCHEMA', 'VIEW');")
            .await
            .context("failed to get privilege grants")?;

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
            .query_to_obj::<GrantOf>(&format!("SHOW GRANTS OF ROLE \"{}\"", &role_name))
            .await
            .context(format!("failed to get grants of role {role_name}"))?;

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
                r#"SHOW FUTURE GRANTS IN SCHEMA "{}"."{}""#,
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
                "SHOW FUTURE GRANTS IN DATABASE \"{}\"",
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
            "SHOW OBJECTS IN SCHEMA \"{}\".\"{}\"",
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
        T: for<'de> Deserialize<'de> + std::fmt::Debug,
    {
        let result = self
            .rest_client
            .query(&SnowflakeRequestConfig {
                sql: query.to_string(),
                use_jwt: self.client != connectors::ConnectorClient::Test,
            })
            .await;

        let result = match result {
            Ok(s) => s,
            Err(e) => {
                error!("error running `{query}`: {e}");
                bail!("error running `{query}`: {e}");
            }
        };

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
                    if let Some(asset_privileges) = asset_map.get_mut(&g.granted_on_name()) {
                        asset_privileges.insert(g.clone());
                    } else {
                        asset_map.insert(g.granted_on_name(), HashSet::from([g.clone()]));
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

    /// Convert future grants into default policies. This is called for each role, so grants contains only policies
    /// for a single role.
    fn future_grants_to_default_policies(
        &self,
        grants: &[FutureGrant],
    ) -> Vec<nodes::RawDefaultPolicy> {
        grants
            .iter()
            // filter down to the asset types we support
            .filter(|g| consts::ASSET_TYPES.contains(&g.grant_on()))
            // Collect policies by asset name and grant_on (asset type). Asset type and role combined give a path, so this will give us a single policy
            // for each combo of (Asset, Path, Asset Type, and Agent)
            .fold(
                HashMap::new(),
                |mut asset_map: HashMap<(String, String), HashSet<FutureGrant>>, g| {
                    if let Some(asset_privileges) = asset_map
                        .get_mut(&(g.granted_on_name().to_owned(), g.grant_on().to_owned()))
                    {
                        asset_privileges.insert(g.clone());
                    } else {
                        asset_map.insert(
                            (g.granted_on_name().to_owned(), g.grant_on().to_owned()),
                            HashSet::from([g.clone()]),
                        );
                    }
                    asset_map
                },
            )
            .iter()
            .filter_map(|(_asset_name, grants)| {
                // When we read, a policy will get created for each unique
                // role/user, root asset, type combination. All privileges will be bunched together
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
                Some(final_grant.into_default_policy(privileges))
            })
            .collect::<Vec<_>>()
    }
}

pub(crate) fn strip_snowflake_quotes(object: String, capitalize: bool) -> String {
    if object.starts_with("\"\"\"") {
        object.replace("\"\"\"", "\"\"")
    } else if object.starts_with('"') {
        // Remove the quotes and return the contained part as-is.
        object.trim_matches('"').to_owned()
    } else {
        // Not quoted – we can just capitalize it (only for
        // Snowflake).
        if capitalize {
            object.to_uppercase()
        } else {
            // In some cases, like when it is a value from Snowflake, we don't need to capitalize it. We just leave it as is.
            object
        }
    }
}

/// Given a snowflake identifier (e.g. a table name, but not a fqn), escape any quotes in it by converting to double quotes.
pub(crate) fn escape_snowflake_quotes(identifier: &str) -> String {
    if identifier.contains('"') {
        identifier.replace('"', "\"\"")
    } else {
        identifier.to_owned()
    }
}

/// A Snowflake Asset. Inner value is the fully-qualified snowflake name.
#[derive(PartialEq, Debug)]
enum SnowflakeAsset {
    Table(String),
    View(String),
    Schema(String),
    Database(String),
}

impl SnowflakeAsset {
    /// Get the snowflake fully-qualified name for the asset
    fn fqn(&self) -> &String {
        match self {
            SnowflakeAsset::Table(fqn) => fqn,
            SnowflakeAsset::View(fqn) => fqn,
            SnowflakeAsset::Schema(fqn) => fqn,
            SnowflakeAsset::Database(fqn) => fqn,
        }
    }

    /// Get the asset type as a &str
    fn asset_type(&self) -> &str {
        match self {
            SnowflakeAsset::Table(_) => "TABLE",
            SnowflakeAsset::View(_) => "VIEW",
            SnowflakeAsset::Schema(_) => "SCHEMA",
            SnowflakeAsset::Database(_) => "DATABASE",
        }
    }
}

pub(crate) fn strip_quotes_and_deserialize<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;
    Ok(strip_snowflake_quotes(buf, false))
}
