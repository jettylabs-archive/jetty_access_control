use anyhow::{bail, Context, Result};
use jetty_core::connectors::nodes as jetty_nodes;
use serde::Deserialize;
use std::collections::HashMap;

use super::CreateNode;

pub(crate) fn to_projects(val: &serde_json::Value) -> Result<Vec<jetty_nodes::Asset>> {
    if let serde_json::Value::Array(projects) = val {
        projects
            .iter()
            .map(|u| u.to_project())
            .collect::<Result<Vec<jetty_nodes::Asset>>>()
    } else {
        bail!["not a JSON array of project data: {:#?}", val]
    }
}

pub(crate) fn to_project(val: &serde_json::Value) -> Result<jetty_nodes::Asset> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct AssetInfo {
        name: String,
        id: String,
        description: String,
        owner: AssetOwner,
        content_permissions: String,
        updated_at: String,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct AssetOwner {
        id: String,
    }

    let asset_info: AssetInfo =
        serde_json::from_value(val.to_owned()).context("parsing asset information")?;

    let metadata = HashMap::from([
        ("project_id".to_owned(), asset_info.id),
        ("project_description".to_owned(), asset_info.description),
        ("project_owner_id".to_owned(), asset_info.owner.id),
        (
            "project_content_permissions".to_owned(),
            asset_info.content_permissions,
        ),
        ("project_update".to_owned(), asset_info.updated_at),
    ]);

    Ok(jetty_nodes::Asset {
        name: asset_info.name,
        metadata,
        ..Default::default()
    })
}
