use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Clone, Default)]
pub(crate) struct Workbook {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub project_id: String,
    pub datasource_connections: String,
    pub datasources: Vec<String>,
    pub updated_at: String,
    pub permissions: Vec<super::Permission>,
}

fn to_node(val: &serde_json::Value) -> Result<Workbook> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct WorkbookInfo {
        name: String,
        id: String,
        owner: super::IdField,
        project: super::IdField,
        updated_at: String,
    }

    let workbook_info: WorkbookInfo =
        serde_json::from_value(val.to_owned()).context("parsing workbook information")?;

    Ok(Workbook {
        id: workbook_info.id,
        name: workbook_info.name,
        owner_id: workbook_info.owner.id,
        project_id: workbook_info.project.id,
        updated_at: workbook_info.updated_at,
        datasource_connections: Default::default(),
        datasources: Default::default(),
        permissions: Default::default(),
    })
}
