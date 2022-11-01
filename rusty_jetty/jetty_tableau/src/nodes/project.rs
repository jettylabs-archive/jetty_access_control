use std::collections::{HashMap, HashSet};

use super::{
    FromTableau, OwnedAsset, Permission, Permissionable, ProjectId, TableauAsset, PROJECT,
};
use crate::{
    coordinator::Environment,
    nodes::SerializedPermission,
    rest::{self, get_tableau_cual, FetchJson, TableauAssetType},
};

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use jetty_core::{
    connectors::{nodes as jetty_nodes, AssetType},
    logging::debug,
};
use serde::{Deserialize, Serialize};

/// Representation of a Tableau Project
#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub(crate) struct Project {
    pub id: ProjectId,
    pub name: String,
    pub owner_id: String,
    pub parent_project_id: Option<ProjectId>,
    pub controlling_permissions_project_id: Option<ProjectId>,
    pub permissions: Vec<Permission>,
}

impl Project {
    pub(crate) fn new(
        id: ProjectId,
        name: String,
        owner_id: String,
        parent_project_id: Option<ProjectId>,
        controlling_permissions_project_id: Option<ProjectId>,
        permissions: Vec<Permission>,
    ) -> Self {
        Self {
            id,
            name,
            owner_id,
            parent_project_id,
            controlling_permissions_project_id,
            permissions,
        }
    }

    /// Determine whether the given user is the project leader.
    pub(crate) fn is_leader(&self, user: &super::User) -> bool {
        self.permissions.iter().any(|p| {
            p.has_capability("ProjectLeader", "Allow") && p.grantee_user_ids().contains(&user.id)
        })
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
        updated_at: String,
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
        )?;

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
        .expect("Generating cual from flow");
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
