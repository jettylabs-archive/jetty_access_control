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

use std::collections::HashMap;

use anyhow::{bail, Result};
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

#[derive(Deserialize)]
struct IdField {
    id: String,
}

#[derive(Clone, Debug)]
pub(crate) struct Permission {
    grantee_user_id: Option<String>,
    grantee_group_id: Option<String>,
    capabilities: HashMap<String, String>,
}

pub(crate) fn to_asset_map<T: GetId + Clone>(
    val: serde_json::Value,
    f: &dyn Fn(&serde_json::Value) -> Result<T>,
) -> Result<HashMap<String, T>> {
    let node_map: HashMap<String, T>;
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
