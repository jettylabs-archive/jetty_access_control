use anyhow::{bail, Context, Result};
use jetty_core::connectors::{nodes as jetty_nodes, UserIdentifier};
use serde::Deserialize;
use serde_json;
use std::collections::{HashMap, HashSet};

pub trait CreateNode {
    fn to_users(&self) -> Result<Vec<jetty_nodes::User>>;
}

impl CreateNode for serde_json::Value {
    fn to_users(&self) -> Result<Vec<jetty_nodes::User>> {
        if let serde_json::Value::Array(users) = &self {
            users
                .iter()
                .map(|u| to_user(u))
                .collect::<Result<Vec<jetty_nodes::User>>>()
        } else {
            bail!["not a JSON array of user data: {:#?}", self]
        }
    }
}

fn to_user(val: &serde_json::Value) -> Result<jetty_nodes::User> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct UserInfo {
        email: String,
        external_auth_user_id: String,
        full_name: String,
        name: String,
        id: String,
        site_role: String,
    }

    let user_info: UserInfo =
        serde_json::from_value(val.to_owned()).context("parsing user information")?;

    let identifiers = HashMap::from([
        (UserIdentifier::Email, user_info.email),
        (UserIdentifier::FullName, user_info.full_name),
    ]);
    let other_identifiers = HashSet::from([
        user_info.external_auth_user_id,
        user_info.name.to_owned(),
        user_info.id,
    ]);
    let metadata = HashMap::from([("site_role".to_owned(), user_info.site_role)]);

    Ok(jetty_nodes::User {
        name: user_info.name,
        identifiers,
        other_identifiers,
        metadata,
        ..Default::default()
    })
}
