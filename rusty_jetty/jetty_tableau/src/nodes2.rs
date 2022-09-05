//! This module defines the different types of nodes we'll need to internally
//! represent Tableau's structure as well as the functionality to turn that into
//! Jetty's node structure.

mod groups;
mod projects;
pub(crate) mod users;

use std::collections::HashMap;

use anyhow::{bail, Result};
use serde::Deserialize;

pub(crate) trait GetId {
    fn get_id(&self) -> String;
}

#[derive(Clone)]
struct Permission {
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
struct Project {
    id: String,
    name: String,
    owner_id: String,
    parent_project_id: Option<String>,
    controlling_permissions_project_id: Option<String>,
    permissions: Vec<Permission>,
}

#[derive(Clone)]
struct Workbook {
    id: String,
    name: String,
    owner_id: String,
    project_id: String,
    datasource_connections: String,
    datasources: Vec<String>,
    updated_at: String,
    permissions: Vec<Permission>,
}

#[derive(Clone)]
struct View {
    id: String,
    name: String,
    workbook_id: String,
    owner_id: String,
    permissions: Vec<Permission>,
}

#[derive(Clone)]
struct Datasource {
    id: String,
    name: String,
    datasource_type: String,
    updated_at: String,
    project_id: String,
    owner_id: String,
    datasource_connections: Vec<String>,
    permissions: Vec<Permission>,
}

#[derive(Clone)]
struct DataConnector {
    id: String,
    connector_type: String,
    user_name: Option<String>,
    derived_from: Vec<String>,
}

#[derive(Clone)]
struct Metric {
    id: String,
    name: String,
    updated_at: String,
    suspended: bool,
    project_id: String,
    owner_id: String,
    underlying_view_id: String,
    permissions: Vec<Permission>, // Not yet sure if this will be possible
}

#[derive(Clone)]
struct Flow {
    id: String,
    name: String,
    project_id: String,
    owner_id: String,
    updated_at: String,
    datasource_connections: Vec<String>,
    permissions: Vec<Permission>,
}

#[derive(Clone)]
struct Lens {
    id: String,
    name: String,
    datasource_id: String,
    project_id: String,
    owner_id: String,
    permissions: Vec<Permission>,
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
