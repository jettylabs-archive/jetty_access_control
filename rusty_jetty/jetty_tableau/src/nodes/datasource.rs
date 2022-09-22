use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};
use async_trait::async_trait;
use jetty_core::{
    connectors::{nodes as jetty_nodes, AssetType},
    cual::Cual,
};
use serde::Deserialize;

use crate::{
    coordinator::HasSources,
    file_parse::xml_docs,
    rest::{self, get_tableau_cual, Downloadable, FetchJson, TableauAssetType, TableauRestClient},
};

use super::FetchPermissions;

#[derive(Clone, Default, Debug, Deserialize)]
pub(crate) struct Datasource {
    pub(crate) cual: Cual,
    pub id: String,
    pub name: String,
    pub datasource_type: String,
    pub updated_at: String,
    pub project_id: String,
    pub owner_id: String,
    pub sources: HashSet<String>,
    pub permissions: Vec<super::Permission>,
    /// Vec of origin cuals
    pub derived_from: Vec<String>,
}

impl Datasource {
    pub(crate) fn new(
        cual: Cual,
        id: String,
        name: String,
        datasource_type: String,
        updated_at: String,
        project_id: String,
        owner_id: String,
        sources: HashSet<String>,
        permissions: Vec<super::Permission>,
        derived_from: Vec<String>,
    ) -> Self {
        Self {
            cual,
            id,
            name,
            datasource_type,
            updated_at,
            project_id,
            owner_id,
            sources,
            permissions,
            derived_from,
        }
    }

    pub(crate) fn cual_suffix(&self) -> String {
        format!("/datasource/{}", &self.id)
    }
}

impl Downloadable for Datasource {
    fn get_path(&self) -> String {
        format!("/datasources/{}/content", &self.id)
    }

    fn match_file(name: &str) -> bool {
        name.ends_with(".tds")
    }
}

impl From<Datasource> for jetty_nodes::Asset {
    fn from(val: Datasource) -> Self {
        jetty_nodes::Asset::new(
            val.cual,
            val.name,
            AssetType::Other,
            // We will add metadata as it's useful.
            HashMap::new(),
            // Governing policies will be assigned in the policy.
            HashSet::new(),
            // Datasources are children of their projects.
            HashSet::from(
                [get_tableau_cual(TableauAssetType::Project, &val.project_id)
                    .expect("Getting parent project for datasource")
                    .uri()],
            ),
            // Children objects will be handled in their respective nodes.
            HashSet::new(),
            // Datasources can be derived from other datasources.
            val.sources,
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

    fn sources(&self) -> (HashSet<String>, HashSet<String>) {
        (self.sources.to_owned(), HashSet::new())
    }

    async fn fetch_sources(
        &self,
        client: &TableauRestClient,
    ) -> Result<(HashSet<String>, HashSet<String>)> {
        // download the source
        let archive = client.download(self, true).await?;
        // get the file
        let file = rest::unzip_text_file(archive, Self::match_file)?;
        // parse the file
        let input_sources = xml_docs::parse(&file)?;
        let output_sources = HashSet::new();

        dbg!(&input_sources);

        Ok((input_sources, output_sources))
    }

    fn set_sources(&mut self, sources: (HashSet<String>, HashSet<String>)) {
        self.sources = sources.0;
    }
}

fn to_node(val: &serde_json::Value) -> Result<super::Datasource> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct AssetInfo {
        name: String,
        id: String,
        updated_at: String,
        #[serde(rename = "type")]
        datasource_type: String,
        owner: super::IdField,
        project: super::IdField,
    }

    let asset_info: AssetInfo =
        serde_json::from_value(val.to_owned()).context("parsing datasource information")?;

    Ok(super::Datasource {
        cual: get_tableau_cual(TableauAssetType::Datasource, &asset_info.id)?,
        id: asset_info.id,
        name: asset_info.name,
        owner_id: asset_info.owner.id,
        project_id: asset_info.project.id,
        updated_at: asset_info.updated_at,
        datasource_type: asset_info.datasource_type,
        permissions: Default::default(),
        sources: Default::default(),
        derived_from: Default::default(),
    })
}
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

impl FetchPermissions for Datasource {
    fn get_endpoint(&self) -> String {
        format!("datasources/{}/permissions", self.id)
    }
}

#[cfg(test)]
mod tests {

    use crate::rest::set_cual_prefix;

    use super::*;
    use anyhow::{Context, Result};

    #[tokio::test]
    async fn test_fetching_flows_works() -> Result<()> {
        let tc = crate::connector_setup()
            .await
            .context("running tableau connector setup")?;
        let nodes = get_basic_datasources(&tc.coordinator.rest_client).await?;
        for (_, v) in nodes {
            println!("{:#?}", v);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_fetching_datasource_permissions_works() -> Result<()> {
        let tc = crate::connector_setup()
            .await
            .context("running tableau connector setup")?;
        let mut nodes = get_basic_datasources(&tc.coordinator.rest_client).await?;
        for (_, v) in &mut nodes {
            v.permissions = v.get_permissions(&tc.coordinator.rest_client).await?;
        }
        for (_, v) in nodes {
            println!("{:#?}", v);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_downloading_datasource_works() -> Result<()> {
        let tc = crate::connector_setup()
            .await
            .context("running tableau connector setup")?;
        let datasources = get_basic_datasources(&tc.coordinator.rest_client).await?;

        let test_datasource = datasources.values().next().unwrap();
        let x = tc
            .coordinator
            .rest_client
            .download(test_datasource, true)
            .await?;
        println!("Downloaded {} bytes", x.len());
        Ok(())
    }

    #[tokio::test]
    async fn test_fetching_sources_for_datasource_works() -> Result<()> {
        let tc = crate::connector_setup()
            .await
            .context("running tableau connector setup")?;
        let datasources = get_basic_datasources(&tc.coordinator.rest_client).await?;

        for test_datasource in datasources.values() {
            let x = test_datasource
                .fetch_sources(&tc.coordinator.rest_client)
                .await?;
        }
        Ok(())
    }

    #[test]
    fn test_asset_from_datasource_works() {
        set_cual_prefix("", "");
        let ds = Datasource::new(
            Cual::new("".to_owned()),
            "id".to_owned(),
            "name".to_owned(),
            "datasource_type".to_owned(),
            "updated".to_owned(),
            "project_id".to_owned(),
            "owner_id".to_owned(),
            HashSet::new(),
            vec![],
            vec![],
        );
        jetty_nodes::Asset::from(ds);
    }

    #[test]
    fn test_datasource_into_asset_works() {
        set_cual_prefix("", "");
        let ds = Datasource::new(
            Cual::new("".to_owned()),
            "id".to_owned(),
            "name".to_owned(),
            "datasource_type".to_owned(),
            "updated".to_owned(),
            "project_id".to_owned(),
            "owner_id".to_owned(),
            HashSet::new(),
            vec![],
            vec![],
        );
        let a: jetty_nodes::Asset = ds.into();
    }
}
