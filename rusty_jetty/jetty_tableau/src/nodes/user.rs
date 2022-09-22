use std::collections::{HashMap, HashSet};

use crate::rest::{self, FetchJson};
use anyhow::{Context, Result};
use jetty_core::connectors::{nodes, UserIdentifier};
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

impl User {
    pub(crate) fn new(
        id: String,
        name: String,
        email: String,
        external_auth_user_id: String,
        full_name: String,
        site_role: String,
    ) -> Self {
        Self {
            id,
            name,
            email,
            external_auth_user_id,
            full_name,
            site_role,
        }
    }
}

impl From<User> for nodes::User {
    fn from(val: User) -> Self {
        nodes::User::new(
            val.name,
            HashMap::from([
                (UserIdentifier::Email, val.email),
                (UserIdentifier::FullName, val.full_name),
            ]),
            HashSet::from([val.external_auth_user_id, val.site_role]),
            HashMap::new(),
            // Handled in groups.
            HashSet::new(),
            // Handled in permissions/policies.
            HashSet::new(),
        )
    }
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

    #[test]
    fn test_jetty_user_from_user_works() {
        let u = User::new(
            "id".to_owned(),
            "name".to_owned(),
            "email".to_owned(),
            "ea_user_id".to_owned(),
            "full_name".to_owned(),
            "site_role".to_owned(),
        );
        nodes::User::from(u);
    }

    #[test]
    fn test_user_into_jetty_user_works() {
        let u = User::new(
            "id".to_owned(),
            "name".to_owned(),
            "email".to_owned(),
            "ea_user_id".to_owned(),
            "full_name".to_owned(),
            "site_role".to_owned(),
        );
        let a: nodes::User = u.into();
    }
}
