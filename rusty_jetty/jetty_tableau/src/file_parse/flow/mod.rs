mod snowflake;

use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, bail, Context, Result};
use jetty_core::logging::{debug, error};
use serde::Deserialize;

use super::RelationType;
use crate::{
    coordinator::Coordinator,
    nodes::ProjectId,
    rest::{get_tableau_cual, TableauAssetType, TableauRestClient},
};

#[derive(Deserialize, Debug)]
pub(crate) struct FlowDoc {
    nodes: HashMap<String, serde_json::Value>,
    connections: HashMap<String, serde_json::Value>,
}

fn get_relation_type(node: &serde_json::Value) -> Result<RelationType> {
    let sql_type = node
        .get("relation")
        .and_then(|n| n.get("type"))
        .and_then(|n| n.as_str())
        .ok_or(anyhow!("unable to get sql type"))?;
    Ok(match sql_type {
        "query" => RelationType::SqlQuery,
        "table" => RelationType::Table,
        _ => bail!("unknown sql type"),
    })
}

impl FlowDoc {
    /// Create a new FlowDoc
    pub(crate) fn new(data: String) -> Result<Self> {
        serde_json::from_str::<FlowDoc>(&data).context("parsing flow document")
    }

    /// Parse a flow document. This MUST be run after projects and datasources have been
    /// fetched.
    pub(crate) fn parse(&self, coord: &Coordinator) -> (HashSet<String>, HashSet<String>) {
        let mut input_cuals: HashSet<String> = HashSet::new();
        let mut output_cuals: HashSet<String> = HashSet::new();

        for (_, node) in &self.nodes {
            if let Some(node_type) = node.get("nodeType").and_then(|v| v.as_str()) {
                match node_type {
                    // Input nodes
                    ".v1.LoadSql" => {
                        self.handle_load_sql(node).map_or_else(
                            |e| {
                                error!(
                                    "skipping data input source of type: {}\nerror: {}",
                                    node_type, e
                                )
                            },
                            |v| input_cuals.extend(v),
                        );
                    }
                    ".v2019_3_1.LoadSqlProxy" => {
                        self.handle_load_sql_proxy(node, &coord.env, &coord.rest_client)
                            .map_or_else(
                                |e| {
                                    error!(
                                        "skipping data input source of type: {}\nerror: {}",
                                        node_type, e
                                    )
                                },
                                |v| input_cuals.extend(v),
                            );
                    }
                    // Output nodes
                    ".v1.PublishExtract" => {
                        self.handle_publish_extract(node, &coord.env, &coord.rest_client)
                            .map_or_else(
                                |e| {
                                    error!(
                                        "skipping data output destination of type: {}\nerror: {}",
                                        node_type, e
                                    )
                                },
                                |v| output_cuals.extend(v),
                            );
                    }
                    ".v2020_3_1.WriteToDatabase" => {
                        self.handle_write_to_database(node).map_or_else(
                            |e| {
                                error!(
                                    "skipping data output destination of type: {}\nerror: {}",
                                    node_type, e
                                )
                            },
                            |v| output_cuals.extend(v),
                        );
                    }
                    o => {
                        if let Some(base_type) = node.get("baseType").and_then(|v| v.as_str()) {
                            match base_type {
                                "input" => debug!("ignoring input node of type: {}", o),
                                "ouput" => debug!("ignoring output node of type: {}", o),
                                _ => (),
                            }
                        }
                    }
                }
            } else {
                debug!("unable to get nodeType for a node")
            }
        }

        (input_cuals, output_cuals)
    }

    fn handle_load_sql(&self, node: &serde_json::Value) -> Result<HashSet<String>> {
        // First check the class or type of database, and then the type of sql object.
        // From there, fetch the cuals
        let class = self
            .get_node_connection_class(node)
            .context("unable to get connection class")?;

        let cuals = match &class[..] {
            "snowflake" => match get_relation_type(node)? {
                RelationType::SqlQuery => snowflake::get_input_query_cuals(self, node),
                RelationType::Table => snowflake::get_input_table_cuals(self, node),
            }?,
            o => bail!("we don't currently support {}", o),
        };

        Ok(cuals)
    }

    fn handle_load_sql_proxy(
        &self,
        node: &serde_json::Value,
        env: &crate::coordinator::Environment,
        _client: &TableauRestClient,
    ) -> Result<HashSet<String>> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct ConnectionAttributes {
            project_name: String,
            datasource_name: String,
        }

        let mut cuals = HashSet::new();

        let connection_attributes = node
            .get("connectionAttributes")
            .ok_or(anyhow!["unable to find connectionAttributes"])?;

