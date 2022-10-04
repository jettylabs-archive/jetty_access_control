use std::collections::{HashMap, HashSet};

use jetty_core::{
    connectors::{
        nodes::{EffectivePermission, PermissionMode, SparseMatrix},
        UserIdentifier,
    },
    cual::{Cual, Cualable},
};

use crate::{
    coordinator::Coordinator,
    nodes::{self, user::SiteRole, OwnedAsset, ProjectId},
    nodes::{Grantee, Permissionable},
};

pub(crate) struct PermissionManager<'x> {
    coordinator: &'x Coordinator,
}

impl<'x> PermissionManager<'x> {
    pub(crate) fn new(coordinator: &'x Coordinator) -> Self {
        Self { coordinator }
    }

    /// Given an asset, get all of the user -> permission pairings.
    ///
    /// Return a map of users to a list of their (origin, capability, mode)
    /// permissions.
    ///
    /// origin indicates whether the permission comes from the user or a group.
    fn get_user_perms<'a, T: Permissionable>(
        &self,
        asset: &'a T,
    ) -> HashMap<&'a nodes::User, Vec<(&'a Grantee /* origin*/, &'a String, &'a String)>> {
        let mut user_perm_map: HashMap<&nodes::User, Vec<(&Grantee, &String, &String)>> =
            HashMap::new();
        asset.get_permissions().iter().for_each(|perm| {
            perm.capabilities.iter().for_each(|p| {
                match &perm.grantee {
                    Grantee::User(u) => {
                        if let Some(perms) = user_perm_map.get_mut(&u) {
                            (*perms).push((&perm.grantee, p.0, p.1));
                        } else {
                            user_perm_map.insert(&u, vec![(&perm.grantee, p.0, p.1)]);
                        }
                    }
                    Grantee::Group(g) => {
                        // insert permission by [user][asset] into map for all users in group.
                        for user in &g.includes {
                            if let Some(perms) = user_perm_map.get_mut(&user) {
                                (*perms).push((&perm.grantee, p.0, p.1));
                            } else {
                                user_perm_map.insert(&user, vec![(&perm.grantee, p.0, p.1)]);
                            }
                        }
                    }
                }
            });
        });
        user_perm_map
    }

    pub(crate) fn get_effective_permissions_for_asset<T: OwnedAsset + Permissionable + Cualable>(
        &self,
        assets: &HashMap<String, T>,
    ) -> SparseMatrix<UserIdentifier, Cual, HashSet<EffectivePermission>> {
        let mut ep: SparseMatrix<UserIdentifier, Cual, HashSet<EffectivePermission>> =
            HashMap::new();
        // for each asset
        assets.values().for_each(|asset| {
            // get all perms as user -> [permission] mapping
            let user_perm_map = self.get_user_perms(asset);
            // We'll go over each of those user -> [permission] mappings to
            // discover effective access.
            user_perm_map.iter().map(|(user, perms)| {
                // 1. check site role for allow alls and missing licenses.
                let allow_all = match user.site_role {
                    SiteRole::Creator
                    | SiteRole::Explorer
                    | SiteRole::ExplorerCanPublish
                    | SiteRole::ReadOnly
                    | SiteRole::SiteAdministratorExplorer
                    | SiteRole::Viewer => Some(false),
                    SiteRole::ServerAdministrator | SiteRole::SiteAdministratorCreator => {
                        Some(true)
                    }
                    SiteRole::Unlicensed | SiteRole::Unknown => None,
                };

                let final_permissions: HashSet<EffectivePermission> = if let Some(allow) = allow_all
                {
                    if allow {
                        // Site role guarantees access, grant all capabilities.
                        perms
                            .iter()
                            .map(|(_, capa, mode)| {
                                EffectivePermission::new(
                                    capa.to_string(),
                                    PermissionMode::Allow,
                                    vec![format!("user has site role {:?}", user.site_role)],
                                )
                            })
                            .collect()
                    } else {
                        // Need to dig more to figure out effective permission.
                        // TODO: climb the project hierarchy here to check all parent projects
                        let ProjectId(project_id) = asset.get_parent_project_id().unwrap();
                        let parent_project = &self.coordinator.env.projects[&project_id.to_owned()];
                        let is_project_leader = parent_project
                            .permissions
                            .iter()
                            .find(|p| {
                                p.has_capability("ProjectLeader", "Allow")
                                    && p.grantee_user_ids().contains(&user.id)
                            })
                            .is_some();
                        let is_project_owner = parent_project.owner_id == user.id;
                        if is_project_leader || is_project_owner {
                            // allow because project leader
                            perms
                                .iter()
                                .map(|(_, capa, mode)| {
                                    EffectivePermission::new(
                                        capa.to_string(),
                                        PermissionMode::Allow,
                                        vec![format!(
                                            "user is the {}.",
                                            if is_project_leader {
                                                "project leader"
                                            } else {
                                                "project owner"
                                            }
                                        )],
                                    )
                                })
                                .collect()
                        } else if asset.get_owner_id() == user.id {
                            // 3. content (asset) owner, allow
                            perms
                                .iter()
                                .map(|(_, capa, mode)| {
                                    EffectivePermission::new(
                                        capa.to_string(),
                                        PermissionMode::Allow,
                                        vec![format!("user is the owner of this content.")],
                                    )
                                })
                                .collect()
                        } else {
                            // 4. denied/allowed for user
                            // 5. denied/allowed for group
                            // apply the permission explicitly given
                            perms
                                .iter()
                                .map(|(grantee, capa, mode)| {
                                    let grantee_type = if matches!(grantee, Grantee::User(_)) {
                                        "user"
                                    } else {
                                        "group"
                                    };
                                    EffectivePermission::new(
                                        capa.to_string(),
                                        PermissionMode::from(mode.as_str()),
                                        vec![format!(
                                            "Permission set explicitly on {} {}.",
                                            grantee_type,
                                            grantee.get_name()
                                        )],
                                    )
                                })
                                .collect()
                        }
                    }
                } else {
                    // No license, deny access to this user.
                    perms
                        .iter()
                        .map(|(_, capa, mode)| {
                            EffectivePermission::new(
                                capa.to_string(),
                                PermissionMode::Deny,
                                vec![format!("User unlicensed or site role unknown.")],
                            )
                        })
                        .collect()
                };
                // Add final_permissions to ep[user][asset]
                ep.insert(
                    UserIdentifier::Email(user.email.to_owned()),
                    HashMap::from([(asset.cual().clone(), final_permissions)]),
                );
            });
        });
        ep
    }

    /// Get the series of parents all the way up for an owned asset.
    fn get_parent_projects_for<T: OwnedAsset>(&self, asset: &T) -> Vec<&nodes::Project> {
        if let Some(ProjectId(parent_project_id)) = asset.get_parent_project_id() {
            let parent = self
                .coordinator
                .env
                .projects
                .get(parent_project_id)
                .expect("getting parent project from env");
            self.get_parent_projects_for_project(parent)
        } else {
            vec![]
        }
    }

    /// Recursive method to get a series of project parents.
    fn get_parent_projects_for_project<'a>(
        &'a self,
        project: &'a nodes::Project,
    ) -> Vec<&'a nodes::Project> {
        if let Some(ProjectId(parent_project_id)) = project.get_parent_project_id() {
            let parent = self
                .coordinator
                .env
                .projects
                .get(parent_project_id)
                .expect("getting parent project from env");
            let mut result = self.get_parent_projects_for_project(parent);
            result.push(project);
            result
        } else {
            vec![]
        }
    }

    pub(crate) fn get_implicit_permissions_for_asset<T: OwnedAsset + Permissionable + Cualable>(
        &self,
        assets: &HashMap<String, T>,
    ) -> SparseMatrix<UserIdentifier, Cual, HashSet<EffectivePermission>> {
        let mut ep: SparseMatrix<UserIdentifier, Cual, HashSet<EffectivePermission>> =
            HashMap::new();
        assets.iter().for_each(|(_, asset)| {
            let asset_capabilities = super::get_capabilities_for_asset_type(asset.get_asset_type());
            // Content owners
            let owner = self
                .coordinator
                .env
                .users
                .get(asset.get_owner_id())
                .expect("getting user from env");
            let perms = asset_capabilities
                .iter()
                .map(|capa| {
                    EffectivePermission::new(
                        capa.to_string(),
                        PermissionMode::Allow,
                        vec!["user is the owner of this content".to_owned()],
                    )
                })
                .collect();
            // TODO: Check for clashes here.
            ep.insert(
                UserIdentifier::Email(owner.email.to_owned()),
                HashMap::from([(asset.cual(), perms)]),
            );
            // Project leaders
            for parent_project in self.get_parent_projects_for(asset) {
                parent_project.permissions.iter().for_each(|perm| {
                    if perm.capabilities.contains_key("ProjectLeader") {
                        let effective_perms: HashSet<EffectivePermission> = asset_capabilities
                            .iter()
                            .map(|capa| {
                                EffectivePermission::new(
                                    capa.to_string(),
                                    PermissionMode::Allow,
                                    vec![format!(
                                        "user is the leader of project {}",
                                        parent_project.name
                                    )],
                                )
                            })
                            .collect();
                        for grantee_email in perm.grantee_user_emails() {
                            // TODO: Check for clashes here.
                            ep.insert(
                                UserIdentifier::Email(grantee_email),
                                HashMap::from([(asset.cual(), effective_perms.clone())]),
                            );
                        }
                    }
                });
            }
        });
        ep
    }
}
