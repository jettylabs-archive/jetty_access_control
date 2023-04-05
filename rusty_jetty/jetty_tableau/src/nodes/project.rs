use std::collections::{HashMap, HashSet};

use super::{
    FromTableau, OwnedAsset, Permission, Permissionable, ProjectId, TableauAsset, PROJECT,
};
use crate::{
    coordinator::Environment,
    nodes::SerializedPermission,
    permissions::consts::{self},
    rest::{self, get_tableau_cual, FetchJson, TableauAssetType},
};

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use jetty_core::{
    connectors::{
        nodes::{self as jetty_nodes, RawDefaultPolicy, RawPolicyGrantee},
        AssetType,
    },
    logging::debug,
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

lazy_static! {
    // This determines the applicable types of the default policies that are fetched.
    // Some are commented out because we don't support the Tableau Catalog yet. Views are
    // covered by workbooks.
    static ref DEFAULT_POLICY_TYPE_CONVERSION: HashMap<String, String> = [
        ("workbooks", "workbook"),
        ("datasources", "datasource"),
        // ("dataroles", "datarole"),
        ("lenses", "lens"),
        ("flows", "flow"),
        ("metrics", "metric"),
        // ("databases", "database"),
        // ("tables", "table"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_owned(), v.to_owned()))
    .collect();
}

/// Representation of a Tableau Project
#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub(crate) struct Project {
    pub id: ProjectId,
    pub name: String,
    pub owner_id: String,
    pub parent_project_id: Option<ProjectId>,
    pub controlling_permissions_project_id: Option<ProjectId>,
    pub permissions: Vec<Permission>,
    pub content_permissions: ContentPermissions,
    /// Map of <Asset Type, Set<Capability>>
    pub default_permissions: HashMap<String, Vec<Permission>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) enum ContentPermissions {
    LockedToProject,
    LockedToProjectWithoutNested,
    ManagedByOwner,
}
impl Default for ContentPermissions {
    fn default() -> Self {
        ContentPermissions::ManagedByOwner
    }
}

impl ToString for ContentPermissions {
    fn to_string(&self) -> String {
        match self {
            ContentPermissions::LockedToProject => "LockedToProject".to_string(),
            ContentPermissions::LockedToProjectWithoutNested => {
                "LockedToProjectWithoutNested".to_string()
            }
            ContentPermissions::ManagedByOwner => "ManagedByOwner".to_string(),
        }
    }
}

impl Project {
    /// Get the default permissions for the project
    pub(crate) async fn update_default_permissions(
        &mut self,
        tc: &crate::TableauRestClient,
        env: &Environment,
    ) -> Result<()> {
        // first add the default project permissions (that's the easy one)
        self.default_permissions
            .insert("project".to_owned(), self.permissions.to_owned());

        for asset_type in DEFAULT_POLICY_TYPE_CONVERSION.keys() {
            let req = tc.build_request(
                format!("projects/{}/default-permissions/{asset_type}", self.id.0),
                None,
                reqwest::Method::GET,
            )?;

            let resp = req.fetch_json_response(None).await?;

            let permissions_array = rest::get_json_from_path(
                &resp,
                &vec!["permissions".to_owned(), "granteeCapabilities".to_owned()],
            );

            let permissions_array = match permissions_array {
                Ok(v) => v,
                // if there are no permissions, we can just skip this asset type for the project
                Err(_) => continue,
            };

            // default project, no parent project, user permission

            let final_permissions = if matches!(permissions_array, serde_json::Value::Array(_)) {
                let permissions: Vec<SerializedPermission> =
                    serde_json::from_value(permissions_array)?;
                permissions
                    .iter()
                    .map(|p| {
                        p.to_owned()
                            .into_permission(env)
                            .expect("Couldn't understand Tableau permission response.")
                    })
                    .collect::<Vec<_>>()
            } else {
                bail!("unable to parse permissions")
            };
            self.default_permissions.insert(
                DEFAULT_POLICY_TYPE_CONVERSION[asset_type].to_owned(),
                final_permissions.to_owned(),
            );

            // View permissions are the same as workbook permissions, so add them here
            if asset_type == "workbooks" {
                self.default_permissions.insert(
                    "view".to_owned(),
                    final_permissions
                        .iter()
                        .map(convert_workbook_permission_to_view_permissions)
                        .collect(),
                );
            }
        }

        Ok(())
    }

    /// Take a Project and generate default policies from it
    pub(crate) fn get_default_policies(
        &self,
        env: &Environment,
    ) -> Vec<jetty_nodes::RawDefaultPolicy> {
        let mut res = Vec::new();
        let root_cual = get_tableau_cual(
            TableauAssetType::Project,
            &self.name,
            self.parent_project_id.as_ref(),
            None,
            env,
        )
        .expect("Generating cual from project");

        for (asset_type, permissions) in &self.default_permissions {
            let target_asset_type = AssetType(asset_type.to_owned());

            for permission in permissions {
                // get the raw policy
                let raw: jetty_nodes::RawPolicy = permission.to_owned().into();
                let mut grantees: HashSet<_> = raw
                    .granted_to_users
                    .into_iter()
                    .map(RawPolicyGrantee::User)
                    .collect();
                grantees.extend(
                    raw.granted_to_groups
                        .into_iter()
                        .map(RawPolicyGrantee::Group),
                );
                for grantee in grantees {
                    res.push(RawDefaultPolicy {
                        privileges: raw.privileges.to_owned(),
                        root_asset: root_cual.to_owned(),
                        wildcard_path: "/**".to_owned(),
                        target_type: target_asset_type.to_owned(),
                        grantee,
                        /// Content permissions are controlled only at the project level
                        metadata: if asset_type == "project" {
                            HashMap::from([(
                                "Tableau Content Permissions".to_owned(),
                                self.content_permissions.to_string(),
                            )])
                        } else {
                            HashMap::new()
                        },
                    });
                }
            }
        }
        res
    }
}

/// Convert JSON into a project struct
fn to_node(val: &serde_json::Value) -> Result<super::Project> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct ProjectInfo {
        name: String,
        id: String,
        owner: super::IdField,
        parent_project_id: Option<String>,
        controlling_permissions_project_id: Option<String>,
        #[serde(rename = "updatedAt")]
        _updated_at: String,
        content_permissions: ContentPermissions,
    }

    let project_info: ProjectInfo =
        serde_json::from_value(val.to_owned()).context("parsing asset information")?;

    Ok(super::Project {
        id: ProjectId(project_info.id),
        name: project_info.name,
        owner_id: project_info.owner.id,
        parent_project_id: project_info.parent_project_id.map(ProjectId),
        controlling_permissions_project_id: project_info
            .controlling_permissions_project_id
            .map(ProjectId),
        permissions: Default::default(),
        default_permissions: Default::default(),
        content_permissions: project_info.content_permissions,
    })
}

