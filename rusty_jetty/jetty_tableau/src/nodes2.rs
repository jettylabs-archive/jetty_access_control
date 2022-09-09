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

use async_trait::async_trait;
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

use crate::rest::{self, FetchJson};

use anyhow::{bail, Result};
use reqwest::Method;
use serde::Deserialize;

/// This trait is implemented by permissionable Tableau asset nodes and makes it simpler to
/// fetch and parse permissions
#[async_trait]
trait FetchPermissions {
    fn get_endpoint(&self) -> String;

    /// Fetches the permissions for an asset and returns them as a vector of Permissions
    async fn get_permissions(&self, tc: &crate::TableauRestClient) -> Result<Vec<Permission>> {
        let resp = tc
            .build_request(self.get_endpoint(), None, reqwest::Method::GET)?
            .fetch_json_response(None)
            .await?;

        let permissions_array = rest::get_json_from_path(
            &resp,
            &vec!["permissions".to_owned(), "granteeCapabilities".to_owned()],
        )?;

        if let serde_json::Value::Array(_) = permissions_array {
            let permissions: Vec<SerializedPermission> = serde_json::from_value(permissions_array)?;
            Ok(permissions
                .iter()
                .map(move |p| p.to_owned().to_permission())
                .collect())
        } else {
            bail!("unable to parse permissions")
        }
    }
}

/// Trait that allows us to the id of a Tableau asset (used for generic id-based queries)
trait GetId {
    fn get_id(&self) -> String;
}

/// This Macro implements the GetId trait for one or more types that have an `id` field.
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

/// Helper struct for deserializing Tableau assets
#[derive(Deserialize, Debug, Clone)]
struct IdField {
    id: String,
}

/// Representation of Tableau permissions
#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Permission {
    grantee: Grantee,
    capabilities: HashMap<String, String>,
}

/// Grantee of a Tableau permission
#[derive(Deserialize, Debug, Clone)]
pub(crate) enum Grantee {
    Group { id: String },
    User { id: String },
}

/// Deserialization helper for Tableau permissions
#[derive(Deserialize, Debug, Clone)]
struct Capability {
    name: String,
    mode: String,
}

/// Deserialization helper for Tableau permissions
#[derive(Deserialize, Debug, Clone)]
struct Capabilities {
    capability: Vec<Capability>,
}

/// Deserialization helper for Tableau permissions
#[derive(Deserialize, Debug, Clone)]
struct SerializedPermission {
    group: Option<IdField>,
    user: Option<IdField>,
    capabilities: Capabilities,
}

impl SerializedPermission {
    /// Converts a Tableau permission response to a Permission struct to use
    /// when representing the Tableau environment
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

/// Converts a JSON Value::Array into the a vector of Tableau assets. Accepts a function to make
/// the JSON -> asset conversion
fn to_asset_map<T: GetId + Clone>(
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
