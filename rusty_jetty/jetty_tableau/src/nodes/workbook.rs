use std::collections::HashMap;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::rest::{self, FetchJson};

use super::FetchPermissions;

#[derive(Clone, Default, Debug, Deserialize)]
pub(crate) struct Workbook {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub project_id: String,
    pub embedded_sources: Vec<EmbeddedSource>,
    pub tableau_datasources: Vec<String>,
    pub updated_at: String,
    pub permissions: Vec<super::Permission>,
}

#[derive(Clone, Default, Debug, Deserialize)]
pub(crate) struct EmbeddedSource {
    pub name: String,
    pub derived_from: Option<Vec<String>>,
}

impl Workbook {
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
}

impl FetchPermissions for Workbook {
    fn get_endpoint(&self) -> String {
        format!("workbooks/{}/permissions", self.id)
    }
}

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
        id: workbook_info.luid,
        name: workbook_info.name,
        owner_id: workbook_info.owner.luid,
        project_id: workbook_info.project_luid,
        updated_at: workbook_info.updated_at,
        embedded_sources: workbook_info
            .embedded_datasources
            .into_iter()
            .map(|s| EmbeddedSource {
                name: s.name,
                derived_from: Default::default(),
            })
            .collect(),
        tableau_datasources: Default::default(),
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
    super::to_asset_map(node, &to_node_graphql)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};

    #[tokio::test]
    async fn test_fetching_workbooks_works() -> Result<()> {
        let tc = crate::connector_setup()
            .await
            .context("running tableau connector setup")?;
        let groups = get_basic_workbooks(&tc.env.rest_client).await?;
        for (_k, v) in groups {
            println!("{:#?}", v);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_fetching_workbook_permissions_works() -> Result<()> {
        let tc = crate::connector_setup()
            .await
            .context("running tableau connector setup")?;
        let mut workbooks = get_basic_workbooks(&tc.env.rest_client).await?;
        for (_k, v) in &mut workbooks {
            v.permissions = v.get_permissions(&tc.env.rest_client).await?;
        }
        for (_k, v) in workbooks {
            println!("{:#?}", v);
        }
        Ok(())
    }
}
