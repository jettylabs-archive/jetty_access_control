mod snowflake;

use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, bail, Context, Result};
use jetty_core::logging::debug;
use serde::Deserialize;

use super::{origin::SourceOrigin, RelationType};
use crate::{
    coordinator::Coordinator,
    nodes::ProjectId,
    rest::{TableauAssetType, TableauRestClient},
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
        .ok_or_else(|| anyhow!("unable to get sql type"))?;
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
    pub(crate) fn parse(
        &self,
        coord: &Coordinator,
    ) -> (HashSet<SourceOrigin>, HashSet<SourceOrigin>) {
        let mut input_origins: HashSet<SourceOrigin> = HashSet::new();
        let mut output_origins: HashSet<SourceOrigin> = HashSet::new();

        for node in self.nodes.values() {
            if let Some(node_type) = node.get("nodeType").and_then(|v| v.as_str()) {
                match node_type {
                    // Input nodes
                    ".v1.LoadSql" => {
                        self.handle_load_sql(node).map_or_else(
                            |e| {
                                debug!(
                                    "skipping data input source of type: {}\nerror: {}",
                                    node_type, e
                                )
                            },
                            |v| input_origins.extend(v),
                        );
                    }
                    ".v2019_3_1.LoadSqlProxy" => {
                        self.handle_load_sql_proxy(node, &coord.env, &coord.rest_client)
                            .map_or_else(
                                |e| {
                                    debug!(
                                        "skipping data input source of type: {}\nerror: {}",
                                        node_type, e
                                    )
                                },
                                |v| input_origins.extend(v),
                            );
                    }
                    // Output nodes
                    ".v1.PublishExtract" => {
                        self.handle_publish_extract(node, &coord.env, &coord.rest_client)
                            .map_or_else(
                                |e| {
                                    debug!(
                                        "skipping data output destination of type: {}\nerror: {}",
                                        node_type, e
                                    )
                                },
                                |v| {
                                    output_origins.insert(v);
                                },
                            );
                    }
                    ".v2020_3_1.WriteToDatabase" => {
                        self.handle_write_to_database(node).map_or_else(
                            |e| {
                                debug!(
                                    "skipping data output destination of type: {}\nerror: {}",
                                    node_type, e
                                )
                            },
                            |v| output_origins.extend(v),
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

        (input_origins, output_origins)
    }

    fn handle_load_sql(&self, node: &serde_json::Value) -> Result<HashSet<SourceOrigin>> {
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
        Ok(cuals.into_iter().map(SourceOrigin::from_cual).collect())
    }

    fn handle_load_sql_proxy(
        &self,
        node: &serde_json::Value,
        env: &crate::coordinator::Environment,
        _client: &TableauRestClient,
    ) -> Result<HashSet<SourceOrigin>> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct ConnectionAttributes {
            project_name: String,
            datasource_name: String,
        }

        let mut origins = HashSet::new();

        let connection_attributes = node
            .get("connectionAttributes")
            .ok_or_else(|| anyhow!["unable to find connectionAttributes"])?;

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

        origins.insert(SourceOrigin::from_id_type(
            TableauAssetType::Datasource,
            correct_datasource[0].id.clone(),
        ));

        Ok(origins)
    }

    fn handle_write_to_database(&self, node: &serde_json::Value) -> Result<HashSet<SourceOrigin>> {
        // First check the class or type of database, and then the type of sql object.
        // From there, fetch the cuals
        if let Ok(class) = self.get_node_connection_class(node) {
            let cuals = match &class[..] {
                "snowflake" => snowflake::get_output_table_cuals(self, node)?
                    .into_iter()
                    .map(SourceOrigin::from_cual)
                    .collect(),
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
    ) -> Result<SourceOrigin> {
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
            .filter(|(_, v)| v.project_id.0 == conn.project_luid && v.name == conn.datasource_name)
            .map(|(_, v)| v)
            .collect();

        if correct_datasource.len() != 1 {
            bail!("unable to find linked datasource; this can happen if the flow has not run");
        }

        Ok(SourceOrigin::from_id_type(
            TableauAssetType::Datasource,
            correct_datasource[0].id.clone(),
        ))
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
    use jetty_core::cual::Cual;

    use crate::{
        file_parse::origin::SourceOrigin,
        nodes::{Datasource, Project, ProjectId},
        rest::{self, TableauAssetType},
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

        let _cual_prefix = rest::get_cual_prefix()?;
        let doc = super::FlowDoc::new(data).unwrap();
        let (input_origins, output_origins) = doc.parse(&coord);
        assert_eq!(input_origins,
            [
                SourceOrigin::from_id_type(TableauAssetType::Datasource,"d99c9c85-a525-4cce-beaa-7ebcda1ea577".to_owned()),
                SourceOrigin::from_cual(Cual::new("snowflake://cea26391.snowflakecomputing.com/JETTY_TEST_DB/GOLD/IRIS_JOINED_TABLE")),
                SourceOrigin::from_cual(Cual::new("snowflake://cea26391.snowflakecomputing.com/JETTY_TEST_DB/SILVER/SILVER_ADULT_AGGREGATED_VIEW")),
                SourceOrigin::from_cual(Cual::new("snowflake://cea26391.snowflakecomputing.com/JETTY_TEST_DB/GOLD/ADULT_AGGREGATED_VIEW")),
                SourceOrigin::from_id_type(TableauAssetType::Datasource,"5f6df88d-aeb2-4551-a4c1-e326a45f4b91".to_owned()),
                SourceOrigin::from_cual(Cual::new("snowflake://cea26391.snowflakecomputing.com/JETTY_TEST_DB/GOLD/IRIS_JOINED_VIEW")),
                SourceOrigin::from_cual(Cual::new("snowflake://cea26391.snowflakecomputing.com/JETTY_TEST_DB/GOLD/ADULT_AGGREGATED_TABLE")),
            ]
        .into_iter()
        .collect::<HashSet<SourceOrigin>>());

        let mut output_origins = output_origins.into_iter().collect::<Vec<_>>();
        output_origins.sort();
        let mut expected_origins = [
                SourceOrigin::from_id_type(TableauAssetType::Datasource,"6df04a18-19a6-4012-8a83-c2b33a8d1907".to_owned()),
                SourceOrigin::from_id_type(TableauAssetType::Datasource,"91dae170-0191-4dba-8cef-5eda957bf122".to_owned()),
                SourceOrigin::from_id_type(TableauAssetType::Datasource,"de1c1844-2ce6-480d-8016-afc7be49827e".to_owned()),
            SourceOrigin::from_cual(Cual::new("snowflake://cea26391.snowflakecomputing.com/JETTY_TEST_DB/GOLD/tableau_special"
                )),
            SourceOrigin::from_cual(Cual::new(
            "snowflake://cea26391.snowflakecomputing.com/JETTY_TEST_DB/RAW/%22Special%20Name%22"
                )),
                SourceOrigin::from_id_type(TableauAssetType::Datasource,"a27d260d-9ff9-4707-82fd-e66cda23275d".to_owned()),
        ]
        .into_iter()
        .collect::<Vec<SourceOrigin>>();
        expected_origins.sort();
        assert_eq!(output_origins, expected_origins);

        Ok(())
    }
}
