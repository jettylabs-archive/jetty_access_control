use std::collections::{HashMap, HashSet};

use super::{FetchPermissions, Permission};
use crate::rest::{self, FetchJson};

use anyhow::{Context, Result};
use jetty_core::{
    connectors::{nodes, AssetType},
    cual::Cual,
};
use serde::Deserialize;

#[derive(Clone, Default, Debug, Deserialize)]
pub(crate) struct View {
    cual: Cual,
    pub id: String,
    pub name: String,
    pub workbook_id: String,
    pub owner_id: String,
    pub project_id: String,
    pub updated_at: String,
    pub permissions: Vec<Permission>,
}

fn to_node(tc: &rest::TableauRestClient, val: &serde_json::Value) -> Result<View> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct AssetInfo {
        name: String,
        id: String,
        owner: super::IdField,
        project: super::IdField,
        workbook: super::IdField,
        updated_at: String,
    }

    let asset_info: AssetInfo =
        serde_json::from_value(val.to_owned()).context("parsing view information")?;

    Ok(View {
        cual: Cual::new(format!("{}/view/{}", tc.get_cual_prefix(), asset_info.id)),
        id: asset_info.id,
        name: asset_info.name,
        owner_id: asset_info.owner.id,
        project_id: asset_info.project.id,
        updated_at: asset_info.updated_at,
        workbook_id: asset_info.workbook.id,
        permissions: Default::default(),
    })
}

pub(crate) async fn get_basic_views(tc: &rest::TableauRestClient) -> Result<HashMap<String, View>> {
    let node = tc
        .build_request("views".to_owned(), None, reqwest::Method::GET)
        .context("fetching views")?
        .fetch_json_response(Some(vec!["views".to_owned(), "view".to_owned()]))
        .await?;
    super::to_asset_map(tc, node, &to_node)
}

impl FetchPermissions for View {
    fn get_endpoint(&self) -> String {
        format!("views/{}/permissions", self.id)
    }
}

impl View {
    pub(crate) fn new(
        cual: Cual,
        id: String,
        name: String,
        workbook_id: String,
        owner_id: String,
        project_id: String,
        updated_at: String,
        permissions: Vec<Permission>,
    ) -> Self {
        Self {
            cual,
            id,
            name,
            workbook_id,
            owner_id,
            project_id,
            updated_at,
            permissions,
        }
    }
}

impl From<View> for nodes::Asset {
    fn from(val: View) -> Self {
        nodes::Asset::new(
            val.cual,
            val.name,
            AssetType::Other,
            // We will add metadata as it's useful.
            HashMap::new(),
            // Governing policies will be assigned in the policy.
            HashSet::new(),
            // Views are children of their workbooks.
            HashSet::from([val.workbook_id]),
            // Children objects will be handled in their respective nodes.
            HashSet::new(),
            // Views are not derived from/to anything.
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
    async fn test_fetching_views_works() -> Result<()> {
        let tc = crate::connector_setup()
            .await
            .context("running tableau connector setup")?;
        let nodes = get_basic_views(&tc.coordinator.rest_client).await?;
        for (_k, v) in nodes {
            println!("{:#?}", v);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_fetching_view_permissions_works() -> Result<()> {
        let tc = crate::connector_setup()
            .await
            .context("running tableau connector setup")?;
        let mut views = get_basic_views(&tc.coordinator.rest_client).await?;
        for (_k, v) in &mut views {
            v.permissions = v.get_permissions(&tc.coordinator.rest_client).await?;
        }
        for (_k, v) in views {
            println!("{:#?}", v);
        }
        Ok(())
    }

    #[test]
    fn test_asset_from_view_works() {
        let v = View::new(
            Cual::new("".to_owned()),
            "id".to_owned(),
            "name".to_owned(),
            "workbook_id".to_owned(),
            "owner_id".to_owned(),
            "project_id".to_owned(),
            "updated_at".to_owned(),
            vec![],
        );
        nodes::Asset::from(v);
    }

    #[test]
    fn test_view_into_asset_works() {
        let v = View::new(
            Cual::new("".to_owned()),
            "id".to_owned(),
            "name".to_owned(),
            "workbook_id".to_owned(),
            "owner_id".to_owned(),
            "project_id".to_owned(),
            "updated_at".to_owned(),
            vec![],
        );
        let a: nodes::Asset = v.into();
    }
}