        let conn: ConnectionAttributes = serde_json::from_value(connection_attributes.to_owned())?;

        // get project_id
        let correct_projects: Vec<_> = env
            .projects
            .iter()
            .filter(|(_, v)| v.name == conn.project_name)
            .map(|(k, _)| k)
            .collect();

        // get datasources from the relevant project(s)
        let correct_datasource: Vec<_> = env
            .datasources
            .iter()
            .filter(|(_, v)| {
                let ProjectId(project_id) = &v.project_id;
                correct_projects.contains(&&project_id.to_owned()) && v.name == conn.datasource_name
            })
            .map(|(_, v)| v)
            .collect();

        if correct_datasource.len() != 1 {
            bail!(
                "unable to find linked datasource; found {} possible matches.",
                correct_datasource.len()
            );
        }

        cuals.insert(
            get_tableau_cual(TableauAssetType::Datasource, &correct_datasource[0].id)?.uri(),
        );

        Ok(cuals)
    }

    fn handle_write_to_database(&self, node: &serde_json::Value) -> Result<HashSet<String>> {
        // First check the class or type of database, and then the type of sql object.
        // From there, fetch the cuals
        if let Ok(class) = self.get_node_connection_class(node) {
            let cuals = match &class[..] {
                "snowflake" => snowflake::get_output_table_cuals(self, node)?,
                o => bail!("we don't currently support {}", o),
            };

            Ok(cuals)
        } else {
            bail!("unable to get connection class");
        }
    }

    fn handle_publish_extract(
        &self,
        node: &serde_json::Value,
        env: &crate::coordinator::Environment,
        _client: &TableauRestClient,
    ) -> Result<HashSet<String>> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct ConnectionAttributes {
            datasource_name: String,
            project_luid: String,
        }

        let conn: ConnectionAttributes = serde_json::from_value(node.to_owned())?;

        let correct_datasource: Vec<_> = env
            .datasources
            .iter()
            .filter(|(_, v)| {
                &v.project_id.0 == &conn.project_luid && v.name == conn.datasource_name
            })
            .map(|(_, v)| v)
            .collect();

        if correct_datasource.len() != 1 {
            bail!("unable to find linked datasource; this can happen if the flow has not run");
        }

        Ok(HashSet::from([get_tableau_cual(
            TableauAssetType::Datasource,
            &correct_datasource[0].id,
        )?
        .uri()]))
    }

    fn get_node_connection_class(&self, node: &serde_json::Value) -> Result<String> {
        let conn_id = if let Some(connection_id) = node.get("connectionId") {
            connection_id.as_str().unwrap()
        } else {
            bail!("unable to find connection");
        };

        Ok(self
            .connections
            .get(&conn_id.to_owned())
            .and_then(|c| c.get("connectionAttributes"))
            .and_then(|a| a.get("class"))
            .and_then(|class| class.as_str())
            .ok_or_else(|| anyhow!["unable to fetch class"])?
            .to_owned())
    }
}

#[cfg(test)]
mod test {
    use std::{
        collections::{HashMap, HashSet},
        fs,
    };

    use anyhow::Result;

    use crate::{
        nodes::{Datasource, Project, ProjectId},
        rest::{self},
    };

