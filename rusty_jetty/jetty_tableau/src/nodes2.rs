//! This module defines the different types of nodes we'll need to internally
//! represent Tableau's structure as well as the functionality to turn that into
//! Jetty's node structure.

mod groups;
mod lenses;
mod projects;
mod views;
mod workbooks;

pub(crate) mod users;

use std::collections::HashMap;

use anyhow::{bail, Result};
use serde::Deserialize;

pub(crate) trait GetId {
    fn get_id(&self) -> String;
}

#[derive(Deserialize)]
struct IdField {
    id: String,
}

#[derive(Clone)]
pub(crate) struct Permission {
    grantee_user_id: Option<String>,
    grantee_group_id: Option<String>,
    capabilities: HashMap<String, String>,
}

#[derive(Clone)]
pub(crate) struct Group {
    id: String,
    name: String,
    includes: Vec<String>,
}

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

#[derive(Clone)]
pub(crate) struct Project {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub parent_project_id: Option<String>,
    pub controlling_permissions_project_id: Option<String>,
    pub permissions: Vec<Permission>,
}

#[derive(Clone, Default)]
pub(crate) struct Workbook {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub project_id: String,
    pub datasource_connections: String,
    pub datasources: Vec<String>,
    pub updated_at: String,
    pub permissions: Vec<Permission>,
}

#[derive(Clone, Default)]
pub(crate) struct View {
    pub id: String,
    pub name: String,
    pub workbook_id: String,
    pub owner_id: String,
    pub project_id: String,
    pub updated_at: String,
    pub permissions: Vec<Permission>,
}

#[derive(Clone)]
pub(crate) struct Datasource {
    pub id: String,
    pub name: String,
    pub datasource_type: String,
    pub updated_at: String,
    pub project_id: String,
    pub owner_id: String,
    pub datasource_connections: Vec<String>,
    pub permissions: Vec<Permission>,
}

#[derive(Clone)]
pub(crate) struct DataConnector {
    pub id: String,
    pub connector_type: String,
    pub user_name: Option<String>,
    pub derived_from: Vec<String>,
}

#[derive(Clone)]
pub(crate) struct Metric {
    pub id: String,
    pub name: String,
    pub updated_at: String,
    pub suspended: bool,
    pub project_id: String,
    pub owner_id: String,
    pub underlying_view_id: String,
    pub permissions: Vec<Permission>, // Not yet sure if this will be possible
}

#[derive(Clone)]
pub(crate) struct Flow {
    pub id: String,
    pub name: String,
    pub project_id: String,
    pub owner_id: String,
    pub updated_at: String,
    pub datasource_connections: Vec<String>,
    pub permissions: Vec<Permission>,
}

#[derive(Clone)]
pub(crate) struct Lens {
    pub id: String,
    pub name: String,
    pub datasource_id: String,
    pub project_id: String,
    pub owner_id: String,
    pub permissions: Vec<Permission>,
}

/// This Macro implements the GetId trait for one or more types.
macro_rules! impl_GetId {
    (for $($t:ty),+) => {
        $(impl GetId for $t {
            fn get_id(&self) -> String {
                self.id.to_owned()
            }
        })*
    }
}

impl_GetId!(for
    Group,
    User,
    Project,
    Workbook,
    View,
    Datasource,
    DataConnector,
    Metric,
    Flow,
    Lens
);

pub(crate) fn to_asset_map<T: GetId + Clone>(
    val: serde_json::Value,
    f: &dyn Fn(&serde_json::Value) -> Result<T>,
) -> Result<HashMap<String, T>> {
    let mut node_map = HashMap::new();
    if let serde_json::Value::Array(assets) = val {
        let node_vec = assets.iter().map(|a| f(a)).collect::<Result<Vec<T>>>()?;

        node_map = node_vec
            .iter()
            .map(|n| (n.get_id(), n.to_owned()))
            .collect();
        Ok(node_map)
    } else {
        bail!["incorrect data structure: {:#?}", val]
    }
}
