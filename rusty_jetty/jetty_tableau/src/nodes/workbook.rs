use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};
use serde::Deserialize;

use super::FetchPermissions;
use crate::rest::{self, Downloadable, FetchJson};

use jetty_core::{
    connectors::{nodes, AssetType},
    cual::Cual,
};

#[derive(Clone, Default, Debug, Deserialize)]
pub(crate) struct Workbook {
    cual: Cual,
    pub id: String,
    /// Unqualified name of the workbook
    pub name: String,
    /// Tableau LUID of owner
    pub owner_id: String,
    /// LUID of project
    pub project_id: String,
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
        project_id: String,
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

    pub(crate) async fn fetch_datasources(&self) -> Result<Vec<super::Datasource>> {
        return Ok(vec![]);
        todo!()
    }

    pub(crate) async fn update_embedded_datasources(
        &mut self,
        _client: rest::TableauRestClient,
    ) -> Result<()> {
        // download the workbook
        // get the datasources
        // yikes...
        todo!()
    }

    fn cual_suffix(&self) -> String {
        format!("/workbook/{}", &self.id)
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

impl FetchPermissions for Workbook {
    fn get_endpoint(&self) -> String {
        format!("workbooks/{}/permissions", self.id)
    }
}

impl From<Workbook> for nodes::Asset {
    fn from(val: Workbook) -> Self {
        nodes::Asset::new(
            val.cual,
            val.name,
            AssetType::Other,
            // We will add metadata as it's useful.
            HashMap::new(),
            // Governing policies will be assigned in the policy.
            HashSet::new(),
            // Workbooks are children of their projects.
            HashSet::from([val.project_id]),
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

fn to_node_graphql(tc: &rest::TableauRestClient, val: &serde_json::Value) -> Result<Workbook> {
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
        cual: Cual::new(format!(
            "{}/workbook/{}",
            tc.get_cual_prefix(),
            workbook_info.luid
        )),
        id: workbook_info.luid,
        name: workbook_info.name,
        owner_id: workbook_info.owner.luid,
        project_id: workbook_info.project_luid,
        updated_at: workbook_info.updated_at,
        has_embedded_sources: workbook_info.embedded_datasources.len() > 0,
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
    super::to_asset_map(&tc, node, &to_node_graphql)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Ok, Result};

    #[tokio::test]
    async fn test_fetching_workbooks_works() -> Result<()> {
        let tc = crate::connector_setup()
            .await
            .context("running tableau connector setup")?;
        let groups = get_basic_workbooks(&tc.coordinator.rest_client).await?;
        for (_k, v) in groups {
            println!("{:#?}", v);
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
        println!("Downloaded {} bytes", x.len());
        Ok(())
    }

    #[tokio::test]
    async fn test_fetching_workbook_permissions_works() -> Result<()> {
        let tc = crate::connector_setup()
            .await
            .context("running tableau connector setup")?;
        let mut workbooks = get_basic_workbooks(&tc.coordinator.rest_client).await?;
        for (_k, v) in &mut workbooks {
            v.permissions = v.get_permissions(&tc.coordinator.rest_client).await?;
        }
        for (_k, v) in workbooks {
            println!("{:#?}", v);
        }
        Ok(())
    }

    #[test]
    fn test_asset_from_workbook_works() {
        let wb = Workbook::new(
            Cual::new("".to_owned()),
            "id".to_owned(),
            "name".to_owned(),
            "owner_id".to_owned(),
            "project_id".to_owned(),
            false,
            HashSet::new(),
            "updated_at".to_owned(),
            vec![],
        );
        nodes::Asset::from(wb);
    }

    #[test]
    fn test_workbook_into_asset_works() {
        let wb = Workbook::new(
            Cual::new("".to_owned()),
            "id".to_owned(),
            "name".to_owned(),
            "owner_id".to_owned(),
            "project_id".to_owned(),
            false,
            HashSet::new(),
            "updated_at".to_owned(),
            vec![],
        );
        let a: nodes::Asset = wb.into();
    }
}
