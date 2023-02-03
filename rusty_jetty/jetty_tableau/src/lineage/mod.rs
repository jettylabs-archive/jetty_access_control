//! Collect lineage for tableau assets using the metadata API.
use anyhow::{anyhow, bail, Context, Result};
use jetty_core::logging::warn;
use serde::{de::DeserializeOwned, Deserialize};

use std::collections::HashMap;
use std::sync::Arc;

use jetty_core::cual::Cual;

use crate::coordinator::{Coordinator, Environment};
use crate::rest::{self, FetchJson};
use crate::TableauRestClient;

// Get the table ids upstream and downstream for all of the assets.

/// Map of database table IDs to database names
type Databases = HashMap<String, String>;

/// Map of database IDs to DatabaseServer structs
type DatabaseServers = HashMap<String, DatabaseServer>;

/// Database server information returned from Tableau
struct DatabaseServer {
    /// Server hostname (e.g., "abc12345.snowflakecomputing.com/")
    hostname: String,
    /// Server connection type (e.g., "snowflake")
    connection_type: String,
}

impl DatabaseServer {
    fn cual_prefix(&self) -> Result<String> {
        Ok(match self.connection_type.as_str() {
            "snowflake" => format!("snowflake://{}", self.hostname),
            _ => bail!("unsupported connection type: {}", self.connection_type),
        })
    }
}

/// Resolve a table id to a database cual
struct TableResolver {
    databases: Databases,
    database_servers: DatabaseServers,
}

impl TableResolver {
    /// Get the database cuals for the given database table.
    /// Returns `None` if the database is not found. This can happen if the
    /// metadata API isn't working properly or if the db has been filtered out
    /// (e.g., snowflake only)
    fn get_cual(&self, table: &DatabaseTable) -> Option<Vec<Cual>> {
        let mut res = Vec::new();

        for db_id in &table.database_ids {
            let db_name = match self.databases.get(db_id) {
                Some(s) => s,
                None => continue,
            };
            let prefix = match self.database_servers.get(db_id) {
                Some(s) => match s.cual_prefix() {
                    Ok(s) => s,
                    Err(e) => {
                        warn!("tableau lineage: failed to get cual prefix: {e}");
                        continue;
                    }
                },
                None => continue,
            };
            match clean_table_name(&table.full_name) {
                Ok((schema, table)) => res.push(Cual::new(
                    format!("{}{}/{}/{}", prefix, db_name, schema, table,).as_str(),
                )),
                Err(e) => {
                    warn!("tableau lineage: unable to clean table name: {e}");
                    continue;
                }
            }
        }

        Some(res)
    }
}
/// All the database tables in the Tableau environment
struct DatabaseTable {
    /// The table ID (metadata API-only)
    id: String,
    /// The full name of the database table ([SCHEMA].[TABLE])
    full_name: String,
    /// The id of the database the table belongs to (metadata API-only)
    database_ids: Vec<String>,
}

/// Given a tableau-formatted table full name (e.g., [SCHEMA].[table]), return two cual-ready
/// name segments (e.g., [schema].[table])
fn clean_table_name(name: &str) -> Result<(String, String)> {
    let parts = name
        .split("].[")
        .map(|identifier| {
            let identifier = identifier.strip_prefix('[').unwrap_or(identifier);
            let identifier = identifier.strip_suffix(']').unwrap_or(identifier);
            identifier.replace(r#"\""#, r#""""#)
        })
        .collect::<Vec<_>>();
    let schema = parts.get(0).ok_or(anyhow!("Invalid schema name"))?;
    let table = parts.get(1).ok_or(anyhow!("Invalid table name"))?;

    Ok((schema.to_owned(), table.to_owned()))
}

impl Coordinator {
    /// Fetch the database servers from the tableau metadata API and
    async fn get_database_servers(&self) -> Result<HashMap<String, DatabaseServer>> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct DatabaseServersResponse {
            connection_type: String,
            hostname: String,
            id: String,
        }

        let query = r#"
        query servers {
            databaseServers(filter: {connectionType: "snowflake"}) {
                connectionType
                hostName
                id
            }
        }
        "#;

        let response: Vec<DatabaseServersResponse> = self
            .graphql_query_to_object_vec(query, vec!["data", "databaseServers"])
            .await?;

        Ok(response
            .into_iter()
            .map(|r| {
                (
                    r.id,
                    DatabaseServer {
                        hostname: r.hostname,
                        connection_type: r.connection_type,
                    },
                )
            })
            .collect())
    }

    /// Fetch the database servers from the tableau metadata API and
    async fn get_databases(&self) -> Result<HashMap<String, String>> {
        #[derive(Deserialize)]
        struct DatabaseResponse {
            id: String,
            name: String,
        }

        let query = r#"
            query servers {
                databases(filter: {connectionType: "snowflake"}) {
                  id
                  name
                }
            }
            "#;

        let response: Vec<DatabaseResponse> = self
            .graphql_query_to_object_vec(query, vec!["data", "databases"])
            .await?;

        Ok(response.into_iter().map(|r| (r.id, r.name)).collect())
    }

    /// Return a vector of objects from a graphql query
    async fn graphql_query_to_object_vec<T>(&self, query: &str, path: Vec<&str>) -> Result<Vec<T>>
    where
        T: DeserializeOwned,
    {
        let node = self
            .rest_client
            .build_graphql_request(query.to_owned())?
            .fetch_json_response(None)
            .await?;
        let node = rest::get_json_from_path(
            &node,
            &path.into_iter().map(|s| s.to_owned()).collect::<Vec<_>>(),
        )?;

        serde_json::from_value(node).map_err(anyhow::Error::from)
    }
}

// graphql to vec of objects

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_table_name() -> Result<()> {
        let name_pairs = vec![
            (
                "[GOLD].[IRIS_JOINED_TABLE]",
                ("GOLD".to_owned(), "IRIS_JOINED_TABLE".to_owned()),
            ),
            (
                r#"[RAW].[\"Special Name\"]"#,
                ("RAW".to_owned(), r#"""Special Name"""#.to_owned()),
            ),
        ];
        for (in_name, out_name) in name_pairs {
            assert_eq!(clean_table_name(in_name)?, out_name);
        }
        Ok(())
    }
}