/// Get basic project information (excluding permissions)
pub(crate) async fn get_basic_projects(
    tc: &rest::TableauRestClient,
) -> Result<HashMap<String, Project>> {
    let node = tc
        .build_request("projects".to_owned(), None, reqwest::Method::GET)
        .context("fetching projects")?
        .fetch_json_response(Some(vec!["projects".to_owned(), "project".to_owned()]))
        .await?;
    super::to_asset_map(tc, node, &to_node)
}

#[async_trait]
impl Permissionable for Project {
    fn get_endpoint(&self) -> String {
        let ProjectId(id) = &self.id;
        format!("projects/{id}/permissions")
    }
    fn set_permissions(&mut self, permissions: Vec<super::Permission>) {
        self.permissions = permissions;
    }

    fn get_permissions(&self) -> &Vec<super::Permission> {
        &self.permissions
    }

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

        // default project, no parent project, user permission
        let final_permissions = if matches!(permissions_array, serde_json::Value::Array(_)) {
            let permissions: Vec<SerializedPermission> = serde_json::from_value(permissions_array)?;
            permissions
                .iter()
                .filter_map(|p| {
                    let permission_result = p.to_owned().into_permission(env);
                    if permission_result.is_err()
                        && &self.name == "default"
                        && matches!(p, SerializedPermission { user: Some(_), .. })
                        && self.parent_project_id.is_none()
                    {
                        // We infer this to be the default project owner
                        // permission (name == "default", no parent project
                        // ID, user owner), for which a user does not exist
                        // (permission_result is an err type). Therefore, we
                        // will skip this permission.
                        debug!("Skipping owner {:?} for default project.", p);
                        None
                    } else {
                        Some(
                            permission_result
                                .expect("Couldn't understand Tableau permission response."),
                        )
                    }
                })
                .collect()
        } else {
            bail!("unable to parse permissions")
        };

        self.set_permissions(final_permissions);
        Ok(())
    }
}

impl FromTableau<Project> for jetty_nodes::RawAsset {
    fn from(val: Project, env: &Environment) -> Self {
        let cual = get_tableau_cual(
            TableauAssetType::Project,
            &val.name,
            val.parent_project_id.as_ref(),
            None,
            env,
        )
        .expect("Generating cual from project");
        let parent_cuals = val
            .get_parent_project_cual(env)
            .map_or_else(HashSet::new, |c| HashSet::from([c.uri()]));
        jetty_nodes::RawAsset::new(
            cual,
            val.name,
            AssetType(PROJECT.to_owned()),
            // We will add metadata as it's useful.
            HashMap::new(),
            // Governing policies will be assigned in the policy.
            HashSet::new(),
            // Projects can be the children of other projects.
            parent_cuals,
            // Children objects will be handled in their respective nodes.
            HashSet::new(),
            // Projects aren't derived from/to anything.
            HashSet::new(),
            HashSet::new(),
            // No tags at this point.
            HashSet::new(),
        )
    }
}

impl TableauAsset for Project {
    fn get_asset_type(&self) -> TableauAssetType {
        TableauAssetType::Project
    }
}

fn convert_workbook_permission_to_view_permissions(permission: &Permission) -> Permission {
    let mut new_capabilities = HashMap::new();
    for (capability, mode) in &permission.capabilities {
        if consts::VIEW_CAPABILITIES.contains(&capability.as_str()) {
            new_capabilities.insert(capability.to_owned(), mode.to_owned());
        }
    }
    Permission {
        grantee: permission.grantee.to_owned(),
        capabilities: new_capabilities,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    impl Project {
        #[allow(clippy::too_many_arguments)]
        pub(crate) fn new(
            id: ProjectId,
            name: String,
            owner_id: String,
            parent_project_id: Option<ProjectId>,
            controlling_permissions_project_id: Option<ProjectId>,
            permissions: Vec<Permission>,
            default_permissions: HashMap<String, Vec<Permission>>,
            content_permissions: ContentPermissions,
        ) -> Self {
            Self {
                id,
                name,
                owner_id,
                parent_project_id,
                controlling_permissions_project_id,
                permissions,
                content_permissions,
                default_permissions,
            }
        }
    }
}