    #[test]
    fn new_parse_flow_works() -> Result<()> {
        let data = fs::read_to_string("test_data/flow2").unwrap();

        let mut coord = crate::coordinator::Coordinator::new_dummy();

        coord.env.projects = HashMap::from([
            (
                "81db10c8-1c14-462f-996f-4bff60f982fa".to_owned(),
                Project {
                    id: ProjectId("81db10c8-1c14-462f-996f-4bff60f982fa".to_owned()),
                    name: "Isaac's Project".to_owned(),
                    ..Default::default()
                },
            ),
            (
                "c585b0f7-fc43-4d0a-8942-12adf443ee98".to_owned(),
                Project {
                    id: ProjectId("c585b0f7-fc43-4d0a-8942-12adf443ee98".to_owned()),
                    name: "Isaac's Project".to_owned(),
                    ..Default::default()
                },
            ),
            (
                "c9726ebd-9c86-4169-83de-5872354edc8c".to_owned(),
                Project {
                    id: ProjectId("c9726ebd-9c86-4169-83de-5872354edc8c".to_owned()),
                    name: "Samples".to_owned(),
                    ..Default::default()
                },
            ),
        ]);

        coord.env.datasources = HashMap::from([
            (
                "a27d260d-9ff9-4707-82fd-e66cda23275d".to_owned(),
                Datasource {
                    id: "a27d260d-9ff9-4707-82fd-e66cda23275d".to_owned(),
                    name: "Output With Custom Query".to_owned(),
                    project_id: ProjectId("c585b0f7-fc43-4d0a-8942-12adf443ee98".to_owned()),
                    ..Default::default()
                },
            ),
            (
                "5f6df88d-aeb2-4551-a4c1-e326a45f4b91".to_owned(),
                Datasource {
                    id: "5f6df88d-aeb2-4551-a4c1-e326a45f4b91".to_owned(),
                    name: "Just sql - different db".to_owned(),
                    project_id: ProjectId("c585b0f7-fc43-4d0a-8942-12adf443ee98".to_owned()),
                    ..Default::default()
                },
            ),
            (
                "d99c9c85-a525-4cce-beaa-7ebcda1ea577".to_owned(),
                Datasource {
                    id: "d99c9c85-a525-4cce-beaa-7ebcda1ea577".to_owned(),
                    name: "multi-connection".to_owned(),
                    project_id: ProjectId("c585b0f7-fc43-4d0a-8942-12adf443ee98".to_owned()),
                    ..Default::default()
                },
            ),
            (
                "6df04a18-19a6-4012-8a83-c2b33a8d1907".to_owned(),
                Datasource {
                    id: "6df04a18-19a6-4012-8a83-c2b33a8d1907".to_owned(),
                    name: "Test Flow Output".to_owned(),
                    project_id: ProjectId("c585b0f7-fc43-4d0a-8942-12adf443ee98".to_owned()),
                    ..Default::default()
                },
            ),
            (
                "de1c1844-2ce6-480d-8016-afc7be49827e".to_owned(),
                Datasource {
                    id: "de1c1844-2ce6-480d-8016-afc7be49827e".to_owned(),
                    name: "Output 4 - Table".to_owned(),
                    project_id: ProjectId("c585b0f7-fc43-4d0a-8942-12adf443ee98".to_owned()),
                    ..Default::default()
                },
            ),
            (
                "91dae170-0191-4dba-8cef-5eda957bf122".to_owned(),
                Datasource {
                    id: "91dae170-0191-4dba-8cef-5eda957bf122".to_owned(),
                    name: "Output 3".to_owned(),
                    project_id: ProjectId("c585b0f7-fc43-4d0a-8942-12adf443ee98".to_owned()),
                    ..Default::default()
                },
            ),
        ]);

        let cual_prefix = rest::get_cual_prefix()?;
        let doc = super::FlowDoc::new(data).unwrap();
        let (input_cuals, output_cuals) = doc.parse(&coord);
        assert_eq!(input_cuals,
            [
                format!["{}/datasource/d99c9c85-a525-4cce-beaa-7ebcda1ea577", &cual_prefix],
                "snowflake://cea26391.snowflakecomputing.com/JETTY_TEST_DB/GOLD/IRIS_JOINED_TABLE".to_owned(),
                "snowflake://cea26391.snowflakecomputing.com/JETTY_TEST_DB/SILVER/SILVER_ADULT_AGGREGATED_VIEW".to_owned(),
                "snowflake://cea26391.snowflakecomputing.com/JETTY_TEST_DB/GOLD/ADULT_AGGREGATED_VIEW".to_owned(),
                format!["{}/datasource/5f6df88d-aeb2-4551-a4c1-e326a45f4b91", &cual_prefix],
                "snowflake://cea26391.snowflakecomputing.com/JETTY_TEST_DB/GOLD/IRIS_JOINED_VIEW".to_owned(),
                "snowflake://cea26391.snowflakecomputing.com/JETTY_TEST_DB/GOLD/ADULT_AGGREGATED_TABLE".to_owned(),
            ]
        .into_iter()
        .collect::<HashSet<String>>());

        assert_eq!(
            output_cuals,
            [
                format!(
                    "{}/datasource/6df04a18-19a6-4012-8a83-c2b33a8d1907",
                    &cual_prefix
                ),
                format!(
                    "{}/datasource/91dae170-0191-4dba-8cef-5eda957bf122",
                    &cual_prefix
                ),
                format!(
                    "{}/datasource/de1c1844-2ce6-480d-8016-afc7be49827e",
                    &cual_prefix
                ),
                "snowflake://cea26391.snowflakecomputing.com/JETTY_TEST_DB/GOLD/TABLEAU_SPECIAL"
                    .to_owned(),
                "snowflake://cea26391.snowflakecomputing.com/JETTY_TEST_DB/RAW/Special%20Name"
                    .to_owned(),
                format!(
                    "{}/datasource/a27d260d-9ff9-4707-82fd-e66cda23275d",
                    &cual_prefix
                ),
            ]
            .into_iter()
            .collect::<HashSet<String>>()
        );

        Ok(())
    }
}
