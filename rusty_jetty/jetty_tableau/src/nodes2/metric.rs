use super::*;
use anyhow::{Context, Result};
use serde::Deserialize;

fn to_node(val: &serde_json::Value) -> Result<super::Metric> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct AssetInfo {
        name: String,
        id: String,
        updated_at: String,
        owner: IdField,
        project: IdField,
        underlying_view: IdField,
        suspended: bool,
    }

    let asset_info: AssetInfo =
        serde_json::from_value(val.to_owned()).context("parsing metric information")?;

    Ok(super::Metric {
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
