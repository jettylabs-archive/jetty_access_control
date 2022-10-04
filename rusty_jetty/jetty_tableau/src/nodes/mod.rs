//! This module defines the different types of nodes we'll need to internally
//! represent Tableau's structure as well as the functionality to turn that into
//! Jetty's node structure.

pub(crate) mod asset_to_policy;
pub(crate) mod datasource;
pub(crate) mod flow;
pub(crate) mod group;
pub(crate) mod lens;
pub(crate) mod metric;
pub(crate) mod project;
pub(crate) mod view;
pub(crate) mod workbook;

pub(crate) mod user;

use async_trait::async_trait;
pub(crate) use datasource::Datasource;
pub(crate) use flow::Flow;
pub(crate) use group::Group;
pub(crate) use lens::Lens;
pub(crate) use metric::Metric;
pub(crate) use project::Project;
pub(crate) use user::User;
pub(crate) use view::View;
pub(crate) use workbook::Workbook;

use std::collections::{HashMap, HashSet};

use crate::{
    coordinator::Environment,
    nodes as tableau_nodes,
    rest::{self, FetchJson, TableauAssetType},
    Cual, Cualable,
};

use jetty_core::connectors::nodes as jetty_nodes;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
/// A Tableau-created Project ID.
pub(crate) struct ProjectId(pub(crate) String);

/// This trait is implemented by permissionable Tableau asset nodes and makes it simpler to
/// fetch and parse permissions
#[async_trait]
pub(crate) trait Permissionable: core::fmt::Debug {
    fn get_endpoint(&self) -> String;

    fn get_permissions(&self) -> &Vec<Permission>;

    fn set_permissions(&mut self, permissions: Vec<Permission>);

    /// Fetches the permissions for an asset and returns them as a vector of Permissions
    async fn update_permissions(
        &mut self,
        tc: &crate::TableauRestClient,
        env: &Environment,
    ) -> Result<()> {
        let req = tc.build_request(self.get_endpoint(), None, reqwest::Method::GET)?;
        println!("{:?}", req);

        let resp = req.fetch_json_response(None).await?;

        let permissions_array = rest::get_json_from_path(
            &resp,
            &vec!["permissions".to_owned(), "granteeCapabilities".to_owned()],
        )?;

        let final_permissions = if matches!(permissions_array, serde_json::Value::Array(_)) {
            let permissions: Vec<SerializedPermission> = serde_json::from_value(permissions_array)?;
            permissions
                .iter()
                .map(move |p| {
                    p.to_owned()
                        .into_permission(env)
                        .expect("Couldn't understand Tableau permission response.")
                })
                .collect()
        } else {
            bail!("unable to parse permissions")
        };

        self.set_permissions(final_permissions);
        Ok(())
    }
}

/// Trait that allows us to the id of a Tableau asset (used for generic id-based queries)
trait GetId {
    fn get_id(&self) -> String;
}

pub(crate) trait TableauAsset {
    /// Get the asset type for this asset.
    fn get_asset_type(&self) -> TableauAssetType;
}

pub(crate) trait OwnedAsset: TableauAsset {
    /// Get the parent project ID.
    fn get_parent_project_id(&self) -> Option<&ProjectId>;
    /// Get the owner ID for this asset.
    fn get_owner_id(&self) -> &str;
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
    Workbook,
    View,
    Datasource,
    Metric,
    Flow,
    Lens
);

/// Project uses ProjectId for its ID field so it needs to have a bespoke impl.
impl GetId for Project {
    fn get_id(&self) -> String {
        self.id.0.to_owned()
    }
}

/// This Macro implements the GetId trait for one or more types that have an `id` field.
macro_rules! impl_OwnedAsset {
    (for $($t:ty),+) => {
        $(impl OwnedAsset for $t {
            fn get_parent_project_id(&self) -> Option<&ProjectId>{
                Some(&self.project_id)
            }

            fn get_owner_id(&self) -> &str {
                &self.owner_id
            }
        })*
    }
}

impl_OwnedAsset!(for
    Workbook,
    View,
    Datasource,
    Metric,
    Flow,
    Lens
);

/// Project is a little different so it needs to have a bespoke impl.
impl OwnedAsset for Project {
    fn get_parent_project_id(&self) -> Option<&ProjectId> {
        self.parent_project_id.as_ref()
    }

