use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;

use crate::{
    coordinator::{Coordinator, HasSources},
    file_parse::{self, flow::FlowDoc},
    rest::{self, Downloadable, FetchJson},
};

use super::FetchPermissions;

#[derive(Clone, Default, Debug, Deserialize)]
pub(crate) struct Flow {
    pub id: String,
    pub name: String,
    pub project_id: String,
    pub owner_id: String,
    pub updated_at: String,
    pub derived_from: HashSet<String>,
    pub derived_to: HashSet<String>,
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

#[async_trait]
impl HasSources for Flow {
    fn id(&self) -> &String {
        &self.id
    }

    fn name(&self) -> &String {
        &self.name
    }

    fn updated_at(&self) -> &String {
        &self.updated_at
    }

    fn sources(&self) -> (HashSet<String>, HashSet<String>) {
        (self.derived_from.to_owned(), self.derived_from.to_owned())
    }

    async fn fetch_sources(
        &self,
        coord: &Coordinator,
    ) -> Result<(HashSet<String>, HashSet<String>)> {
        // download the source
        let archive = coord.rest_client.download(self, true).await?;
        // get the file
        let file = rest::unzip_text_file(archive, Self::match_file)?;
        // parse the file
        let flow_doc = FlowDoc::new(file)?;
        Ok(flow_doc.parse(&coord))
    }

    fn set_sources(&mut self, sources: (HashSet<String>, HashSet<String>)) {
        self.derived_from = sources.0;
        self.derived_to = sources.1;
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
        derived_from: Default::default(),
        derived_to: Default::default(),
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
