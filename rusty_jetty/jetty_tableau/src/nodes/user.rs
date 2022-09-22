use std::collections::HashMap;

use crate::rest::{self, FetchJson};
use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub external_auth_user_id: String,
    pub full_name: String,
    pub site_role: String,
}

pub(crate) fn to_node(val: &serde_json::Value) -> Result<User> {
    serde_json::from_value(val.to_owned()).context("parsing user information")
}

pub(crate) async fn get_basic_users(tc: &rest::TableauRestClient) -> Result<HashMap<String, User>> {
    let users = tc
        .build_request("users".to_owned(), None, reqwest::Method::GET)
        .context("fetching users")?
        .fetch_json_response(Some(vec!["users".to_owned(), "user".to_owned()]))
        .await?;
    super::to_asset_map(tc, users, &to_node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;

    #[tokio::test]
    async fn test_fetching_users_works() -> Result<()> {
        let tc = crate::connector_setup()
            .await
            .context("running tableau connector setup")?;
        let users = get_basic_users(&tc.coordinator.rest_client).await?;
        for (_k, v) in users {
            println!("{}", v.name);
        }
        Ok(())
    }
}