    fn get_owner_id(&self) -> &str {
        &self.owner_id
    }
}

/// This Macro implements the Cualable trait for one or more types that have a `cual` field.
macro_rules! impl_Cualable {
    (for $($t:ty),+) => {
        $(impl Cualable for $t {
            fn cual(&self) -> Cual{
                self.cual.clone()
            }
        })*
    }
}

impl_Cualable!(for
    Workbook,
    View,
    Datasource,
    Metric,
    Flow,
    Lens,
    Project
);

/// Helper struct for deserializing Tableau assets
#[derive(Deserialize, Debug, Clone)]
struct IdField {
    id: String,
}

/// Representation of Tableau permissions
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Permission {
    pub(crate) grantee: Grantee,
    pub(crate) capabilities: HashMap<String, String>,
}

impl Permission {
    pub(crate) fn has_capability(&self, cap: &str, mode: &str) -> bool {
        self.capabilities
            .iter()
            .find(|(c, m)| c.as_str() == cap && m.as_str() == mode)
            .is_some()
    }

    pub(crate) fn grantee_user_ids(&self) -> Vec<String> {
        match &self.grantee {
            Grantee::User(u) => vec![u.id.to_owned()],
            Grantee::Group(g) => g.includes.clone().iter().map(|u| u.id.to_owned()).collect(),
        }
    }

    pub(crate) fn grantee_user_emails(&self) -> Vec<String> {
        match &self.grantee {
            Grantee::User(u) => vec![u.email.to_owned()],
            Grantee::Group(g) => g
                .includes
                .clone()
                .iter()
                .map(|u| u.email.to_owned())
                .collect(),
        }
    }
}

/// Permissions and Jetty policies map 1:1.
impl From<Permission> for jetty_nodes::Policy {
    /// In order to get a Jetty policy from a permission, we need to grab
    /// the user or group it's been granted to.
    fn from(val: Permission) -> Self {
        let mut granted_to_groups = HashSet::new();
        let mut granted_to_users = HashSet::new();

        match val.grantee {
            Grantee::Group(tableau_nodes::Group { id, .. }) => granted_to_groups.insert(id),
            Grantee::User(tableau_nodes::User { id, .. }) => granted_to_users.insert(id),
        };

        jetty_nodes::Policy::new(
            // Leaving names empty for now for policies since they don't have
            // a lot of significance for policies here anyway.
            "".to_owned(),
            val.capabilities.into_values().collect(),
            // Handled by the caller.
            HashSet::new(),
            HashSet::new(),
            granted_to_groups,
            granted_to_users,
            false,
            false,
        )
    }
}

/// Grantee of a Tableau permission
#[derive(Deserialize, Debug, Clone, Serialize)]
pub(crate) enum Grantee {
    Group(tableau_nodes::Group),
    User(tableau_nodes::User),
}

impl Grantee {
    /// Get a human-readable name for this grantee.
    pub(crate) fn get_name(&self) -> &str {
        match self {
            Grantee::Group(g) => &g.name,
            Grantee::User(g) => &g.email,
        }
    }
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
    /// when representing the Tableau environment.
    pub(crate) fn into_permission(self, env: &Environment) -> Result<Permission> {
        // Get the grantee object from the environment. We assume the env
        // should already have it available.
        let grantee = match self {
            Self {
                group: Some(IdField { ref id }),
                ..
            } => Grantee::Group(
                env.groups
                    .get(id)
                    .unwrap_or_else(|| panic!("Group {} not yet in environment", id))
                    .clone(),
            ),
            Self {
                user: Some(IdField { ref id }),
                ..
            } => Grantee::User(
                env.users
                    .get(id)
                    .unwrap_or_else(|| panic!("User {} not yet in environment", id))
                    .clone(),
            ),
            _ => bail!("no user or group for permission {:#?}", self),
        };

        Ok(Permission {
            grantee,
            capabilities: self
                .capabilities
                .capability
                .iter()
                .map(|Capability { name, mode }| (name.to_owned(), mode.to_owned()))
                .collect(),
        })
    }
}

/// Converts a JSON Value::Array into the a vector of Tableau assets. Accepts a function to make
/// the JSON -> asset conversion
fn to_asset_map<T: GetId + Clone>(
    tc: &rest::TableauRestClient,
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
