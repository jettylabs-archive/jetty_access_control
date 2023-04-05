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
    rest::{self, get_tableau_cual, FetchJson, TableauAssetType},
    Cual,
};

use jetty_core::connectors::nodes as jetty_nodes;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const DATASOURCE: &str = "datasource";
const WORKBOOK: &str = "workbook";
const PROJECT: &str = "project";
const FLOW: &str = "flow";
const METRIC: &str = "metric";
const LENS: &str = "lens";
const VIEW: &str = "view";

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
/// A Tableau-created Project ID.
pub(crate) struct ProjectId(pub(crate) String);

/// Conversion from Tableau types.
pub(crate) trait FromTableau<T> {
    fn from(val: T, env: &Environment) -> Self;
}

pub(crate) trait IntoTableau<U: FromTableau<Self>>
where
    Self: Sized,
{
    fn into(self, env: &Environment) -> U;
}

impl<T, U> IntoTableau<U> for T
where
    U: FromTableau<T>,
{
    fn into(self, env: &Environment) -> U {
        <U>::from(self, env)
    }
}

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

        let resp = req.fetch_json_response(None).await?;

        let permissions_array = rest::get_json_from_path(
            &resp,
            &vec!["permissions".to_owned(), "granteeCapabilities".to_owned()],
        );

        let permissions_array = match permissions_array {
            Ok(v) => v,
            // if there are no permissions, we can just skip this part
            Err(_) => return Ok(()),
        };

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
    /// Get the cual for the asset's parent project if one exists.
    fn get_parent_project_cual(&self, env: &Environment) -> Option<Cual> {
        self.get_parent_project_id().and_then(|ppid| {
            let ProjectId(pid) = ppid;
            let project = env
                .projects
                .get(pid)
                .expect("getting flow parent project by id");
            get_tableau_cual(
                TableauAssetType::Project,
                &project.name,
                project.parent_project_id.as_ref(),
                None,
                env,
            )
            .ok()
        })
    }
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

/// This Macro implements the OwnedAsset. Provides utilities to crawl tree of project and content owners.
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

/// Common behavior across tableau assets
pub(crate) trait TableauCualable {
    /// Get the cual for the associated asset object.
    fn cual(&self, env: &Environment) -> Cual;
}

/// This Macro implements the Cualable trait for one or more types that have a `cual` field.
macro_rules! impl_Cualable {
    (for $($t:tt),+) => {
        $(impl TableauCualable for $t {
            fn cual(&self, env: &Environment) -> Cual{
                    get_tableau_cual(
                        TableauAssetType::$t,
                        &self.name,
                        self.get_parent_project_id(),
                        None,
                        env,
                    )
                    .expect(&format!("making cual for tableau asset {:?}", TableauAssetType::$t))
            }
        })*
    }
}

impl_Cualable!(for
    Workbook,
    Datasource,
    Flow,
    Project
);

impl TableauCualable for View {
    fn cual(&self, env: &Environment) -> Cual {
        get_tableau_cual(
            TableauAssetType::View,
            &self.name,
            self.get_parent_project_id(),
            Some(&self.workbook_id),
            env,
        )
        .expect("making cual for view")
    }
}

impl TableauCualable for Metric {
    fn cual(&self, env: &Environment) -> Cual {
        get_tableau_cual(
            TableauAssetType::Metric,
            &self.name,
            self.get_parent_project_id(),
            Some(&self.underlying_view_id),
            env,
        )
        .expect("making cual for metric")
    }
}

impl TableauCualable for Lens {
    fn cual(&self, env: &Environment) -> Cual {
        get_tableau_cual(
            TableauAssetType::Lens,
            &self.name,
            self.get_parent_project_id(),
            Some(&self.datasource_id),
            env,
        )
        .expect("making cual for lens")
    }
}

/// Helper struct for deserializing Tableau assets
#[derive(Deserialize, Debug, Clone, Default)]
struct IdField {
    id: String,
}

pub(crate) struct IndividualPermission {
    pub(crate) capability: String,
    pub(crate) mode: TableauPermissionMode,
}

#[derive(Debug)]
pub(crate) enum TableauPermissionMode {
    Allow,
    Deny,
    Other,
}

impl ToString for TableauPermissionMode {
    fn to_string(&self) -> String {
        match self {
            TableauPermissionMode::Allow => "Allow".to_owned(),
            TableauPermissionMode::Deny => "Deny".to_owned(),
            TableauPermissionMode::Other => "Other".to_owned(),
        }
    }
}

impl IndividualPermission {
    pub(crate) fn from_string(val: &String) -> Self {
        let mode = if val.starts_with("Allow") {
            TableauPermissionMode::Allow
        } else if val.starts_with("Deny") {
            TableauPermissionMode::Deny
        } else {
            TableauPermissionMode::Other
        };

        let capability = val.strip_prefix("Deny").unwrap_or(val);
        let capability = capability
            .strip_prefix("Allow")
            .unwrap_or(capability)
            .to_owned();
        IndividualPermission { capability, mode }
    }
}

/// Representation of Tableau permissions
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Permission {
    pub(crate) grantee: Grantee,
    pub(crate) capabilities: HashMap<String, String>,
}

/// Permissions and Jetty policies map 1:1.
impl From<Permission> for jetty_nodes::RawPolicy {
    /// In order to get a Jetty policy from a permission, we need to grab
    /// the user or group it's been granted to.
    fn from(val: Permission) -> Self {
        let mut granted_to_groups = HashSet::new();
        let mut granted_to_users = HashSet::new();

        match val.grantee {
            Grantee::Group(tableau_nodes::Group { name, .. }) => granted_to_groups.insert(name),
            Grantee::User(tableau_nodes::User { id, .. }) => granted_to_users.insert(id),
        };

        jetty_nodes::RawPolicy::new(
            // Leaving names empty for now for policies since they don't have
            // a lot of significance for policies here anyway.
            Uuid::new_v4().to_string(),
            val.capabilities
                .into_iter()
                .filter_map(|(capability, mode)| {
                    if &capability == "InheritedProjectLeader" {
                        None
                    } else {
                        Some(format!("{mode}{capability}"))
                    }
                })
                .collect(),
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
#[derive(Deserialize, Debug, Clone, Serialize, Hash)]
pub(crate) enum Grantee {
    Group(tableau_nodes::Group),
    User(tableau_nodes::User),
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
                    .unwrap_or_else(|| panic!("Group {id} not yet in environment"))
                    .clone(),
            ),
            Self {
                user: Some(IdField { ref id }),
                ..
            } => Grantee::User(
                env.users
                    .get(id)
                    .unwrap_or_else(|| panic!("User {id} not yet in environment"))
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
    _tc: &rest::TableauRestClient,
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
