use std::collections::HashMap;

use super::{FetchPermissions, Permission};
use crate::rest::{self, FetchJson};

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Clone, Default, Debug, Deserialize)]
pub(crate) struct Project {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub parent_project_id: Option<String>,
    pub controlling_permissions_project_id: Option<String>,
    pub permissions: Vec<Permission>,
}

fn to_node(val: &serde_json::Value) -> Result<super::Project> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct ProjectInfo {
        name: String,
        id: String,
        owner: super::IdField,
        parent_project_id: Option<String>,
        controlling_permissions_project_id: Option<String>,
        updated_at: String,
    }

    let project_info: ProjectInfo =
        serde_json::from_value(val.to_owned()).context("parsing asset information")?;

    Ok(super::Project {
        id: project_info.id,
        name: project_info.name,
        owner_id: project_info.owner.id,
        parent_project_id: project_info.parent_project_id,
        controlling_permissions_project_id: project_info.controlling_permissions_project_id,
        permissions: Default::default(),
    })
}

pub(crate) async fn get_basic_projects(
    tc: &rest::TableauRestClient,
) -> Result<HashMap<String, Project>> {
    let node = tc
        .build_request("projects".to_owned(), None, reqwest::Method::GET)
        .context("fetching projects")?
        .fetch_json_response(Some(vec!["projects".to_owned(), "project".to_owned()]))
        .await?;
    super::to_asset_map(node, &to_node)
}

impl FetchPermissions for Project {
    fn get_endpoint(&self) -> String {
        format!("projects/{}/permissions", self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};

    #[tokio::test]
    async fn test_fetching_projects_works() -> Result<()> {
        let tc = tokio::task::spawn_blocking(|| {
            crate::connector_setup().context("running tableau connector setup")
        })
        .await??;
        let nodes = get_basic_projects(&tc.env.rest_client).await?;
        for (_k, v) in nodes {
            println!("{:#?}", v);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_fetching_project_permissions_works() -> Result<()> {
        let tc = tokio::task::spawn_blocking(|| {
            crate::connector_setup().context("running tableau connector setup")
        })
        .await??;
        let mut nodes = get_basic_projects(&tc.env.rest_client).await?;
        for (_k, v) in &mut nodes {
            v.permissions = v.get_permissions(&tc.env.rest_client).await?;
        }
        for (_k, v) in nodes {
            println!("{:#?}", v);
        }
        Ok(())
    }
}
