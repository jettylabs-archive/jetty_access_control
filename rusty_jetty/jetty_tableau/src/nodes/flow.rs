use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};
use jetty_core::{
    connectors::{nodes as jetty_nodes, AssetType},
    cual::Cual,
};
use serde::Deserialize;

use crate::rest::{self, get_tableau_cual, Downloadable, FetchJson, TableauAssetType};

use super::FetchPermissions;

#[derive(Clone, Default, Debug, Deserialize)]
pub(crate) struct Flow {
    pub(crate) cual: Cual,
    pub id: String,
    pub name: String,
    pub project_id: String,
    pub owner_id: String,
    pub updated_at: String,
    // needs to have input and output sources
    pub datasource_connections: Vec<String>,
    pub permissions: Vec<super::Permission>,
}

impl Flow {
    pub(crate) fn new(
        cual: Cual,
        id: String,
        name: String,
        project_id: String,
        owner_id: String,
        updated_at: String,
        datasource_connections: Vec<String>,
        permissions: Vec<super::Permission>,
    ) -> Self {
        Self {
            cual,
            id,
            name,
            project_id,
            owner_id,
            updated_at,
            datasource_connections,
            permissions,
        }
    }
}

impl Downloadable for Flow {
    fn get_path(&self) -> String {
        format!("/flows/{}/content", &self.id)
    }

    fn match_file(name: &str) -> bool {
        name == "flow"
    }
}

fn to_node(val: &serde_json::Value) -> Result<Flow> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct AssetInfo {
        name: String,
        id: String,
        updated_at: String,
        owner: super::IdField,
        project: super::IdField,
    }

    let asset_info: AssetInfo =
        serde_json::from_value(val.to_owned()).context("parsing flow information")?;

    Ok(Flow {
        cual: get_tableau_cual(TableauAssetType::Flow, &asset_info.id)?,
        id: asset_info.id,
        name: asset_info.name,
        owner_id: asset_info.owner.id,
        project_id: asset_info.project.id,
        updated_at: asset_info.updated_at,
        permissions: Default::default(),
        datasource_connections: Default::default(),
    })
}

pub(crate) async fn get_basic_flows(tc: &rest::TableauRestClient) -> Result<HashMap<String, Flow>> {
    let node = tc
        .build_request("flows".to_owned(), None, reqwest::Method::GET)
        .context("fetching flows")?
        .fetch_json_response(Some(vec!["flows".to_owned(), "flow".to_owned()]))
        .await?;
    super::to_asset_map(tc, node, &to_node)
}

impl FetchPermissions for Flow {
    fn get_endpoint(&self) -> String {
        format!("flows/{}/permissions", self.id)
    }
}

impl From<Flow> for jetty_nodes::Asset {
    fn from(val: Flow) -> Self {
        jetty_nodes::Asset::new(
            val.cual,
            val.name,
            AssetType::Other,
            // We will add metadata as it's useful.
            HashMap::new(),
            // Governing policies will be assigned in the policy.
            HashSet::new(),
            // Flows are children of their projects?
            HashSet::from(
                [get_tableau_cual(TableauAssetType::Project, &val.project_id)
                    .expect("Getting parent project CUAL")
                    .uri()],
            ),
            // Children objects will be handled in their respective nodes.
            HashSet::new(),
            // Flows are derived from their source data.
            HashSet::from_iter(val.datasource_connections.iter().map(|c| {
                get_tableau_cual(TableauAssetType::Datasource, c)
                    .expect("Getting datasource CUAL for flow")
                    .uri()
            })),
            HashSet::new(),
            // No tags at this point.
            HashSet::new(),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::rest::set_cual_prefix;

    use super::*;
    use anyhow::{Context, Result};

    #[tokio::test]
    async fn test_fetching_flows_works() -> Result<()> {
        let tc = crate::connector_setup()
            .await
            .context("running tableau connector setup")?;
        let nodes = get_basic_flows(&tc.coordinator.rest_client).await?;
        for (_k, v) in nodes {
            println!("{:#?}", v);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_fetching_flow_permissions_works() -> Result<()> {
        let tc = crate::connector_setup()
            .await
            .context("running tableau connector setup")?;
        let mut nodes = get_basic_flows(&tc.coordinator.rest_client).await?;
        for (_k, v) in &mut nodes {
            v.permissions = v.get_permissions(&tc.coordinator.rest_client).await?;
        }
        for (_k, v) in nodes {
            println!("{:#?}", v);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_downloading_flow_works() -> Result<()> {
        let tc = crate::connector_setup()
            .await
            .context("running tableau connector setup")?;
        let flows = get_basic_flows(&tc.coordinator.rest_client).await?;

        let test_flow = flows.values().next().unwrap();
        let x = tc.coordinator.rest_client.download(test_flow, true).await?;
        println!("Downloaded {} bytes", x.len());
        Ok(())
    }

    #[test]
    fn test_asset_from_flow_works() {
        set_cual_prefix("", "");
        let l = Flow::new(
            Cual::new("".to_owned()),
            "id".to_owned(),
            "name".to_owned(),
            "project_id".to_owned(),
            "owner_id".to_owned(),
            "updated".to_owned(),
            vec![],
            vec![],
        );
        jetty_nodes::Asset::from(l);
    }

    #[test]
    fn test_flow_into_asset_works() {
        set_cual_prefix("", "");
        let l = Flow::new(
            Cual::new("".to_owned()),
            "id".to_owned(),
            "name".to_owned(),
            "project_id".to_owned(),
            "owner_id".to_owned(),
            "updated".to_owned(),
            vec![],
            vec![],
        );
        let a: jetty_nodes::Asset = l.into();
    }
}
