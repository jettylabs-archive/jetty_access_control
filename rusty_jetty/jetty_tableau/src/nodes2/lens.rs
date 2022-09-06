use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Clone)]
pub(crate) struct Lens {
    pub id: String,
    pub name: String,
    pub datasource_id: String,
    pub project_id: String,
    pub owner_id: String,
    pub permissions: Vec<super::Permission>,
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
        id: asset_info.id,
        name: asset_info.name,
        owner_id: asset_info.owner_id,
        project_id: asset_info.project_id,
        datasource_id: asset_info.datasource_id,
        permissions: Default::default(),
    })
}
