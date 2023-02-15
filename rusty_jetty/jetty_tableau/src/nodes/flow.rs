use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};
use async_trait::async_trait;
use jetty_core::connectors::{nodes as jetty_nodes, AssetType};
use serde::{Deserialize, Serialize};

use crate::{
    coordinator::{Environment, HasSources},
    origin::SourceOrigin,
    rest::{self, get_tableau_cual, FetchJson, TableauAssetType},
};

use super::{FromTableau, OwnedAsset, Permissionable, ProjectId, TableauAsset, FLOW};

/// Representation of a Tableau Flow
#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub(crate) struct Flow {
    pub id: String,
    pub name: String,
    pub project_id: ProjectId,
    pub owner_id: String,
    pub updated_at: String,
    pub(crate) derived_from: HashSet<SourceOrigin>,
    pub(crate) derived_to: HashSet<SourceOrigin>,
    pub permissions: Vec<super::Permission>,
}

#[async_trait]
impl HasSources for Flow {
    fn set_sources(&mut self, sources: (HashSet<SourceOrigin>, HashSet<SourceOrigin>)) {
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

impl FromTableau<Flow> for jetty_nodes::RawAsset {
    fn from(val: Flow, env: &Environment) -> Self {
        let cual = get_tableau_cual(
            TableauAssetType::Flow,
            &val.name,
            Some(&val.project_id),
            None,
            env,
        )
        .expect("Generating cual from flow");
        let parent_cual = val
            .get_parent_project_cual(env)
            .expect("getting parent cual")
            .uri();
        jetty_nodes::RawAsset::new(
            cual,
            val.name,
            AssetType(FLOW.to_owned()),
            // We will add metadata as it's useful.
            HashMap::new(),
            // Governing policies will be assigned in the policy.
            HashSet::new(),
            // Flows are children of their projects
            HashSet::from([parent_cual]),
            // Children objects will be handled in their respective nodes.
            HashSet::new(),
            // Flows are derived from their source data.
            val.derived_from
                .into_iter()
                .map(|o| o.into_cual(env).to_string())
                .collect(),
            // Flows can also be used to create other data assets
            val.derived_to
                .into_iter()
                .map(|o| o.into_cual(env).to_string())
                .collect(),
            // No tags at this point.
            HashSet::new(),
        )
    }
}
