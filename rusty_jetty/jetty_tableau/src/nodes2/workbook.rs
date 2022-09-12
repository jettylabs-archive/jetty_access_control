use std::collections::HashMap;

use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;

use crate::rest::{self, get_json_from_path, FetchJson, TableauRestClient};

#[derive(Clone, Default, Debug, Deserialize)]
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

impl Workbook {
    async fn get_permissions(&self, tc: &TableauRestClient) -> Result<Vec<super::Permission>> {
        let resp = tc
            .build_request(
                format!("workbooks/{}/permissions", self.id),
                None,
                reqwest::Method::GET,
            )?
            .fetch_json_response(None)
            .await?;

        let permissions_array = get_json_from_path(
            &resp,
            &vec!["permissions".to_owned(), "granteeCapabilities".to_owned()],
        )?;

        if let serde_json::Value::Array(_) = permissions_array {
            let permissions: Vec<super::SerializedPermission> =
                serde_json::from_value(permissions_array)?;
            Ok(permissions
                .iter()
                .map(move |p| p.to_owned().to_permission())
                .collect())
        } else {
            bail!("unable to parse permissions")
        }
    }
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

pub(crate) async fn get_basic_workbooks(
    tc: &rest::TableauRestClient,
) -> Result<HashMap<String, Workbook>> {
    let node = tc
        .build_request("workbooks".to_owned(), None, reqwest::Method::GET)
        .context("fetching groups")?
        .fetch_json_response(Some(vec!["workbooks".to_owned(), "workbook".to_owned()]))
        .await?;
    super::to_asset_map(node, &to_node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};

    use crate::nodes2::Permission;

    #[tokio::test]
    async fn test_fetching_workbooks_works() -> Result<()> {
        let tc = tokio::task::spawn_blocking(|| {
            crate::connector_setup().context("running tableau connector setup")
        })
        .await??;
        let groups = get_basic_workbooks(&tc.rest_client).await?;
        for (_k, v) in groups {
            println!("{:#?}", v);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_fetching_workbook_permissions_works() -> Result<()> {
        let tc = tokio::task::spawn_blocking(|| {
            crate::connector_setup().context("running tableau connector setup")
        })
        .await??;
        let mut workbooks = get_basic_workbooks(&tc.rest_client).await?;
        for (_k, v) in &mut workbooks {
            v.permissions = v.get_permissions(&tc.rest_client).await?;
        }
        for (_k, v) in workbooks {
            println!("{:#?}", v);
        }
        Ok(())
    }
}
