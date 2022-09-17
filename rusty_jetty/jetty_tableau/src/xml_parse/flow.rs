use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;

use crate::rest::TableauRestClient;

#[derive(Deserialize, Debug)]
struct FlowDoc {
    nodes: HashMap<String, serde_json::Value>,
    connections: HashMap<String, serde_json::Value>,
}

impl FlowDoc {
    fn new(data: String) -> Result<Self> {
        serde_json::from_str::<FlowDoc>(&data).context("parsing flow document")
    }

    fn get_input_cuals(
        &self,
        env: &crate::coordinator::Environment,
        client: &TableauRestClient,
    ) -> HashSet<String> {
        let mut cuals = HashSet::new();
        // first, get the cuals from datasources
        if let Ok(new_cuals) = self.get_datasource_input_cuals(env, client) {
            cuals.extend(new_cuals);
        } else {
            println!("Error fetching Flow -> Datasource connections");
        }

        // next add table cuals
        let table_nodes = self.get_sql_table_nodes();
        for (_, node) in table_nodes {
            if let Some(class) = self.get_node_connection_class(node) {
                match &class[..] {
                    "snowflake" => {
                        if let Ok(new_cuals) = snowflake::get_table_cuals(self, node) {
                            cuals.extend(new_cuals);
                        } else {
                            println!("Error fetching Flow -> Table connections")
                        }
                    }
                    c => println!("unsupported node class <{}>", c),
                }
            } else {
                println!("unable to get datasource class")
            }
        }

        // finally, add query cuals
        let table_nodes = self.get_custom_sql_nodes();
        for (_, node) in table_nodes {
            if let Some(class) = self.get_node_connection_class(node) {
                match &class[..] {
                    "snowflake" => {
                        if let Ok(new_cuals) = snowflake::get_query_cuals(self, node) {
                            cuals.extend(new_cuals);
                        } else {
                            println!("Error fetching Flow -> Table connections")
                        }
                    }
                    c => println!("unsupported node class <{}>", c),
                }
            } else {
                println!("unable to get datasource class")
            }
        }

        cuals
    }

    fn get_input_nodes(&self) -> HashMap<String, &serde_json::Value> {
        self.nodes
            .iter()
            .filter(|(k, v)| {
                v.get("baseType") == Some(&serde_json::Value::String("input".to_owned()))
            })
            .map(|(k, v)| (k.to_owned(), v))
            .collect()
    }

    fn get_sql_nodes(&self) -> HashMap<String, &serde_json::Value> {
        self.get_input_nodes()
            .iter()
            .filter(|(k, v)| {
                v.get("nodeType").unwrap_or(&serde_json::Value::default())
                    == &serde_json::Value::String(".v1.LoadSql".to_owned())
            })
            .map(|(k, v)| (k.to_owned(), *v))
            .collect()
    }

    fn get_custom_sql_nodes(&self) -> HashMap<String, &serde_json::Value> {
        self.get_sql_nodes()
            .iter()
            .filter(|(k, v)| {
                v.get("relation")
                    .unwrap_or(&serde_json::Value::default())
                    .get("type")
                    .unwrap_or(&serde_json::Value::default())
                    == &serde_json::Value::String("query".to_owned())
            })
            .map(|(k, v)| (k.to_owned(), *v))
            .collect()
    }

    fn get_sql_table_nodes(&self) -> HashMap<String, &serde_json::Value> {
        self.get_sql_nodes()
            .iter()
            .filter(|(k, v)| {
                v.get("relation")
                    .unwrap_or(&serde_json::Value::default())
                    .get("type")
                    .unwrap_or(&serde_json::Value::default())
                    == &serde_json::Value::String("table".to_owned())
            })
            .map(|(k, v)| (k.to_owned(), *v))
            .collect()
    }

    fn get_datasource_input_cuals(
        &self,
        env: &crate::coordinator::Environment,
        client: &TableauRestClient,
    ) -> Result<HashSet<String>> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct ConnectionAttributes {
            project_name: String,
            datasource_name: String,
        }

        // Get the relevant nodes
        let nodes: Vec<_> = self
            .get_input_nodes()
            .iter()
            .filter(|(k, v)| {
                v.get("nodeType")
                    == Some(&serde_json::Value::String(
                        ".v2019_3_1.LoadSqlProxy".to_owned(),
                    ))
            })
            .map(|(k, v)| *v)
            .collect();

        let mut cuals = HashSet::new();

