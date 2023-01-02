use std::collections::{HashMap, HashSet};

use crate::rest::{self, FetchJson};
use anyhow::{Context, Result};
use jetty_core::connectors::{
    nodes::{self as jetty_nodes},
    UserIdentifier,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Copy, Default, Debug, Hash, PartialEq, Eq)]
pub(crate) enum SiteRole {
    Creator,
    Explorer,
    ExplorerCanPublish,
    ServerAdministrator,
    SiteAdministratorExplorer,
    SiteAdministratorCreator,
    Unlicensed,
    ReadOnly,
    Viewer,
    #[default]
    Unknown,
}

/// Representation of Tableau user
#[derive(Deserialize, Serialize, Clone, Default, Debug, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub external_auth_user_id: String,
    pub full_name: String,
    pub site_role: SiteRole,
}

impl From<User> for jetty_nodes::RawUser {
    fn from(val: User) -> Self {
        jetty_nodes::RawUser::new(
            val.id,
            HashSet::from([
                UserIdentifier::Email(val.email),
                UserIdentifier::FullName(val.full_name),
                UserIdentifier::Other(val.external_auth_user_id),
            ]),
            HashMap::new(),
            // Handled in groups.
            HashSet::new(),
            // Handled in permissions/policies.
            HashSet::new(),
        )
    }
}

/// Convert JSON into a User struct
pub(crate) fn to_node(val: &serde_json::Value) -> Result<User> {
    serde_json::from_value(val.to_owned()).context("parsing user information")
}

/// Fetch basic user information. This actually includes everything in the user struct!
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

    impl User {
        pub(crate) fn new(
            id: String,
            name: String,
            email: String,
            external_auth_user_id: String,
            full_name: String,
            site_role: SiteRole,
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

    #[test]
    #[allow(unused_must_use)]
    fn test_jetty_user_from_user_works() {
        let u = User::new(
            "id".to_owned(),
            "name".to_owned(),
            "email".to_owned(),
            "ea_user_id".to_owned(),
            "full_name".to_owned(),
            Default::default(),
        );
        jetty_nodes::RawUser::from(u);
    }

    #[test]
    #[allow(unused_must_use)]
    fn test_user_into_jetty_user_works() {
        let u = User::new(
            "id".to_owned(),
            "name".to_owned(),
            "email".to_owned(),
            "ea_user_id".to_owned(),
            "full_name".to_owned(),
            Default::default(),
        );
        Into::<jetty_nodes::RawUser>::into(u);
    }
}
