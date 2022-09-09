//! This module defines the different types of nodes we'll need to internally
//! represent Tableau's structure as well as the functionality to turn that into
//! Jetty's node structure.

mod data_connection;
mod datasource;
mod flow;
pub(crate) mod group;
mod lens;
mod metric;
mod project;
mod view;
mod workbook;

pub(crate) mod user;

pub(crate) use data_connection::DataConnection;
pub(crate) use datasource::Datasource;
pub(crate) use flow::Flow;
pub(crate) use group::Group;
pub(crate) use lens::Lens;
pub(crate) use metric::Metric;
pub(crate) use project::Project;
pub(crate) use user::User;
pub(crate) use view::View;
pub(crate) use workbook::Workbook;

use std::{collections::HashMap, fs::Permissions};

use crate::rest;

use anyhow::{bail, Result};
use reqwest::Method;
use serde::Deserialize;

pub(crate) trait GetId {
    fn get_id(&self) -> String;
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
    DataConnection,
    Metric,
    Flow,
    Lens
);

#[derive(Deserialize, Debug, Clone)]
struct IdField {
    id: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Permission {
    grantee: Grantee,
    capabilities: HashMap<String, String>,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Capability {
    name: String,
    mode: String,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) enum Grantee {
    Group { id: String },
    User { id: String },
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Capabilities {
    capability: Vec<Capability>,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct SerializedPermission {
    group: Option<IdField>,
    user: Option<IdField>,
    capabilities: Capabilities,
}

impl SerializedPermission {
    pub(crate) fn to_permission(self) -> Permission {
        let mut grantee_value = Grantee::Group { id: "".to_owned() };
        if let Some(id) = self.group {
            grantee_value = Grantee::Group { id: id.id }
        } else {
            grantee_value = Grantee::User {
                id: self.user.unwrap().id,
            }
        };

        Permission {
            grantee: grantee_value,
            capabilities: self
                .capabilities
                .capability
                .iter()
                .map(|c| (c.name.to_owned(), c.mode.to_owned()))
                .collect(),
        }
    }
}

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