        // Iterate through the nodes and link them to the relevant datasources
        for node in nodes {
            let connection_attributes = node
                .get("connectionAttributes")
                .ok_or(anyhow!["unable to find connectionAttributes"])?;

            let conn: ConnectionAttributes =
                serde_json::from_value(connection_attributes.to_owned())?;

            // get project_id
            let correct_projects: Vec<_> = env
                .projects
                .iter()
                .filter(|(_, v)| v.name == conn.project_name)
                .map(|(k, _)| k)
                .collect();

            let correct_datasource: Vec<_> = env
                .datasources
                .iter()
                .filter(|(_, v)| {
                    correct_projects.contains(&&v.project_id.to_owned())
                        && v.name == conn.datasource_name
                })
                .map(|(_, v)| v)
                .collect();

            if correct_datasource.len() != 1 {
                bail!("unable to find linked datasource");
            }

            cuals.insert(format!(
                "{}{}",
                client.get_cual_prefix(),
                correct_datasource[0].cual_suffix()
            ));
        }

        Ok(cuals)
    }

    fn get_output_nodes(&self) -> HashMap<String, &serde_json::Value> {
        self.nodes
            .iter()
            .filter(|(k, v)| {
                v.get("baseType") == Some(&serde_json::Value::String("output".to_owned()))
            })
            .map(|(k, v)| (k.to_owned(), v))
            .collect()
    }

    fn get_datasource_output_cuals(
        &self,
        env: crate::coordinator::Environment,
        client: TableauRestClient,
    ) -> Result<HashSet<String>> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct ConnectionAttributes {
            datasource_name: String,
            project_luid: String,
        }

        // Get the relevant nodes
        let nodes: Vec<_> = self
            .get_output_nodes()
            .iter()
            .filter(|(k, v)| {
                v.get("nodeType")
                    == Some(&serde_json::Value::String(".v1.PublishExtract".to_owned()))
            })
            .map(|(k, v)| *v)
            .collect();

        let mut cuals = HashSet::new();
        // Iterate through the nodes and link them to the relevant datasources
        for node in nodes {
            let conn: ConnectionAttributes = serde_json::from_value(node.to_owned())?;

            let correct_datasource: Vec<_> = env
                .datasources
                .iter()
                .filter(|(_, v)| {
                    v.project_id.to_owned() == conn.project_luid && v.name == conn.datasource_name
                })
                .map(|(_, v)| v)
                .collect();

            if correct_datasource.len() != 1 {
                bail!("unable to find linked datasource");
            }

            cuals.insert(format!(
                "{}{}",
                client.get_cual_prefix(),
                correct_datasource[0].cual_suffix()
            ));
        }

        Ok(cuals)
    }

    fn get_database_output_nodes(&self) -> HashMap<String, &serde_json::Value> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct ConnectionAttributes {
            datasource_name: String,
            project_luid: String,
        }

        // Get the relevant nodes
        self.get_output_nodes()
            .iter()
            .filter(|(k, v)| {
                v.get("nodeType")
                    == Some(&serde_json::Value::String(
                        ".v2020_3_1.WriteToDatabase".to_owned(),
                    ))
            })
            .map(|(k, v)| (k.to_owned(), *v))
            .collect()
    }

    fn get_node_connection_class(&self, node: &serde_json::Value) -> Option<String> {
        let conn_id = if let Some(connection_id) = node.get("connectionId") {
            connection_id.as_str().unwrap()
        } else {
            return None;
        };

        let attributes = if let Some(attributes) =
            self.connections[&conn_id.to_owned()].get("connectionAttributes")
        {
            attributes
        } else {
            return None;
        };

        if let Some(class) = attributes.get("class") {
            class.as_str().and_then(|s| Some(s.to_owned()))
        } else {
            None
        }
    }
}

mod snowflake {
    use std::collections::{HashMap, HashSet};

    use anyhow::{anyhow, Result};
    use serde::Deserialize;

    use super::FlowDoc;

    #[derive(Deserialize)]
    struct ConnectionAttributes {
        schema: String,
        dbname: String,
    }

    fn get_server_info(doc: &FlowDoc, connection_id: &String) -> Result<String> {
        Ok(doc.connections[connection_id]
            .get("connectionAttributes")
            .and_then(|v| v.get("server"))
            .and_then(|v| v.as_str())
            .unwrap()
            .to_owned())
    }

