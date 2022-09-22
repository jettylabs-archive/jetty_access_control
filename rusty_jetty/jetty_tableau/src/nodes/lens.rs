use std::collections::HashMap;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::rest::{self, FetchJson};

use super::Permissionable;

/// Representation of a Tableau Lens
#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub(crate) struct Lens {
    pub id: String,
    pub name: String,
    pub datasource_id: String,
    pub project_id: String,
    pub owner_id: String,
    pub permissions: Vec<super::Permission>,
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
        project_id: asset_info.project_id,
        datasource_id: asset_info.datasource_id,
        permissions: Default::default(),
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
    super::to_asset_map(node, &to_node)
}

impl Permissionable for Lens {
    fn get_endpoint(&self) -> String {
        format!("lenses/{}/permissions", self.id)
    }
    fn set_permissions(&mut self, permissions: Vec<super::Permission>) {
        self.permissions = permissions;
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
            v.update_permissions(&tc.coordinator.rest_client).await;
        }
        for (_k, v) in nodes {
            println!("{:#?}", v);
        }
        Ok(())
    }
}
