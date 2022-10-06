use std::collections::{HashMap, HashSet};

use jetty_core::{
    connectors::{
        nodes::{EffectivePermission, PermissionMode, SparseMatrix},
        UserIdentifier,
    },
    cual::{Cual, Cualable},
};

use super::matrix::{InsertOrMerge, Merge};
use crate::{
    coordinator::Coordinator,
    nodes::{self, user::SiteRole, OwnedAsset, ProjectId, TableauAsset},
    nodes::{Grantee, Permissionable},
};

pub(crate) struct PermissionManager<'x> {
    coordinator: &'x Coordinator,
}

impl<'x> PermissionManager<'x> {
    /// Basic constructor.
    pub(crate) fn new(coordinator: &'x Coordinator) -> Self {
        Self { coordinator }
    }

    /// Crate-public method for getting all effective permissions for an asset
    /// class.
    ///
    /// Gets explicit permissions, combines them with implicit permissions,
    /// and then combines those with the site-role-specific permissions.
    pub(crate) fn get_effective_permissions_for_asset<T: OwnedAsset + Permissionable + Cualable>(
        &self,
        assets: &HashMap<String, T>,
    ) -> SparseMatrix<UserIdentifier, Cual, HashSet<EffectivePermission>> {
        let mut all_perms = self.get_explicit_effective_permissions_for_asset(assets);
        all_perms
            .merge(self.get_implicit_effective_permissions_for_asset(assets))
            .expect("merging explicit and implicit permissions");
        all_perms
            .merge(self.get_effective_permissions_for_site_roles(assets))
            .expect("merging explicit/implicit permissions with site role permissions");

        all_perms
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

    /// Get all superusers from the environment.
    fn superusers(&self) -> impl Iterator<Item = &nodes::User> {
        self.coordinator
            .env
            .users
            .values()
            .filter(|user| match user.site_role {
                SiteRole::Creator
                | SiteRole::Explorer
                | SiteRole::ExplorerCanPublish
                | SiteRole::ReadOnly
                | SiteRole::SiteAdministratorExplorer
                | SiteRole::Viewer
                | SiteRole::Unlicensed
                | SiteRole::Unknown => false,
                SiteRole::ServerAdministrator | SiteRole::SiteAdministratorCreator => true,
            })
    }

    /// Get all unlicensed users from the environment.
    fn unlicensed_users(&self) -> impl Iterator<Item = &nodes::User> {
        self.coordinator
            .env
            .users
            .values()
            .filter(|user| match user.site_role {
                SiteRole::Creator
                | SiteRole::Explorer
                | SiteRole::ExplorerCanPublish
                | SiteRole::ReadOnly
                | SiteRole::SiteAdministratorExplorer
                | SiteRole::Viewer
                | SiteRole::ServerAdministrator
                | SiteRole::SiteAdministratorCreator => false,
                SiteRole::Unlicensed | SiteRole::Unknown => true,
            })
    }

    /// Get all effective permissions that are based on site role.
    fn get_effective_permissions_for_site_roles<T: OwnedAsset + Permissionable + Cualable>(
        &self,
        assets: &HashMap<String, T>,
    ) -> SparseMatrix<UserIdentifier, Cual, HashSet<EffectivePermission>> {
        let mut ep: SparseMatrix<UserIdentifier, Cual, HashSet<EffectivePermission>> =
            HashMap::new();
        for asset in assets.values() {
            let cual = asset.cual();

            // Superusers
            let superusers = self.superusers();
            let asset_capabilities = super::get_capabilities_for_asset_type(asset.get_asset_type());
            for su in superusers {
                let effective_permissions = asset_capabilities
                    .iter()
                    .map(|capa| {
                        EffectivePermission::new(
                            capa.to_string(),
                            PermissionMode::Allow,
                            vec![format!("user has site role {:?}", su.site_role)],
                        )
                    })
                    .collect();
                ep.insert_or_merge(
                    UserIdentifier::Email(su.email.to_owned()),
                    HashMap::from([(cual.clone(), effective_permissions)]),
                );
            }

            // Unlicensed/unknown users
            let unlicensed_users = self.unlicensed_users();
            let asset_capabilities = super::get_capabilities_for_asset_type(asset.get_asset_type());
            for user in unlicensed_users {
                let effective_permissions = asset_capabilities
                    .iter()
                    .map(|capa| {
                        EffectivePermission::new(
                            capa.to_string(),
                            PermissionMode::Deny,
                            vec![format!("user has site role {:?}", user.site_role)],
                        )
                    })
                    .collect();
                ep.insert_or_merge(
                    UserIdentifier::Email(user.email.to_owned()),
                    HashMap::from([(cual.clone(), effective_permissions)]),
                );
            }
        }
        ep
    }

    /// Get all effective permissions that are explicitly set for the assets
    /// given.
    fn get_explicit_effective_permissions_for_asset<T: OwnedAsset + Permissionable + Cualable>(
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
                // apply the permission explicitly given
                let explicit_effective_permissions = perms
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
                    .collect();
                // Add permissions to ep[user][asset]
                ep.insert(
                    UserIdentifier::Email(user.email.to_owned()),
                    HashMap::from([(asset.cual().clone(), explicit_effective_permissions)]),
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

    /// Get all effective permissions for permissions implicitly defined by
    /// content ownership and project leadership.
    fn get_implicit_effective_permissions_for_asset<T: OwnedAsset + Permissionable + Cualable>(
        &self,
        assets: &HashMap<String, T>,
    ) -> SparseMatrix<UserIdentifier, Cual, HashSet<EffectivePermission>> {
        let mut ep: SparseMatrix<UserIdentifier, Cual, HashSet<EffectivePermission>> =
            HashMap::new();
        assets.iter().for_each(|(_, asset)| {
            let asset_capabilities = super::get_capabilities_for_asset_type(asset.get_asset_type());
            // Content owners
            let some_owner = self.coordinator.env.users.get(asset.get_owner_id());
            if let Some(owner) = some_owner {
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
                ep.insert_or_merge(
                    UserIdentifier::Email(owner.email.to_owned()),
                    HashMap::from([(asset.cual(), perms)]),
                );

            // Project leaders
            for parent_project in self.get_parent_projects_for(asset) {
                for perm in &parent_project.permissions {
                    if perm.capabilities.contains_key("ProjectLeader") {
                        let leader_effective_permissions: HashSet<EffectivePermission> =
                            asset_capabilities
                                .iter()
                                .map(|capa| {
                                    EffectivePermission::new(
                                        capa.to_string(),
                                        PermissionMode::Allow,
                                        vec![format!(
                                            "user has the project leader role on {}",
                                            parent_project.name
                                        )],
                                    )
                                })
                                .collect();
                        for grantee_email in perm.grantee_user_emails() {
                            ep.insert_or_merge(
                                UserIdentifier::Email(grantee_email),
                                HashMap::from([(
                                    asset.cual(),
                                    leader_effective_permissions.clone(),
                                )]),
                            );
                        }
                    }
                }
            }
            } else {
                // We assume the asset is the default project with the default owner, it's not going to be in the env.
                println!("Failed getting user {:?} from env. Assuming it's the default project default owner.", asset.get_owner_id());
            }
        });
        ep
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        coordinator::Environment,
        nodes::{Flow, Project, User},
        rest::TableauRestClient,
    };

    use super::*;

    #[test]
    fn test_effective_perms_for_asset_works() {
        let mut env = Environment::default();
        let mut user = User::default();
        user.site_role = SiteRole::SiteAdministratorCreator;
        env.flows = HashMap::from([("flow".to_owned(), Flow::default())]);
        env.users = HashMap::from([("".to_owned(), user)]);
        env.projects = HashMap::from([("".to_owned(), Project::default())]);
        let rest_client = TableauRestClient::new_dummy();
        let coordinator = &Coordinator {
            env: env,
            rest_client,
        };

        let m = PermissionManager::new(coordinator);

        let ep = m.get_effective_permissions_for_asset(&m.coordinator.env.flows);
        assert_eq!(
            ep,
            HashMap::from([(
                UserIdentifier::Email("".to_owned(),),
                HashMap::from([(
                    Cual::new("".to_owned(),),
                    HashSet::from([
                        EffectivePermission {
                            privilege: "Read".to_owned(),
                            mode: PermissionMode::Allow,
                            reasons: vec![
                                "user is the owner of this content".to_owned(),
                                "user has site role SiteAdministratorCreator".to_owned(),
                            ],
                        },
                        EffectivePermission {
                            privilege: "Write".to_owned(),
                            mode: PermissionMode::Allow,
                            reasons: vec![
                                "user is the owner of this content".to_owned(),
                                "user has site role SiteAdministratorCreator".to_owned(),
                            ],
                        },
                        EffectivePermission {
                            privilege: "ChangeHierarchy".to_owned(),
                            mode: PermissionMode::Allow,
                            reasons: vec![
                                "user is the owner of this content".to_owned(),
                                "user has site role SiteAdministratorCreator".to_owned(),
                            ],
                        },
                        EffectivePermission {
                            privilege: "Execute".to_owned(),
                            mode: PermissionMode::Allow,
                            reasons: vec![
                                "user is the owner of this content".to_owned(),
                                "user has site role SiteAdministratorCreator".to_owned(),
                            ],
                        },
                        EffectivePermission {
                            privilege: "Delete".to_owned(),
                            mode: PermissionMode::Allow,
                            reasons: vec![
                                "user is the owner of this content".to_owned(),
                                "user has site role SiteAdministratorCreator".to_owned(),
                            ],
                        },
                        EffectivePermission {
                            privilege: "ExportXml".to_owned(),
                            mode: PermissionMode::Allow,
                            reasons: vec![
                                "user is the owner of this content".to_owned(),
                                "user has site role SiteAdministratorCreator".to_owned(),
                            ],
                        },
                        EffectivePermission {
                            privilege: "ChangePermissions".to_owned(),
                            mode: PermissionMode::Allow,
                            reasons: vec![
                                "user is the owner of this content".to_owned(),
                                "user has site role SiteAdministratorCreator".to_owned(),
                            ],
                        },
                        EffectivePermission {
                            privilege: "WebAuthoringForFlows".to_owned(),
                            mode: PermissionMode::Allow,
                            reasons: vec![
                                "user is the owner of this content".to_owned(),
                                "user has site role SiteAdministratorCreator".to_owned(),
                            ],
                        },
                    ])
                )])
            )])
        );
    }
}