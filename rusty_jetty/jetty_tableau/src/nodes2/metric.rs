use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Clone)]
pub(crate) struct Metric {
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
