//! Collect lineage for tableau assets using the metadata API.
//!

mod assets;
mod sql;

use anyhow::{bail, Context, Result};
use jetty_core::logging::{error, warn};
use serde::{de::DeserializeOwned, Deserialize};

use std::collections::{HashMap, HashSet};

use jetty_core::cual::Cual;

use crate::coordinator::Coordinator;
use crate::rest::{self, FetchJson};

// Get the table ids upstream and downstream for all of the assets.

/// Map of database table IDs to database names
type Databases = HashMap<String, String>;

/// Map of database IDs to DatabaseServer structs
type DatabaseServers = HashMap<String, DatabaseServer>;

/// Database server information returned from Tableau
#[derive(Debug)]
struct DatabaseServer {
    /// Server hostname (e.g., "abc12345.snowflakecomputing.com/")
    host_name: String,
    /// Server connection type (e.g., "snowflake")
    connection_type: String,
}

/// Just a string id (for deserialization purposes)
#[derive(Deserialize, Hash, Eq, PartialEq, Debug)]
struct IdField {
    id: String,
}

impl DatabaseServer {
    fn cual_prefix(&self) -> Result<String> {
        Ok(match self.connection_type.as_str() {
            "snowflake" => format!(
                "snowflake://{}",
                if self.host_name.is_empty() {
                    "NO_HOSTNAME"
                } else {
                    &self.host_name
                }
            ),
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
    fn get_cual(&self, table: &DatabaseTable) -> Option<Cual> {
        let db_name = self.databases.get(&table.database_id)?;
        let prefix = self
            .database_servers
            .get(&table.database_id)?
            .cual_prefix()
            .ok()?;
        match sql::parse_identifier(&table.full_name) {
            Ok(t) => Some(Cual::new({
                let (db, schema, table) = if t.len() == 3 {
                    (t[0].to_owned(), t[1].to_owned(), t[2].to_owned())
                } else if t.len() == 2 {
                    (db_name.to_owned(), t[0].to_owned(), t[1].to_owned())
                } else {
                    warn!(
                        "unable to clean table name ({}); skipping table",
                        table.full_name
                    );
                    return None;
                };

                format!(
                    "{}{}{}/{}/{}",
                    prefix,
                    if prefix.ends_with('/') { "" } else { "/" },
                    db,
                    schema,
                    table,
                )
                .as_str()
            })),
            Err(e) => {
                warn!(
                    "unable to clean table name ({}); skipping table: {e}",
                    table.full_name
                );
                None
            }
        }
    }
}

/// Map of database IDs to DatabaseServer structs
type DatabaseTables = HashMap<String, DatabaseTable>;

#[derive(Debug)]
/// All the database tables in the Tableau environment
struct DatabaseTable {
    /// The full name of the database table ([SCHEMA].[TABLE], or schema.table, or even db.schema.table)
    full_name: String,
    /// The id of the database the table belongs to (metadata API-only)
    database_id: String,
}

fn get_table_cuals(tables: DatabaseTables, resolver: TableResolver) -> HashMap<String, Cual> {
    tables
        .into_iter()
        .filter_map(|(id, table)| resolver.get_cual(&table).map(|c| (id, c)))
        .collect()
}

impl Coordinator {
    /// Fetch the database tables from the tableau metadata API
    async fn get_database_tables(&self) -> Result<DatabaseTables> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct DatabaseTablesResponse {
            full_name: String,
            id: String,
            database: IdField,
        }

        let query = r#"
    query datbaseTables {
        databaseTables {
          fullName
          id
          database {
            id
          }
        }
      }
    "#;

        let response: Vec<DatabaseTablesResponse> = self
            .graphql_query_to_object_vec(query, vec!["data", "databaseTables"])
            .await?;

        Ok(response
            .into_iter()
            .map(|r| {
                (
                    r.id,
                    DatabaseTable {
                        full_name: r.full_name,
                        database_id: r.database.id,
                    },
                )
            })
            .collect())
    }

    /// Fetch the tables that use custom unsupported SQL
    async fn get_tables_with_unsupported_sql(&self) -> Result<HashSet<String>> {
        let query = r#"    
        query Assets {
            customSQLTables(filter:{isUnsupportedCustomSql: true}){
              id
            }
          }
    "#;

        let response: Vec<IdField> = self
            .graphql_query_to_object_vec(query, vec!["data", "customSQLTables"])
            .await?;

        Ok(response.into_iter().map(|r| r.id).collect())
    }

    /// Fetch the database servers from the tableau metadata API
    async fn get_database_servers(&self) -> Result<HashMap<String, DatabaseServer>> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct DatabaseServersResponse {
            connection_type: String,
            host_name: String,
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
                        host_name: r.host_name,
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
            .await
            .context(format!("query: {query}"))?;
        let node = rest::get_json_from_path(
            &node,
            &path.into_iter().map(|s| s.to_owned()).collect::<Vec<_>>(),
        )?;

        serde_json::from_value(node).map_err(anyhow::Error::from)
    }

    /// Update the lineage of data assets
    // FUTURE: If needed, parts of this can be run concurrently.
    pub(super) async fn update_lineage(&mut self) -> Result<()> {
        let unsupported_sql = &self.get_tables_with_unsupported_sql().await.map_err(|e| {
            error!("failed to get tables with unsupported sql -- error: {}", &e);
            e
        })?;
        let resolver = TableResolver {
            databases: self.get_databases().await.map_err(|e| {
                error!(
                    "failed to get databases from the metadata api -- error: {}",
                    &e
                );
                e
            })?,
            database_servers: self.get_database_servers().await.map_err(|e| {
                error!(
                    "failed to get database servers from the metadata api -- error: {}",
                    &e
                );
                e
            })?,
        };
        let tables = self.get_database_tables().await.map_err(|e| {
            error!(
                "failed to get database tables from the metadata api -- error: {}",
                &e
            );
            e
        })?;
        let cual_map = get_table_cuals(tables, resolver);

        // Update workbooks
        assets::update_sources(
            self.fetch_workbooks_references(&cual_map, unsupported_sql)
                .await
                .map_err(|e| {
                    error!("failed to fetch references for workbooks -- error: {}", &e);
                    e
                })?,
            &mut self.env.workbooks,
        );

        assets::update_sources(
            self.fetch_metrics_references(&cual_map, unsupported_sql)
                .await
                .map_err(|e| {
                    error!("failed to fetch references for metrics -- error: {}", &e);
                    e
                })?,
            &mut self.env.metrics,
        );

        assets::update_sources(
            self.fetch_flows_references(&cual_map, unsupported_sql)
                .await
                .map_err(|e| {
                    error!("failed to fetch references for flows -- error: {}", &e);
                    e
                })?,
            &mut self.env.flows,
        );

        assets::update_sources(
            self.fetch_datasources_references(&cual_map, unsupported_sql)
                .await
                .map_err(|e| {
                    error!(
                        "failed to fetch references for datasources -- error: {}",
                        &e
                    );
                    e
                })?,
            &mut self.env.datasources,
        );

        assets::update_sources(
            self.fetch_lenses_references(&cual_map, unsupported_sql)
                .await
                .map_err(|e| {
                    error!("failed to fetch references for lenses -- error: {}", &e);
                    e
                })?,
            &mut self.env.lenses,
        );

        // View references set to be their parent cual when the jetty node is created

        Ok(())
    }
}

// graphql to vec of objects
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::get_live_tableau_connector;

    #[tokio::test]
    #[ignore]
    async fn test_fetch_data_works() -> Result<()> {
        let tab = get_live_tableau_connector().await?;
        let resolver = TableResolver {
            databases: tab.coordinator.get_databases().await?,
            database_servers: tab.coordinator.get_database_servers().await?,
        };
        let tables = tab.coordinator.get_database_tables().await?;
        let cual_map = get_table_cuals(tables, resolver);
        dbg!(cual_map);
        Ok(())
    }
}
