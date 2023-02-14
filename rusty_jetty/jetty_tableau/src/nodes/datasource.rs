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

use super::{FromTableau, OwnedAsset, Permissionable, ProjectId, TableauAsset, DATASOURCE};

/// Representation of a Tableau Datasource
#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub(crate) struct Datasource {
    pub id: String,
    pub name: String,
    pub updated_at: String,
    pub project_id: ProjectId,
    pub owner_id: String,
    /// collection of origin sources
    pub sources: HashSet<SourceOrigin>,
    pub permissions: Vec<super::Permission>,
}

impl FromTableau<Datasource> for jetty_nodes::RawAsset {
    fn from(val: Datasource, env: &Environment) -> Self {
        let cual = get_tableau_cual(
            TableauAssetType::Datasource,
            &val.name,
            Some(&val.project_id),
            None,
            env,
        )
        .expect("Generating cual from datasource");
        let parent_cual = val
            .get_parent_project_cual(env)
            .expect("getting parent cual")
            .uri();
        jetty_nodes::RawAsset::new(
            cual,
            val.name,
            AssetType(DATASOURCE.to_owned()),
            // We will add metadata as it's useful.
            HashMap::new(),
            // Governing policies will be assigned in the policy.
            HashSet::new(),
            // Datasources are children of their projects.
            HashSet::from([parent_cual]),
            // Children objects will be handled in their respective nodes.
            HashSet::new(),
            // Datasources can be derived from other datasources.
            val.sources
                .into_iter()
                .map(|o| o.into_cual(env).to_string())
                .collect(),
            // Handled in any child datasources.
            HashSet::new(),
            // No tags at this point.
            HashSet::new(),
        )
    }
}

#[async_trait]
impl HasSources for Datasource {
    fn set_sources(&mut self, sources: (HashSet<SourceOrigin>, HashSet<SourceOrigin>)) {
        self.sources = sources.0;
    }
}

impl TableauAsset for Datasource {
    fn get_asset_type(&self) -> TableauAssetType {
        TableauAssetType::Datasource
    }
}

/// Convert a JSON value to a Datasource node
fn to_node(val: &serde_json::Value) -> Result<super::Datasource> {
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
        serde_json::from_value(val.to_owned()).context("parsing datasource information")?;

    Ok(super::Datasource {
        id: asset_info.id,
        name: asset_info.name,
        owner_id: asset_info.owner.id,
        project_id: ProjectId(asset_info.project.id),
        updated_at: asset_info.updated_at,
        permissions: Default::default(),
        sources: Default::default(),
    })
}

/// Fetch basic datasource information. Doesn't include permissions or sources. Those need
/// to be fetched separately
pub(crate) async fn get_basic_datasources(
    tc: &rest::TableauRestClient,
) -> Result<HashMap<String, Datasource>> {
    let node = tc
        .build_request("datasources".to_owned(), None, reqwest::Method::GET)
        .context("fetching datasources")?
        .fetch_json_response(Some(vec![
            "datasources".to_owned(),
            "datasource".to_owned(),
        ]))
        .await?;
    super::to_asset_map(tc, node, &to_node)
}

impl Permissionable for Datasource {
    /// URI path to fetch datasource permissions
    fn get_endpoint(&self) -> String {
        format!("datasources/{}/permissions", self.id)
    }

    /// function to set permissions
    fn set_permissions(&mut self, permissions: Vec<super::Permission>) {
        self.permissions = permissions;
    }

    fn get_permissions(&self) -> &Vec<super::Permission> {
        &self.permissions
    }
}
