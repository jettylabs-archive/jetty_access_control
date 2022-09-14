use std::collections::HashMap;

use super::{FetchPermissions, Permission};
use crate::rest::{self, FetchJson};

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Clone, Default, Debug, Deserialize)]
pub(crate) struct View {
    pub id: String,
    pub name: String,
    pub workbook_id: String,
    pub owner_id: String,
    pub project_id: String,
    pub updated_at: String,
    pub permissions: Vec<Permission>,
}

fn to_node(val: &serde_json::Value) -> Result<View> {
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
    super::to_asset_map(node, &to_node)
}

impl FetchPermissions for View {
    fn get_endpoint(&self) -> String {
        format!("views/{}/permissions", self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};

    #[tokio::test]
    async fn test_fetching_views_works() -> Result<()> {
        let tc = tokio::task::spawn_blocking(|| {
            crate::connector_setup().context("running tableau connector setup")
        })
        .await??;
        let nodes = get_basic_views(&tc.rest_client).await?;
        for (_k, v) in nodes {
            println!("{:#?}", v);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_fetching_view_permissions_works() -> Result<()> {
        let tc = tokio::task::spawn_blocking(|| {
            crate::connector_setup().context("running tableau connector setup")
        })
        .await??;
        let mut views = get_basic_views(&tc.rest_client).await?;
        for (_k, v) in &mut views {
            v.permissions = v.get_permissions(&tc.rest_client).await?;
        }
        for (_k, v) in views {
            println!("{:#?}", v);
        }
        Ok(())
    }
}
