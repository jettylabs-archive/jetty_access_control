//! Snowflake Connector
//!
//! Everything needed for connection and interaction with Snowflake.&
//!
//! ```
//! use jetty_core::connectors::{Connector, ConnectorClient};
//! use jetty_core::jetty::{ConnectorConfig, CredentialsBlob};
//! use jetty_snowflake::SnowflakeConnector;
//!
//! let config = ConnectorConfig::default();
//! let credentials = CredentialsBlob::default();
//! let connector_client = ConnectorClient::Core;
//! let snow = SnowflakeConnector::new(&config, &credentials, Some(connector_client));
//! ```

mod consts;
mod creds;
mod cual;
mod entry_types;
mod rest;

pub use entry_types::*;
use jetty_core::cual::Cualable;
use rest::{SnowflakeRequestConfig, SnowflakeRestClient, SnowflakeRestConfig};

use futures::stream::{FuturesUnordered, StreamExt};
use std::collections::{HashMap, HashSet};
use std::iter::zip;

use jetty_core::{
    connectors,
    connectors::{nodes, Connector, UserIdentifier},
    jetty::{ConnectorConfig, CredentialsBlob},
};

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value as JsonValue;
use structmap::{value::Value, FromMap, GenericMap};

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
                println!("{:?}", e);
                false
            }
            Ok(_) => true,
        };
    }

    async fn get_data(&mut self) -> nodes::ConnectorData {
        nodes::ConnectorData {
            groups: self
                .get_jetty_groups()
                .await
                .context("failed to get groups")
                .unwrap(),
            users: self
                .get_jetty_users()
                .await
                .context("failed to get users")
                .unwrap(),
            assets: self
                .get_jetty_assets()
                .await
                .context("failed to get assets")
                .unwrap(),
            tags: self
                .get_jetty_tags()
                .await
                .context("failed to get tags")
                .unwrap(),
            policies: self
                .get_jetty_policies()
                .await
                .context("failed to get policies")
                .unwrap(),
        }
    }

    /// Validates the configs and bootstraps a Snowflake connection.
    ///
    /// Validates that the required fields are present to authenticate to
    /// Snowflake. Stashes the credentials in the struct for use when
    /// connecting.
    fn new(
        _config: &ConnectorConfig,
        credentials: &CredentialsBlob,
        connector_client: Option<connectors::ConnectorClient>,
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

impl SnowflakeConnector {
    /// Get all grants to a user
    pub async fn get_grants_to_user(&self, user_name: &str) -> Result<Vec<RoleGrant>> {
        self.query_to_obj::<RoleGrant>(&format!("SHOW GRANTS TO USER {}", user_name))
            .await
            .context("failed to get grants to user")
    }

    /// Get all grants to a role – the privileges and "children" roles.
    pub async fn get_grants_to_role(&self, role_name: &str) -> Result<Vec<Grant>> {
        self.query_to_obj::<Grant>(&format!("SHOW GRANTS TO ROLE {}", role_name))
            .await
            .context("failed to get grants to role")
    }

    /// Get all grants on a role – the "parent" roles.
    pub async fn get_grants_on_role(&self, role_name: &str) -> Result<Vec<Grant>> {
        self.query_to_obj::<Grant>(&format!("SHOW GRANTS ON ROLE {}", role_name))
            .await
            .context("failed to get grants on role")
    }
    /// Get all users.
    pub async fn get_users(&self) -> Result<Vec<User>> {
        self.query_to_obj::<User>("SHOW USERS")
            .await
            .context("failed to get users")
    }

    /// Get all roles.
    pub async fn get_roles(&self) -> Result<Vec<Role>> {
        self.query_to_obj::<Role>("SHOW ROLES")
            .await
            .context("failed to get roles")
    }

    /// Get all databases.
    pub async fn get_databases(&self) -> Result<Vec<Database>> {
        self.query_to_obj::<Database>("SHOW DATABASES")
            .await
            .context("failed to get databases")
    }

    /// Get all warehouses.
    pub async fn get_warehouses(&self) -> Result<Vec<Warehouse>> {
        self.query_to_obj::<Warehouse>("SHOW WAREHOUSES")
            .await
            .context("failed to get warehouses")
    }

    /// Get all schemas.
    pub async fn get_schemas(&self) -> Result<Vec<Schema>> {
        self.query_to_obj::<Schema>("SHOW SCHEMAS")
            .await
            .context("failed to get schemas")
    }

    /// Get all views.
    pub async fn get_views(&self) -> Result<Vec<View>> {
        self.query_to_obj::<View>("SHOW VIEWS")
            .await
            .context("failed to get views")
    }

    /// Get all tables.
    pub async fn get_tables(&self) -> Result<Vec<Table>> {
        self.query_to_obj::<Table>("SHOW TABLES")
            .await
            .context("failed to get tables")
    }

    /// Execute the given query and deserialize the result into the given type.
    pub async fn query_to_obj<T>(&self, query: &str) -> Result<Vec<T>>
    where
        T: FromMap,
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
            panic!("Unexpected partitioned return value: {}", info);
        }
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

    /// When we read, a policy will get created for each unique
    /// role/user, asset combination. All privileges will be bunched together
    /// for that combination.
    async fn grant_to_policy(
        &self,
        role_name: &str,
        grants: &HashSet<Grant>,
    ) -> Option<nodes::Policy> {
        if grants.is_empty() {
            // No privileges.
            return None;
        }
        let privileges: Vec<String> = grants.iter().map(|g| g.privilege.to_owned()).collect();
        Some(nodes::Policy::new(
            format!(
                "{}.{}.{}",
                role_name.to_owned(),
                privileges.join("."),
                role_name,
            ),
            privileges.iter().cloned().collect(),
            // Unwrap here is fine since we asserted that the set was not empty above.
            HashSet::from([grants.iter().next().unwrap().name.to_owned()]),
            HashSet::new(),
            HashSet::from([role_name.to_owned()]),
            // No direct user grants in Snowflake. Grants must pass through roles.
            HashSet::new(),
            // Defaults here for data read from Snowflake should be false.
            false,
            false,
        ))
    }

    async fn get_jetty_policies(&self) -> Result<Vec<nodes::Policy>> {
        let mut res = vec![];
        // Role grants
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
                // Collect roles by asset name so the policy:asset ratio is 1:1.
                .fold(
                    HashMap::new(),
                    |mut asset_map: HashMap<String, HashSet<Grant>>, g| {
                        if let Some(asset_privileges) = asset_map.get_mut(&g.name) {
                            asset_privileges.insert(g.clone());
                        } else {
                            asset_map.insert(g.name.to_owned(), HashSet::from([g.clone()]));
                        }
                        asset_map
                    },
                )
                .iter()
                .map(|(_asset_name, grants)| self.grant_to_policy(&role.name, grants))
                .collect::<FuturesUnordered<_>>()
                .collect::<Vec<_>>()
                .await;
            res.append(&mut grants_to_role)
        }
        let res = res
            .iter()
            // TODO: This is disgusting, we should fix it.
            .filter(|x| x.is_some())
            .map(|x| x.as_ref().unwrap())
            .cloned()
            .collect();
        Ok(res)
    }

    /// Enterprise-only feature. We'll have to do something about that at some point.
    async fn get_jetty_tags(&self) -> Result<Vec<nodes::Tag>> {
        Ok(vec![])
    }

    async fn get_jetty_groups(&self) -> Result<Vec<nodes::Group>> {
        let mut res = vec![];
        for role in self.get_roles().await.context("failed to get roles")? {
            let sub_roles = self
                .get_grants_to_role(&role.name)
                .await?
                .iter()
                // Only get subgroups
                .filter(|g| g.granted_on == "ROLE")
                .map(|g| g.name.to_owned())
                .collect();
            res.push(nodes::Group::new(
                role.name.to_owned(),
                HashMap::new(),
                // We only handle parent relationships. The resulting
                // child relationships are handled by Jetty.
                HashSet::new(),
                // Included users are handled in get_jetty_users
                HashSet::new(),
                sub_roles,
                // Policies applied are handled in get_jetty_policies
                HashSet::new(),
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
                HashMap::from([
                    (UserIdentifier::Email, user.email),
                    (UserIdentifier::FirstName, user.first_name),
                    (UserIdentifier::LastName, user.last_name),
                ]),
                HashSet::from([user.display_name, user.login_name]),
                HashMap::new(),
                user_roles.iter().map(|role| role.role.clone()).collect(),
                // Policies applied are handled in get_jetty_policies
                HashSet::new(),
            ));
        }
        Ok(res)
    }

    async fn get_jetty_assets(&self) -> Result<Vec<nodes::Asset>> {
        let mut res = vec![];
        for table in self.get_tables().await? {
            res.push(nodes::Asset::new(
                table.cual(),
                format!(
                    "{}.{}.{}",
                    table.database_name, table.schema_name, table.name
                ),
                connectors::AssetType::DBTable,
                HashMap::new(),
                // Policies applied are handled in get_jetty_policies
                HashSet::new(),
                HashSet::from([format!("{}.{}", table.database_name, table.schema_name)]),
                // Handled in child_of for parents.
                HashSet::new(),
                // We aren't extracting lineage from Snowflake right now.
                HashSet::new(),
                HashSet::new(),
                HashSet::new(),
            ));
        }

        for view in self.get_views().await? {
            res.push(nodes::Asset::new(
                view.cual(),
                format!("{}.{}.{}", view.database_name, view.schema_name, view.name),
                connectors::AssetType::DBView,
                HashMap::new(),
                // Policies applied are handled in get_jetty_policies
                HashSet::new(),
                HashSet::from([format!("{}.{}", view.database_name, view.schema_name)]),
                // Handled in child_of for parents.
                HashSet::new(),
                // We aren't extracting lineage from Snowflake right now.
                HashSet::new(),
                HashSet::new(),
                HashSet::new(),
            ));
        }

        let schemas = self.get_schemas().await?;

        for schema in schemas {
            // TODO: Get subassets
            res.push(nodes::Asset::new(
                schema.cual(),
                format!("{}.{}", schema.database_name, schema.name),
                connectors::AssetType::DBSchema,
                HashMap::new(),
                // Policies applied are handled in get_jetty_policies
                HashSet::new(),
                HashSet::from([schema.database_name]),
                // Handled in child_of for parents.
                HashSet::new(),
                // We aren't extracting lineage from Snowflake right now.
                HashSet::new(),
                HashSet::new(),
                HashSet::new(),
            ));
        }

        for db in self.get_databases().await? {
            // TODO: Get subassets
            res.push(nodes::Asset::new(
                db.cual(),
                db.name,
                connectors::AssetType::DBDB,
                HashMap::new(),
                // Policies applied are handled in get_jetty_policies
                HashSet::new(),
                HashSet::new(),
                // Handled in child_of for parents.
                HashSet::new(),
                // We aren't extracting lineage from Snowflake right now.
                HashSet::new(),
                HashSet::new(),
                HashSet::new(),
            ));
        }

        Ok(res)
    }
}
