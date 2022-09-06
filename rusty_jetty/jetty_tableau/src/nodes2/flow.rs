use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Clone)]
pub(crate) struct Flow {
    pub id: String,
    pub name: String,
    pub project_id: String,
    pub owner_id: String,
    pub updated_at: String,
    pub datasource_connections: Vec<String>,
    pub permissions: Vec<super::Permission>,
}

fn to_node(val: &serde_json::Value) -> Result<Flow> {
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
        serde_json::from_value(val.to_owned()).context("parsing flow information")?;

    Ok(Flow {
        id: asset_info.id,
        name: asset_info.name,
        owner_id: asset_info.owner.id,
        project_id: asset_info.project.id,
        updated_at: asset_info.updated_at,
        permissions: Default::default(),
        datasource_connections: Default::default(),
    })
}
