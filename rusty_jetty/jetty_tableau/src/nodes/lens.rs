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
pub(crate) struct Lens {
    pub(crate) cual: Cual,
    pub id: String,
    pub name: String,
    pub datasource_id: String,
    pub project_id: String,
    pub owner_id: String,
    pub permissions: Vec<super::Permission>,
}

impl Lens {
    pub(crate) fn new(
        cual: Cual,
        id: String,
        name: String,
        datasource_id: String,
        project_id: String,
        owner_id: String,
        permissions: Vec<super::Permission>,
    ) -> Self {
        Self {
            cual,
            id,
            name,
            datasource_id,
            project_id,
            owner_id,
            permissions,
        }
    }
}

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
        cual: Cual::new(format!("{}/lens/{}", tc.get_cual_prefix(), asset_info.id)),
        id: asset_info.id,
        name: asset_info.name,
        owner_id: asset_info.owner_id,
        project_id: asset_info.project_id,
        datasource_id: asset_info.datasource_id,
        permissions: Default::default(),
    })
}
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

impl From<Lens> for nodes::Asset {
    fn from(val: Lens) -> Self {
        nodes::Asset::new(
            val.cual,
            val.name,
            AssetType::Other,
            // We will add metadata as it's useful.
            HashMap::new(),
            // Governing policies will be assigned in the policy.
            HashSet::new(),
            // Lenses are children of their projects?
            HashSet::from([val.project_id]),
            // Children objects will be handled in their respective nodes.
            HashSet::new(),
            // Lenses are derived from their source data.
            HashSet::from([val.datasource_id]),
            HashSet::new(),
            // No tags at this point.
            HashSet::new(),
        )
    }
}

impl FetchPermissions for Lens {
    fn get_endpoint(&self) -> String {
        format!("lenses/{}/permissions", self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};

    #[tokio::test]
    async fn test_fetching_lenses_works() -> Result<()> {
        let tc = crate::connector_setup()
            .await
            .context("running tableau connector setup")?;
        let nodes = get_basic_lenses(&tc.coordinator.rest_client).await?;
        for (_k, v) in nodes {
            println!("{:#?}", v);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_fetching_lens_permissions_works() -> Result<()> {
        let tc = crate::connector_setup()
            .await
            .context("running tableau connector setup")?;
        let mut nodes = get_basic_lenses(&tc.coordinator.rest_client).await?;
        for (_k, v) in &mut nodes {
            v.permissions = v.get_permissions(&tc.coordinator.rest_client).await?;
        }
        for (_k, v) in nodes {
            println!("{:#?}", v);
        }
        Ok(())
    }

    #[test]
    fn test_asset_from_lens_works() {
        let l = Lens::new(
            Cual::new("".to_owned()),
            "id".to_owned(),
            "name".to_owned(),
            "datasource_id".to_owned(),
            "project_id".to_owned(),
            "owner_id".to_owned(),
            vec![],
        );
        nodes::Asset::from(l);
    }

    #[test]
    fn test_lens_into_asset_works() {
        let l = Lens::new(
            Cual::new("".to_owned()),
            "id".to_owned(),
            "name".to_owned(),
            "datasource_id".to_owned(),
            "project_id".to_owned(),
            "owner_id".to_owned(),
            vec![],
        );
        let a: nodes::Asset = l.into();
    }
}
