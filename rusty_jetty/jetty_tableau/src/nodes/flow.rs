use std::collections::HashMap;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::rest::{self, Downloadable, FetchJson};

use super::FetchPermissions;

#[derive(Clone, Default, Debug, Deserialize)]
pub(crate) struct Flow {
    pub id: String,
    pub name: String,
    pub project_id: String,
    pub owner_id: String,
    pub updated_at: String,
    pub datasource_connections: Vec<String>,
    pub permissions: Vec<super::Permission>,
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
    super::to_asset_map(node, &to_node)
}

impl FetchPermissions for Flow {
    fn get_endpoint(&self) -> String {
        format!("flows/{}/permissions", self.id)
    }
}

#[cfg(test)]
mod tests {
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
}
