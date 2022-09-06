use anyhow::{Context, Result};
use serde::Deserialize;

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
