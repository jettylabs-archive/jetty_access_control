use std::collections::HashMap;

use crate::rest::{self, FetchJson};
use anyhow::{Context, Result};
use futures::StreamExt;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub(crate) struct Group {
    pub id: String,
    pub name: String,
    /// Vec of user uids
    pub includes: Vec<String>,
}

impl Group {
    pub(crate) async fn get_users(&mut self, tc: &rest::TableauRestClient) -> Result<()> {
        let resp = tc
            .build_request(
                format!("groups/{}/users", self.id),
                None,
                reqwest::Method::GET,
            )
            .context("fetching group membership")?
            .fetch_json_response(Some(vec!["users".to_owned(), "user".to_owned()]))
            .await
            .context(format!("getting membership for group {}", self.name))?;

        let user_vec: Vec<super::IdField> =
            serde_json::from_value(resp).context("parsing group membership")?;
        self.includes = user_vec.iter().map(|u| u.id.to_owned()).collect();
        Ok(())
    }
}

pub(crate) fn to_node(val: &serde_json::Value) -> Result<Group> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct GroupInfo {
        name: String,
        id: String,
    }
    let group_info: GroupInfo =
        serde_json::from_value(val.to_owned()).context("parsing group information")?;

    Ok(Group {
        id: group_info.id,
        name: group_info.name,
        includes: Vec::new(),
    })
}

pub(crate) async fn get_basic_groups(
    tc: &rest::TableauRestClient,
) -> Result<HashMap<String, Group>> {
    let node = tc
        .build_request("groups".to_owned(), None, reqwest::Method::GET)
        .context("fetching groups")?
        .fetch_json_response(Some(vec!["groups".to_owned(), "group".to_owned()]))
        .await?;
    super::to_asset_map(node, &to_node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};

    #[tokio::test]
    async fn test_fetching_groups_works() -> Result<()> {
        let tc = crate::connector_setup()
            .await
            .context("running tableau connector setup")?;
        let groups = get_basic_groups(&tc.coordinator.rest_client).await?;
        for (_k, v) in groups {
            println!("{:#?}", v);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_fetching_groups_with_users_works() -> Result<()> {
        let tc = crate::connector_setup()
            .await
            .context("running tableau connector setup")?;
        let mut groups = get_basic_groups(&tc.coordinator.rest_client).await?;
        for (_k, v) in &mut groups {
            v.get_users(&tc.coordinator.rest_client);
            println!("{:#?}", v);
        }
        Ok(())
    }
}
