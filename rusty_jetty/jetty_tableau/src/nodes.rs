use anyhow::{bail, Context, Result};
use jetty_core::connectors::{nodes as jetty_nodes, UserIdentifier};
use serde::Deserialize;
use serde_json;
use std::collections::{HashMap, HashSet};

pub trait CreateNode {
    fn to_users(&self) -> Result<Vec<jetty_nodes::User>>;
    fn to_groups(&self) -> Result<Vec<jetty_nodes::Group>>;
}

#[derive(Deserialize)]
struct Domain {
    name: String,
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

    fn to_groups(&self) -> Result<Vec<jetty_nodes::Group>> {
        if let serde_json::Value::Array(groups) = &self {
            groups
                .iter()
                .map(|u| to_group(u))
                .collect::<Result<Vec<jetty_nodes::Group>>>()
        } else {
            bail!["not a JSON array of group data: {:#?}", self]
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
        domain: Domain,
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
        user_info.id.to_owned(),
    ]);
    let metadata = HashMap::from([
        ("site_role".to_owned(), user_info.site_role),
        ("user_id".to_owned(), user_info.id),
        ("domain".to_owned(), user_info.domain.name),
    ]);

    Ok(jetty_nodes::User {
        name: user_info.name,
        identifiers,
        other_identifiers,
        metadata,
        ..Default::default()
    })
}

fn to_group(val: &serde_json::Value) -> Result<jetty_nodes::Group> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct GroupInfo {
        name: String,
        id: String,
        domain: Domain,
        import: Option<GroupImportInfo>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct GroupImportInfo {
        source: String,
        domain_name: String,
        site_role: String,
        grant_license_mode: String,
    }

    let group_info: GroupInfo =
        serde_json::from_value(val.to_owned()).context("parsing group information")?;

    let mut metadata = HashMap::from([
        ("domain".to_owned(), group_info.domain.name),
        ("group_id".to_owned(), group_info.id),
    ]);

    if let Some(import) = group_info.import {
        let import_info = HashMap::from([
            ("import_source".to_owned(), import.source),
            ("import_domain_name".to_owned(), import.domain_name),
            ("import_site_role".to_owned(), import.site_role),
            (
                "import_grant_license_mode".to_owned(),
                import.grant_license_mode,
            ),
        ]);
        metadata.extend(import_info);
    };
    Ok(jetty_nodes::Group {
        name: group_info.name,
        metadata,
        ..Default::default()
    })
}