    pub(super) fn get_table_cuals(
        doc: &FlowDoc,
        node: &serde_json::Value,
    ) -> Result<HashSet<String>> {
        #[derive(Deserialize)]
        struct TableRelation {
            table: String,
        }

        #[derive(Deserialize)]
        struct TableInfo {
            #[serde(rename = "connectionAttributes")]
            connection_attributes: ConnectionAttributes,
            #[serde(rename = "connectionId")]
            connection_id: String,
            relation: TableRelation,
        }

        let table_info: TableInfo = serde_json::from_value(node.to_owned())?;
        let server = get_server_info(doc, &table_info.connection_id)?;

        let snowflake_table = crate::xml_parse::snowflake::SnowflakeTableInfo {
            table: table_info.relation.table,
            connection: table_info.connection_id.to_owned(),
        };
        let connections = HashMap::from([(
            table_info.connection_id.to_owned(),
            crate::xml_parse::NamedConnection::Snowflake(
                crate::xml_parse::snowflake::SnowflakeConnectionInfo {
                    name: table_info.connection_id,
                    db: table_info.connection_attributes.dbname,
                    server,
                    schema: table_info.connection_attributes.schema,
                },
            ),
        )]);
        let mut cuals = HashSet::new();
        cuals.extend(snowflake_table.to_cuals(&connections)?);

        Ok(cuals)
    }

    fn get_output_cuals(
        doc: &FlowDoc,
        nodes: HashMap<String, serde_json::Value>,
    ) -> Result<HashSet<String>> {
        #[derive(Deserialize)]
        struct OutputDbAttributes {
            schema: String,
            dbname: String,
            warehouse: String,
            tablename: String,
        }

        #[derive(Deserialize)]
        struct TableInfo {
            attributes: OutputDbAttributes,
            #[serde(rename = "connectionId")]
            connection_id: String,
        }

        let mut relations = HashSet::new();

        for (_, v) in nodes {
            let table_info: TableInfo = serde_json::from_value(v.to_owned())?;
            let server = get_server_info(doc, &table_info.connection_id)?;

            let table = table_info.attributes.tablename;
            // Fix up the table name:
            if table.starts_with('"') {
                let table = table.trim_matches('"');
            } else if table.starts_with("'") {
                let table = table.trim_matches('\'');
            } else if table.starts_with('[') {
                let table = table.trim_matches('[');
                let table = table.trim_matches(']');
            } else if table.starts_with('`') {
                let table = table.trim_matches('`');
            }

            let snowflake_table = crate::xml_parse::snowflake::SnowflakeTableInfo {
                table,
                connection: table_info.connection_id.to_owned(),
            };
            let connections = HashMap::from([(
                table_info.connection_id.to_owned(),
                crate::xml_parse::NamedConnection::Snowflake(
                    crate::xml_parse::snowflake::SnowflakeConnectionInfo {
                        name: table_info.connection_id,
                        db: table_info.attributes.dbname,
                        server,
                        schema: table_info.attributes.schema,
                    },
                ),
            )]);

            relations.extend(snowflake_table.to_cuals(&connections)?);
        }
        Ok(relations)
    }

    pub(super) fn get_query_cuals(
        doc: &FlowDoc,
        node: &serde_json::Value,
    ) -> Result<HashSet<String>> {
        #[derive(Deserialize)]
        struct QueryRelation {
            query: String,
        }

        #[derive(Deserialize)]
        struct QueryInfo {
            #[serde(rename = "connectionAttributes")]
            connection_attributes: ConnectionAttributes,
            #[serde(rename = "connectionId")]
            connection_id: String,
            relation: QueryRelation,
        }

        let mut relations = HashSet::new();

        let table_info: QueryInfo = serde_json::from_value(node.to_owned())?;
        let server = get_server_info(doc, &table_info.connection_id)?;

        let snowflake_table = crate::xml_parse::snowflake::SnowflakeQueryInfo {
            query: table_info.relation.query,
            connection: table_info.connection_id.to_owned(),
        };
        let connections = HashMap::from([(
            table_info.connection_id.to_owned(),
            crate::xml_parse::NamedConnection::Snowflake(
                crate::xml_parse::snowflake::SnowflakeConnectionInfo {
                    name: table_info.connection_id,
                    db: table_info.connection_attributes.dbname,
                    server,
                    schema: table_info.connection_attributes.schema,
                },
            ),
        )]);

        relations.extend(snowflake_table.to_cuals(&connections)?);
        Ok(relations)
    }
}

#[cfg(test)]
mod test {
    use std::fs;

    use anyhow::Result;

    #[test]
    fn parse_flow_works() -> Result<()> {
        let data = fs::read_to_string("test_data/flow2".to_owned()).unwrap();

        let coord = crate::coordinator::Coordinator::new(crate::TableauCredentials {
            username: "isaac@get-jetty.com".to_owned(),
            password: "".to_owned(),
            server_name: "10ax.online.tableau.com".to_owned(),
            site_name: "jettydev".to_owned(),
        });

        let doc = super::FlowDoc::new(data).unwrap();
        let cuals = doc.get_input_cuals(&coord.env, &coord.rest_client);
        dbg!(cuals);
        Ok(())
    }
}
