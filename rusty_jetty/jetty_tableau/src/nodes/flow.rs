use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};
use async_trait::async_trait;
use jetty_core::{
    connectors::{nodes as jetty_nodes, AssetType},
    cual::Cual,
};
use serde::{Deserialize, Serialize};

use crate::{
    coordinator::{Coordinator, HasSources},
    file_parse::flow::FlowDoc,
    rest::{self, get_tableau_cual, Downloadable, FetchJson, TableauAssetType},
};

use super::{Permissionable, ProjectId, TableauAsset};

/// Representation of a Tableau Flow
#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub(crate) struct Flow {
    pub(crate) cual: Cual,
    pub id: String,
    pub name: String,
    pub project_id: ProjectId,
    pub owner_id: String,
    pub updated_at: String,
    pub derived_from: HashSet<String>,
    pub derived_to: HashSet<String>,
    pub permissions: Vec<super::Permission>,
}

impl Flow {
    pub(crate) fn new(
        cual: Cual,
        id: String,
        name: String,
        project_id: ProjectId,
        owner_id: String,
        updated_at: String,
        derived_from: HashSet<String>,
        derived_to: HashSet<String>,
        permissions: Vec<super::Permission>,
    ) -> Self {
        Self {
            cual,
            id,
            name,
            project_id,
            owner_id,
            updated_at,
            derived_from,
            derived_to,
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
        Ok(flow_doc.parse(coord))
    }

    fn set_sources(&mut self, sources: (HashSet<String>, HashSet<String>)) {
        self.derived_from = sources.0;
        self.derived_to = sources.1;
    }
}

impl TableauAsset for Flow {
    fn get_asset_type(&self) -> TableauAssetType {
        TableauAssetType::Flow
    }
}

/// Convert JSON into a Flow struct
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
        project_id: ProjectId(asset_info.project.id),
        updated_at: asset_info.updated_at,
        permissions: Default::default(),
        derived_from: Default::default(),
        derived_to: Default::default(),
    })
}

/// Get basic information about all flows. Does not include permissions or derived_to and derived_from information.
pub(crate) async fn get_basic_flows(tc: &rest::TableauRestClient) -> Result<HashMap<String, Flow>> {
    let node = tc
        .build_request("flows".to_owned(), None, reqwest::Method::GET)
        .context("fetching flows")?
        .fetch_json_response(Some(vec!["flows".to_owned(), "flow".to_owned()]))
        .await?;
    super::to_asset_map(tc, node, &to_node)
}

impl Permissionable for Flow {
    fn get_endpoint(&self) -> String {
        format!("flows/{}/permissions", self.id)
    }
    fn set_permissions(&mut self, permissions: Vec<super::Permission>) {
        self.permissions = permissions;
    }

    fn get_permissions(&self) -> &Vec<super::Permission> {
        &self.permissions
    }
}

impl From<Flow> for jetty_nodes::Asset {
    fn from(val: Flow) -> Self {
        let ProjectId(project_id) = val.project_id;
        jetty_nodes::Asset::new(
            val.cual,
            val.name,
            AssetType::Other,
            // We will add metadata as it's useful.
            HashMap::new(),
            // Governing policies will be assigned in the policy.
            HashSet::new(),
            // Flows are children of their projects?
            HashSet::from([get_tableau_cual(TableauAssetType::Project, &project_id)
                .expect("Getting parent project CUAL")
                .uri()]),
            // Children objects will be handled in their respective nodes.
            HashSet::new(),
            // Flows are derived from their source data.
            val.derived_from,
            // Flows can also be used to create other data assets
            val.derived_to,
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
            v.update_permissions(&tc.coordinator.rest_client, &tc.coordinator.env)
                .await;
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
    #[allow(unused_must_use)]
    fn test_asset_from_flow_works() {
        set_cual_prefix("", "");
        let l = Flow::new(
            Cual::new("".to_owned()),
            "id".to_owned(),
            "name".to_owned(),
            ProjectId("project_id".to_owned()),
            "owner_id".to_owned(),
            "updated".to_owned(),
            Default::default(),
            Default::default(),
            Default::default(),
        );
        jetty_nodes::Asset::from(l);
    }

    #[test]
    #[allow(unused_must_use)]
    fn test_flow_into_asset_works() {
        set_cual_prefix("", "");
        let l = Flow::new(
            Cual::new("".to_owned()),
            "id".to_owned(),
            "name".to_owned(),
            ProjectId("project_id".to_owned()),
            "owner_id".to_owned(),
            "updated".to_owned(),
            Default::default(),
            Default::default(),
            Default::default(),
        );
        Into::<jetty_nodes::Asset>::into(l);
    }
}
