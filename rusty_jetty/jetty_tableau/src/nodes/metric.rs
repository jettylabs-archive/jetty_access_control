use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};
use jetty_core::{
    connectors::{nodes, AssetType},
    cual::Cual,
};
use serde::Deserialize;

use crate::rest::{self, FetchJson};

use super::FetchPermissions;

#[derive(Clone, Default, Debug, Deserialize)]
pub(crate) struct Metric {
    pub(crate) cual: Cual,
    pub id: String,
    pub name: String,
    pub updated_at: String,
    pub suspended: bool,
    pub project_id: String,
    pub owner_id: String,
    pub underlying_view_id: String,
    pub permissions: Vec<super::Permission>, // Not yet sure if this will be possible
}

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
        cual: Cual::new(format!("{}/metric/{}", tc.get_cual_prefix(), asset_info.id)),
        id: asset_info.id,
        name: asset_info.name,
        owner_id: asset_info.owner.id,
        project_id: asset_info.project.id,
        updated_at: asset_info.updated_at,
        suspended: asset_info.suspended,
        underlying_view_id: asset_info.underlying_view.id,
        permissions: Default::default(),
    })
}

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

impl Metric {
    pub(crate) fn new(
        cual: Cual,
        id: String,
        name: String,
        updated_at: String,
        suspended: bool,
        project_id: String,
        owner_id: String,
        underlying_view_id: String,
        permissions: Vec<super::Permission>,
    ) -> Self {
        Self {
            cual,
            id,
            name,
            updated_at,
            suspended,
            project_id,
            owner_id,
            underlying_view_id,
            permissions,
        }
    }
}

impl FetchPermissions for Metric {
    fn get_endpoint(&self) -> String {
        format!("metrics/{}/permissions", self.id)
    }
}

impl From<Metric> for nodes::Asset {
    fn from(val: Metric) -> Self {
        nodes::Asset::new(
            val.cual,
            val.name,
            AssetType::Other,
            // We will add metadata as it's useful.
            HashMap::new(),
            // Governing policies will be assigned in the policy.
            HashSet::new(),
            // Metrics are children of the underlying view.
            HashSet::from([val.underlying_view_id]),
            // Children objects will be handled in their respective nodes.
            HashSet::new(),
            // Metrics aren't derived from anything
            HashSet::new(),
            HashSet::new(),
            // No tags at this point.
            HashSet::new(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};

    #[tokio::test]
    async fn test_fetching_metrics_works() -> Result<()> {
        let tc = crate::connector_setup()
            .await
            .context("running tableau connector setup")?;
        let nodes = get_basic_metrics(&tc.coordinator.rest_client).await?;
        for (_k, v) in nodes {
            println!("{:#?}", v);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_fetching_metric_permissions_works() -> Result<()> {
        let tc = crate::connector_setup()
            .await
            .context("running tableau connector setup")?;
        let mut nodes = get_basic_metrics(&tc.coordinator.rest_client).await?;
        for (_k, v) in &mut nodes {
            v.permissions = v.get_permissions(&tc.coordinator.rest_client).await?;
        }
        for (_k, v) in nodes {
            println!("{:#?}", v);
        }
        Ok(())
    }

    #[test]
    fn test_asset_from_metric_works() {
        let m = Metric::new(
            Cual::new("".to_owned()),
            "id".to_owned(),
            "name".to_owned(),
            "updated_at".to_owned(),
            false,
            "project".to_owned(),
            "owner".to_owned(),
            "updated_at".to_owned(),
            vec![],
        );
        nodes::Asset::from(m);
    }

    #[test]
    fn test_metric_into_asset_works() {
        let m = Metric::new(
            Cual::new("".to_owned()),
            "id".to_owned(),
            "name".to_owned(),
            "updated_at".to_owned(),
            false,
            "project".to_owned(),
            "owner".to_owned(),
            "updated_at".to_owned(),
            vec![],
        );
        let a: nodes::Asset = m.into();
    }
}
