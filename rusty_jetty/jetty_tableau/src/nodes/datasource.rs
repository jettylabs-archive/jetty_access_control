use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};
use async_trait::async_trait;
use jetty_core::connectors::{nodes as jetty_nodes, AssetType};
use serde::{Deserialize, Serialize};

use crate::{
    coordinator::{Coordinator, Environment, HasSources},
    file_parse::{origin::SourceOrigin, xml_docs},
    rest::{self, get_tableau_cual, Downloadable, FetchJson, TableauAssetType},
};

use super::{FromTableau, OwnedAsset, Permissionable, ProjectId, TableauAsset, DATASOURCE};

/// Representation of a Tableau Datasource
#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub(crate) struct Datasource {
    pub id: String,
    pub name: String,
    pub updated_at: String,
    pub project_id: ProjectId,
    pub owner_id: String,
    /// collection of origin sources
    pub sources: HashSet<SourceOrigin>,
    pub permissions: Vec<super::Permission>,
}

impl Datasource {
    pub(crate) fn new(
        id: String,
        name: String,
        updated_at: String,
        project_id: ProjectId,
        owner_id: String,
        sources: HashSet<SourceOrigin>,
        permissions: Vec<super::Permission>,
    ) -> Self {
        Self {
            id,
            name,
            updated_at,
            project_id,
            owner_id,
            sources,
            permissions,
        }
    }
}

impl Downloadable for Datasource {
    /// URI Path for asset download
    fn get_path(&self) -> String {
        format!("/datasources/{}/content", &self.id)
    }

    /// Function to match the right filenames to extract from downloaded zip
    fn match_file(name: &str) -> bool {
        name.ends_with(".tds")
    }
}

impl FromTableau<Datasource> for jetty_nodes::RawAsset {
    fn from(val: Datasource, env: &Environment) -> Self {
        let cual = get_tableau_cual(
            TableauAssetType::Datasource,
            &val.name,
            Some(&val.project_id),
            None,
            env,
        )
        .expect("Generating cual from datasource");
        let parent_cual = val
            .get_parent_project_cual(env)
            .expect("getting parent cual")
            .uri();
        jetty_nodes::RawAsset::new(
            cual,
            val.name,
            AssetType(DATASOURCE.to_owned()),
            // We will add metadata as it's useful.
            HashMap::new(),
            // Governing policies will be assigned in the policy.
            HashSet::new(),
            // Datasources are children of their projects.
            HashSet::from([parent_cual]),
            // Children objects will be handled in their respective nodes.
            HashSet::new(),
            // Datasources can be derived from other datasources.
            val.sources
                .into_iter()
                .map(|o| o.into_cual(env).to_string())
                .collect(),
            // Handled in any child datasources.
            HashSet::new(),
            // No tags at this point.
            HashSet::new(),
        )
    }
}

#[async_trait]
impl HasSources for Datasource {
    fn id(&self) -> &String {
        &self.id
    }

    fn name(&self) -> &String {
        &self.name
    }

    fn updated_at(&self) -> &String {
        &self.updated_at
    }

    fn sources(&self) -> (HashSet<SourceOrigin>, HashSet<SourceOrigin>) {
        (self.sources.to_owned(), HashSet::new())
    }

    async fn fetch_sources(
        &self,
        coord: &Coordinator,
    ) -> Result<(HashSet<SourceOrigin>, HashSet<SourceOrigin>)> {
        // download the source
        let archive = coord.rest_client.download(self, true).await?;
        // get the file
        let file = rest::unzip_text_file(archive, Self::match_file)?;
        // parse the file
        let input_sources = xml_docs::parse(&file)?;
        // datasources don't have output sources (derive_to), so just return an empty set
        let output_sources = HashSet::new();

        Ok((input_sources, output_sources))
    }

    fn set_sources(&mut self, sources: (HashSet<SourceOrigin>, HashSet<SourceOrigin>)) {
        self.sources = sources.0;
    }
}

impl TableauAsset for Datasource {
    fn get_asset_type(&self) -> TableauAssetType {
        TableauAssetType::Datasource
    }
}

/// Convert a JSON value to a Datasource node
fn to_node(val: &serde_json::Value) -> Result<super::Datasource> {
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
        serde_json::from_value(val.to_owned()).context("parsing datasource information")?;

    Ok(super::Datasource {
        id: asset_info.id,
        name: asset_info.name,
        owner_id: asset_info.owner.id,
        project_id: ProjectId(asset_info.project.id),
        updated_at: asset_info.updated_at,
        permissions: Default::default(),
        sources: Default::default(),
    })
}

/// Fetch basic datasource information. Doesn't include permissions or sources. Those need
/// to be fetched separately
pub(crate) async fn get_basic_datasources(
    tc: &rest::TableauRestClient,
) -> Result<HashMap<String, Datasource>> {
    let node = tc
        .build_request("datasources".to_owned(), None, reqwest::Method::GET)
        .context("fetching datasources")?
        .fetch_json_response(Some(vec![
            "datasources".to_owned(),
            "datasource".to_owned(),
        ]))
        .await?;
    super::to_asset_map(tc, node, &to_node)
}

impl Permissionable for Datasource {
    /// URI path to fetch datasource permissions
    fn get_endpoint(&self) -> String {
        format!("datasources/{}/permissions", self.id)
    }

    /// function to set permissions
    fn set_permissions(&mut self, permissions: Vec<super::Permission>) {
        self.permissions = permissions;
    }

    fn get_permissions(&self) -> &Vec<super::Permission> {
        &self.permissions
    }
}
