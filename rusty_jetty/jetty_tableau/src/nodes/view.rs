use std::collections::{HashMap, HashSet};

use super::{FromTableau, Permission, Permissionable, ProjectId, TableauAsset, VIEW};
use crate::{
    coordinator::Environment,
    rest::{self, get_tableau_cual, FetchJson, TableauAssetType},
};

use anyhow::{Context, Result};
use jetty_core::connectors::{nodes as jetty_nodes, AssetType};
use serde::{Deserialize, Serialize};

/// Representation of a Tableau View
#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub(crate) struct View {
    pub id: String,
    pub name: String,
    pub workbook_id: String,
    pub owner_id: String,
    pub project_id: ProjectId,
    pub updated_at: String,
    pub permissions: Vec<Permission>,
}

/// Create a View from a JSON object
fn to_node(val: &serde_json::Value) -> Result<View> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct AssetInfo {
        name: String,
        id: String,
        owner: super::IdField,
        #[serde(default)]
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
        project_id: ProjectId(asset_info.project.id),
        updated_at: asset_info.updated_at,
        workbook_id: asset_info.workbook.id,
        permissions: Default::default(),
    })
}

/// Get basic view information (excluding permissions)
pub(crate) async fn get_basic_views(tc: &rest::TableauRestClient) -> Result<HashMap<String, View>> {
    let node = tc
        .build_request("views".to_owned(), None, reqwest::Method::GET)
        .context("fetching views")?
        .fetch_json_response(Some(vec!["views".to_owned(), "view".to_owned()]))
        .await?;
    let views = super::to_asset_map(tc, node, &to_node)?;

    // If the project ID is empty, it's in a personal space and we don't want to include it.
    let views = views
        .into_iter()
        .filter(|(_, w)| !w.project_id.0.is_empty())
        .collect();

    Ok(views)
}

impl Permissionable for View {
    fn get_endpoint(&self) -> String {
        format!("views/{}/permissions", self.id)
    }
    fn set_permissions(&mut self, permissions: Vec<super::Permission>) {
        self.permissions = permissions;
    }

    fn get_permissions(&self) -> &Vec<Permission> {
        &self.permissions
    }
}

impl FromTableau<View> for jetty_nodes::RawAsset {
    fn from(val: View, env: &Environment) -> Self {
        let cual = get_tableau_cual(
            TableauAssetType::View,
            &val.name,
            Some(&val.project_id),
            Some(&val.workbook_id),
            env,
        )
        .expect("Generating cual from Lens");
        let parent_workbook = env
            .workbooks
            .get(&val.workbook_id)
            .expect("getting view parent workbook by id");
        let parent_cual = get_tableau_cual(
            TableauAssetType::Workbook,
            &parent_workbook.name,
            Some(&parent_workbook.project_id),
            None,
            env,
        )
        .expect("getting parent cual")
        .uri();
        jetty_nodes::RawAsset::new(
            cual,
            val.name,
            AssetType(VIEW.to_owned()),
            // We will add metadata as it's useful.
            HashMap::new(),
            // Governing policies will be assigned in the policy.
            HashSet::new(),
            // Views are children of their workbooks.
            HashSet::from([parent_cual.to_owned()]),
            // Children objects will be handled in their respective nodes.
            HashSet::new(),
            // Views are derived from their parent workbooks.
            HashSet::from([parent_cual]),
            HashSet::new(),
            // No tags at this point.
            HashSet::new(),
        )
    }
}

impl TableauAsset for View {
    fn get_asset_type(&self) -> TableauAssetType {
        TableauAssetType::View
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    impl View {
        pub(crate) fn new(
            id: String,
            name: String,
            workbook_id: String,
            owner_id: String,
            project_id: ProjectId,
            updated_at: String,
            permissions: Vec<Permission>,
        ) -> Self {
            Self {
                id,
                name,
                workbook_id,
                owner_id,
                project_id,
                updated_at,
                permissions,
            }
        }
    }
}
