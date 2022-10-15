use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    coordinator::{Coordinator, HasSources},
    file_parse::xml_docs,
    rest::{self, get_tableau_cual, Downloadable, FetchJson, TableauAssetType},
};

use jetty_core::{
    connectors::{nodes as jetty_nodes, AssetType},
    cual::Cual,
};

use super::{Permissionable, ProjectId, TableauAsset, WORKBOOK};

/// Representation of Tableau Workbook
#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub(crate) struct Workbook {
    pub(crate) cual: Cual,
    pub id: String,
    /// Unqualified name of the workbook
    pub name: String,
    /// Tableau LUID of owner
    pub owner_id: String,
    /// LUID of project
    pub project_id: ProjectId,
    /// Probably not necessary?
    pub has_embedded_sources: bool,
    /// HashSet of derived-from cuals
    pub sources: HashSet<String>,
    pub updated_at: String,
    pub permissions: Vec<super::Permission>,
}

impl Workbook {
    pub(crate) fn new(
        cual: Cual,
        id: String,
        name: String,
        owner_id: String,
        project_id: ProjectId,
        has_embedded_sources: bool,
        sources: HashSet<String>,
        updated_at: String,
        permissions: Vec<super::Permission>,
    ) -> Self {
        Self {
            cual,
            id,
            name,
            owner_id,
            project_id,
            has_embedded_sources,
            sources,
            updated_at,
            permissions,
        }
    }
}

impl Downloadable for Workbook {
    fn get_path(&self) -> String {
        format!("/workbooks/{}/content", &self.id)
    }

    fn match_file(name: &str) -> bool {
        name.ends_with(".twb")
    }
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
        coord: &Coordinator,
    ) -> Result<(HashSet<String>, HashSet<String>)> {
        // download the source
        let archive = coord.rest_client.download(self, true).await?;
        // get the file
        let file = rest::unzip_text_file(archive, Self::match_file)?;
        // parse the file
        let input_sources = xml_docs::parse(&file)?;
        let output_sources = HashSet::new();

        Ok((input_sources, output_sources))
    }

    fn set_sources(&mut self, sources: (HashSet<String>, HashSet<String>)) {
        self.sources = sources.0;
    }
}

impl From<Workbook> for jetty_nodes::Asset {
    fn from(val: Workbook) -> Self {
        let ProjectId(project_id) = val.project_id;
        jetty_nodes::Asset::new(
            val.cual,
            val.name,
            AssetType(WORKBOOK.to_owned()),
            // We will add metadata as it's useful.
            HashMap::new(),
            // Governing policies will be assigned in the policy.
            HashSet::new(),
            // Workbooks are children of their projects.
            HashSet::from([get_tableau_cual(TableauAssetType::Project, &project_id)
                .unwrap()
                .uri()]),
            // Children objects will be handled in their respective nodes.
            HashSet::new(),
            // Workbooks are derived from their source data.
            val.sources,
            HashSet::new(),
            // No tags at this point.
            HashSet::new(),
        )
    }
}

/// Take a JSON object returned from a GraphQL query and turn it into a notebook
fn to_node_graphql(val: &serde_json::Value) -> Result<Workbook> {
    #[derive(Deserialize)]
    struct LuidField {
        luid: String,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct EmbeddedSourceHelper {
        name: String,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct WorkbookInfo {
        name: String,
        luid: String,
        owner: LuidField,
        project_luid: String,
        updated_at: String,
        embedded_datasources: Vec<EmbeddedSourceHelper>,
    }

    // Get inner result data

    let workbook_info: WorkbookInfo =
        serde_json::from_value(val.to_owned()).context("parsing workbook information")?;

    Ok(Workbook {
        cual: get_tableau_cual(TableauAssetType::Workbook, &workbook_info.luid)?,
        id: workbook_info.luid,
        name: workbook_info.name,
        owner_id: workbook_info.owner.luid,
        project_id: ProjectId(workbook_info.project_luid),
        updated_at: workbook_info.updated_at,
        has_embedded_sources: !workbook_info.embedded_datasources.is_empty(),
        sources: Default::default(),
        permissions: Default::default(),
    })
}

/// Get basic workbook information. This uses a GraphQL query to get the essentials
pub(crate) async fn get_basic_workbooks(
    tc: &rest::TableauRestClient,
) -> Result<HashMap<String, Workbook>> {
    let query = r#"
    query workbooks {
        workbooks {
          updatedAt
          name
          luid
          owner {
            luid
          }
          projectLuid
          embeddedDatasources {
            id
            name
          }
        }
      }"#
    .to_owned();
    let node = tc
        .build_graphql_request(query)
        .context("fetching workbooks")?
        .fetch_json_response(None)
        .await?;
    let node = rest::get_json_from_path(&node, &vec!["data".to_owned(), "workbooks".to_owned()])?;
    super::to_asset_map(tc, node, &to_node_graphql)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Ok, Result};
    use jetty_core::logging::debug;

    #[tokio::test]
    async fn test_fetching_workbooks_works() -> Result<()> {
        let tc = crate::connector_setup()
            .await
            .context("running tableau connector setup")?;
        let groups = get_basic_workbooks(&tc.coordinator.rest_client).await?;
        for (_k, v) in groups {
            debug!("{:#?}", v);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_downloading_workbook_works() -> Result<()> {
        let tc = crate::connector_setup()
            .await
            .context("running tableau connector setup")?;
        let workbooks = get_basic_workbooks(&tc.coordinator.rest_client).await?;

        let test_workbook = workbooks.values().next().unwrap();
        let x = tc
            .coordinator
            .rest_client
            .download(test_workbook, true)
            .await?;
        debug!("Downloaded {} bytes", x.len());
        Ok(())
    }

    #[tokio::test]
    async fn test_fetching_workbook_permissions_works() -> Result<()> {
        let tc = crate::connector_setup()
            .await
            .context("running tableau connector setup")?;
        let mut workbooks = get_basic_workbooks(&tc.coordinator.rest_client).await?;
        for (_k, v) in &mut workbooks {
            v.update_permissions(&tc.coordinator.rest_client, &tc.coordinator.env)
                .await?;
        }
        for (_k, v) in workbooks {
            debug!("{:#?}", v);
        }
        Ok(())
    }

    #[test]
    #[allow(unused_must_use)]
    fn test_asset_from_workbook_works() {
        let wb = Workbook::new(
            Cual::new("".to_owned()),
            "id".to_owned(),
            "name".to_owned(),
            "owner_id".to_owned(),
            ProjectId("project_id".to_owned()),
            false,
            HashSet::new(),
            "updated_at".to_owned(),
            vec![],
        );
        jetty_nodes::Asset::from(wb);
    }

    #[test]
    #[allow(unused_must_use)]
    fn test_workbook_into_asset_works() {
        let wb = Workbook::new(
            Cual::new("".to_owned()),
            "id".to_owned(),
            "name".to_owned(),
            "owner_id".to_owned(),
            ProjectId("project_id".to_owned()),
            false,
            HashSet::new(),
            "updated_at".to_owned(),
            vec![],
        );
        Into::<jetty_nodes::Asset>::into(wb);
    }
}
