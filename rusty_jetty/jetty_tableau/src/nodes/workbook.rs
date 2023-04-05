use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    coordinator::{Environment, HasSources},
    origin::SourceOrigin,
    rest::{self, get_tableau_cual, FetchJson, TableauAssetType},
};

use jetty_core::{
    connectors::{nodes as jetty_nodes, AssetType},
    logging::debug,
};

use super::{FromTableau, OwnedAsset, Permissionable, ProjectId, TableauAsset, WORKBOOK};

/// Representation of Tableau Workbook
#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub(crate) struct Workbook {
    pub id: String,
    /// Unqualified name of the workbook
    pub name: String,
    /// Tableau LUID of owner
    pub owner_id: String,
    /// LUID of project
    pub project_id: ProjectId,
    /// HashSet of derived-from origins
    pub sources: HashSet<SourceOrigin>,
    pub updated_at: String,
    pub permissions: Vec<super::Permission>,
}

impl Permissionable for Workbook {
    fn get_endpoint(&self) -> String {
        format!("workbooks/{}/permissions", self.id)
    }
    fn set_permissions(&mut self, permissions: Vec<super::Permission>) {
        self.permissions = permissions;
    }

    fn get_permissions(&self) -> &Vec<super::Permission> {
        &self.permissions
    }
}

impl TableauAsset for Workbook {
    fn get_asset_type(&self) -> TableauAssetType {
        TableauAssetType::Workbook
    }
}

#[async_trait]
impl HasSources for Workbook {
    fn set_sources(&mut self, sources: (HashSet<SourceOrigin>, HashSet<SourceOrigin>)) {
        self.sources = sources.0;
    }
}

impl FromTableau<Workbook> for jetty_nodes::RawAsset {
    fn from(val: Workbook, env: &Environment) -> Self {
        let cual = get_tableau_cual(
            TableauAssetType::Workbook,
            &val.name,
            Some(&val.project_id),
            None,
            env,
        )
        .expect("Generating cual from workbook");
        let parent_cual = val
            .get_parent_project_cual(env)
            .expect("getting parent cual")
            .uri();
        jetty_nodes::RawAsset::new(
            cual,
            val.name,
            AssetType(WORKBOOK.to_owned()),
            // We will add metadata as it's useful.
            HashMap::new(),
            // Governing policies will be assigned in the policy.
            HashSet::new(),
            // Workbooks are children of their projects.
            HashSet::from([parent_cual]),
            // Children objects will be handled in their respective nodes.
            HashSet::new(),
            // Workbooks are derived from their source data.
            val.sources
                .into_iter()
                .map(|o| o.into_cual(env).to_string())
                .collect(),
            HashSet::new(),
            // No tags at this point.
            HashSet::new(),
        )
    }
}

fn to_node(val: &serde_json::Value) -> Result<Workbook> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct WorkbookInfo {
        name: String,
        id: String,
        owner: super::IdField,
        #[serde(default)]
        project: super::IdField,
        updated_at: String,
    }

    let workbook_info: WorkbookInfo =
        serde_json::from_value(val.to_owned()).context("parsing workbook information")?;

    Ok(Workbook {
        id: workbook_info.id,
        name: workbook_info.name,
        owner_id: workbook_info.owner.id,
        project_id: ProjectId(workbook_info.project.id),
        updated_at: workbook_info.updated_at,
        permissions: Default::default(),
        sources: Default::default(),
    })
}

pub(crate) async fn get_basic_workbooks(
    tc: &rest::TableauRestClient,
) -> Result<HashMap<String, Workbook>> {
    let node = tc
        .build_request("workbooks".to_owned(), None, reqwest::Method::GET)
        .context("fetching workbooks")?
        .fetch_json_response(Some(vec!["workbooks".to_owned(), "workbook".to_owned()]))
        .await?;
    let asset_map = super::to_asset_map(tc, node, &to_node)?;

    debug!("Fetched {} workbooks", &asset_map.len());

    // If the project ID is empty, it's in a personal space and we don't want to include it.
    let asset_map: HashMap<String, Workbook> = asset_map
        .into_iter()
        .filter(|(_, w)| !w.project_id.0.is_empty())
        .collect();

    debug!("Filtered to {} workbooks", &asset_map.len());

    Ok(asset_map)
}

#[cfg(test)]
mod tests {
    use crate::nodes::Permission;

    use super::*;

    impl Workbook {
        #[allow(clippy::too_many_arguments)]
        pub(crate) fn new(
            id: String,
            name: String,
            owner_id: String,
            project_id: ProjectId,
            sources: HashSet<SourceOrigin>,
            updated_at: String,
            permissions: Vec<Permission>,
        ) -> Self {
            Self {
                id,
                name,
                owner_id,
                project_id,
                sources,
                updated_at,
                permissions,
            }
        }
    }
}
