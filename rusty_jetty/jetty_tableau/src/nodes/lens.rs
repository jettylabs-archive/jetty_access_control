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

use super::{FromTableau, Permissionable, ProjectId, TableauAsset, LENS};

/// Representation of a Tableau Lens
#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub(crate) struct Lens {
    pub id: String,
    pub name: String,
    pub datasource_id: String,
    pub project_id: ProjectId,
    pub owner_id: String,
    pub permissions: Vec<super::Permission>,
    /// HashSet of derived-from origins
    pub sources: HashSet<SourceOrigin>,
}

/// Convert JSON to a Lens struct
fn to_node(val: &serde_json::Value) -> Result<Lens> {
    #[derive(Deserialize)]
    struct AssetInfo {
        name: String,
        id: String,
        datasource_id: String,
        owner_id: String,
        project_id: String,
    }

    let asset_info: AssetInfo =
        serde_json::from_value(val.to_owned()).context("parsing lens information")?;

    Ok(Lens {
        id: asset_info.id,
        name: asset_info.name,
        owner_id: asset_info.owner_id,
        project_id: ProjectId(asset_info.project_id),
        datasource_id: asset_info.datasource_id,
        permissions: Default::default(),
        sources: Default::default(),
    })
}

/// Get basic lense information. Excludes permissions.
pub(crate) async fn get_basic_lenses(
    tc: &rest::TableauRestClient,
) -> Result<HashMap<String, Lens>> {
    let node = tc
        .build_lens_request("askdata/lenses".to_owned(), None, reqwest::Method::GET)
        .context("fetching lenses")?;

    let node = node
        .fetch_json_response(None)
        .await
        .context("fetching and parsing response")?;
    let node = rest::get_json_from_path(&node, &vec!["lenses".to_owned()])?;
    super::to_asset_map(tc, node, &to_node)
}

impl FromTableau<Lens> for jetty_nodes::RawAsset {
    fn from(val: Lens, env: &Environment) -> Self {
        let cual = get_tableau_cual(
            TableauAssetType::Lens,
            &val.name,
            Some(&val.project_id),
            Some(&val.datasource_id),
            env,
        )
        .expect("Generating cual from Lens");
        let parent_datasource = env
            .datasources
            .get(&val.datasource_id)
            .expect("getting lens parent datasource by id");
        let parent_cual = get_tableau_cual(
            TableauAssetType::Datasource,
            &parent_datasource.name,
            Some(&parent_datasource.project_id),
            None,
            env,
        )
        .expect("getting parent cual")
        .uri();
        jetty_nodes::RawAsset::new(
            cual,
            val.name,
            AssetType(LENS.to_owned()),
            // We will add metadata as it's useful.
            HashMap::new(),
            // Governing policies will be assigned in the policy.
            HashSet::new(),
            // Lenses are children of their datasources
            HashSet::from([parent_cual]),
            // Children objects will be handled in their respective nodes.
            HashSet::new(),
            // Lenses are derived from upstream tables.
            val.sources
                .into_iter()
                .map(|o| o.into_cual(env).to_string())
                .collect(),
            HashSet::new(),
            // No tags at this point.
            HashSet::new(),
        )
    }
}

impl TableauAsset for Lens {
    fn get_asset_type(&self) -> TableauAssetType {
        TableauAssetType::Lens
    }
}

impl Permissionable for Lens {
    fn get_endpoint(&self) -> String {
        format!("lenses/{}/permissions", self.id)
    }
    fn set_permissions(&mut self, permissions: Vec<super::Permission>) {
        self.permissions = permissions;
    }

    fn get_permissions(&self) -> &Vec<super::Permission> {
        &self.permissions
    }
}

#[async_trait]
impl HasSources for Lens {
    fn set_sources(&mut self, sources: (HashSet<SourceOrigin>, HashSet<SourceOrigin>)) {
        self.sources = sources.0;
    }
}
