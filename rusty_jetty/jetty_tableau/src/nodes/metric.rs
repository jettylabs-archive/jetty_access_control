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

use super::{FromTableau, Permissionable, ProjectId, TableauAsset, METRIC};

/// Representation of Tableau metric
#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub(crate) struct Metric {
    pub id: String,
    pub name: String,
    pub updated_at: String,
    pub suspended: bool,
    pub project_id: ProjectId,
    pub owner_id: String,
    pub underlying_view_id: String,
    pub permissions: Vec<super::Permission>,
    /// HashSet of derived-from origins
    pub sources: HashSet<SourceOrigin>,
}

/// Convert JSON to a Metric struct
fn to_node(val: &serde_json::Value) -> Result<Metric> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct AssetInfo {
        name: String,
        id: String,
        updated_at: String,
        owner: super::IdField,
        project: super::IdField,
        underlying_view: super::IdField,
        suspended: bool,
    }

    let asset_info: AssetInfo =
        serde_json::from_value(val.to_owned()).context("parsing metric information")?;

    Ok(Metric {
        id: asset_info.id,
        name: asset_info.name,
        owner_id: asset_info.owner.id,
        project_id: ProjectId(asset_info.project.id),
        updated_at: asset_info.updated_at,
        suspended: asset_info.suspended,
        underlying_view_id: asset_info.underlying_view.id,
        permissions: Default::default(),
        sources: Default::default(),
    })
}

/// Get basic metric info, excluding permissions
pub(crate) async fn get_basic_metrics(
    tc: &rest::TableauRestClient,
) -> Result<HashMap<String, Metric>> {
    let node = tc
        .build_request("metrics".to_owned(), None, reqwest::Method::GET)
        .context("fetching metrics")?
        .fetch_json_response(Some(vec!["metrics".to_owned(), "metric".to_owned()]))
        .await?;
    super::to_asset_map(tc, node, &to_node)
}

impl Permissionable for Metric {
    fn get_endpoint(&self) -> String {
        format!("metrics/{}/permissions", self.id)
    }
    fn set_permissions(&mut self, permissions: Vec<super::Permission>) {
        self.permissions = permissions;
    }

    fn get_permissions(&self) -> &Vec<super::Permission> {
        &self.permissions
    }
}

impl FromTableau<Metric> for jetty_nodes::RawAsset {
    fn from(val: Metric, env: &Environment) -> Self {
        let cual = get_tableau_cual(
            TableauAssetType::Metric,
            &val.name,
            Some(&val.project_id),
            Some(&val.underlying_view_id),
            env,
        )
        .expect("Generating cual from Lens");
        let underlying_view = env
            .views
            .get(&val.underlying_view_id)
            .expect("getting metric parent view by id");
        let parent_cual = get_tableau_cual(
            TableauAssetType::View,
            &underlying_view.name,
            Some(&underlying_view.project_id),
            Some(&underlying_view.workbook_id),
            env,
        )
        .expect("getting parent cual")
        .uri();
        jetty_nodes::RawAsset::new(
            cual,
            val.name,
            AssetType(METRIC.to_owned()),
            // We will add metadata as it's useful.
            HashMap::new(),
            // Governing policies will be assigned in the policy.
            HashSet::new(),
            // Metrics are children of the underlying view.
            HashSet::from([parent_cual]),
            // Children objects will be handled in their respective nodes.
            HashSet::new(),
            // Metrics are derived from upstream tables.
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

impl TableauAsset for Metric {
    fn get_asset_type(&self) -> TableauAssetType {
        TableauAssetType::Metric
    }
}

#[async_trait]
impl HasSources for Metric {
    fn set_sources(&mut self, sources: (HashSet<SourceOrigin>, HashSet<SourceOrigin>)) {
        self.sources = sources.0;
    }
}
