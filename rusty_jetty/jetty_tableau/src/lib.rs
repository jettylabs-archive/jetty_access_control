#![allow(dead_code, unused)]

mod coordinator;
mod file_parse;
mod nodes;
mod permissions;
mod rest;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use permissions::get_capabilities_for_asset_type;
use rest::{TableauAssetType, TableauRestClient};
use serde::Deserialize;
use serde_json::json;

use jetty_core::{
    connectors::{
        nodes::{self as jetty_nodes, EffectivePermission, SparseMatrix},
        nodes::{ConnectorData, PermissionMode},
        ConnectorClient, UserIdentifier,
    },
    cual::{Cual, Cualable},
    jetty::{ConnectorConfig, CredentialsBlob},
    Connector,
};

use nodes::{
    asset_to_policy::env_to_jetty_policies, user::SiteRole, Grantee, OwnedAsset, Permissionable,
    ProjectId,
};

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

pub type TableauConfig = HashMap<String, String>;

/// Credentials for authenticating with Tableau.
///
/// The user sets these up by following Jetty documentation
/// and pasting their connection info into their connector config.
#[derive(Deserialize, Debug, Default)]
struct TableauCredentials {
    username: String,
    password: String,
    /// Tableau server name like 10ay.online.tableau.com *without* the `https://`
    server_name: String,
    site_name: String,
}

#[allow(dead_code)]
#[derive(Default)]
pub struct TableauConnector {
    config: TableauConfig,
    coordinator: coordinator::Coordinator,
}

impl TableauConnector {
    pub async fn setup(&mut self) -> Result<()> {
        self.coordinator.update_env().await?;
        Ok(())
    }

    /// Get the environment, but transformed into Jetty objects.
    fn env_to_jetty_all(
        &self,
    ) -> (
        Vec<jetty_nodes::Group>,
        Vec<jetty_nodes::User>,
        Vec<jetty_nodes::Asset>,
        Vec<jetty_nodes::Tag>,
        Vec<jetty_nodes::Policy>,
    ) {
        // Transform assets
        let flows: Vec<jetty_nodes::Asset> = self.object_to_jetty(&self.coordinator.env.flows);
        let projects = self.object_to_jetty(&self.coordinator.env.projects);
        let lenses = self.object_to_jetty(&self.coordinator.env.lenses);
        let datasources = self.object_to_jetty(&self.coordinator.env.datasources);
        let workbooks = self.object_to_jetty(&self.coordinator.env.workbooks);
        let metrics = self.object_to_jetty(&self.coordinator.env.metrics);
        let views = self.object_to_jetty(&self.coordinator.env.views);

        let all_assets = flows
            .into_iter()
            .chain(projects.into_iter())
            .chain(lenses.into_iter())
            .chain(datasources.into_iter())
            .chain(workbooks.into_iter())
            .chain(metrics.into_iter())
            .chain(views.into_iter())
            .collect();

        // Transform policies
        let flow_policies: Vec<jetty_nodes::Policy> =
            env_to_jetty_policies(&mut self.coordinator.env.flows.clone().into_values());
        let project_policies: Vec<jetty_nodes::Policy> =
            env_to_jetty_policies(&mut self.coordinator.env.projects.clone().into_values());
        let lens_policies: Vec<jetty_nodes::Policy> =
            env_to_jetty_policies(&mut self.coordinator.env.lenses.clone().into_values());
        let datasource_policies: Vec<jetty_nodes::Policy> =
            env_to_jetty_policies(&mut self.coordinator.env.datasources.clone().into_values());
        let workbook_policies: Vec<jetty_nodes::Policy> =
            env_to_jetty_policies(&mut self.coordinator.env.workbooks.clone().into_values());
        let metric_policies: Vec<jetty_nodes::Policy> =
            env_to_jetty_policies(&mut self.coordinator.env.metrics.clone().into_values());
        let view_policies: Vec<jetty_nodes::Policy> =
            env_to_jetty_policies(&mut self.coordinator.env.views.clone().into_values());
        let all_policies = flow_policies
            .into_iter()
            .chain(project_policies.into_iter())
            .chain(lens_policies.into_iter())
            .chain(datasource_policies.into_iter())
            .chain(workbook_policies.into_iter())
            .chain(metric_policies.into_iter())
            .chain(view_policies.into_iter())
            .collect();

        (
            self.object_to_jetty(&self.coordinator.env.groups),
            self.object_to_jetty(&self.coordinator.env.users),
            all_assets,
            vec![], // self.object_to_jetty(&self.coordinator.env.tags);
            all_policies,
        )
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

    fn get_effective_permissions_for_asset<T: OwnedAsset + Permissionable + Cualable>(
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
                                    PermissionMode::from(PermissionMode::Allow),
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

    fn get_implicit_permissions_for_asset<T: OwnedAsset + Permissionable + Cualable>(
        &self,
        assets: &HashMap<String, T>,
    ) -> SparseMatrix<UserIdentifier, Cual, HashSet<EffectivePermission>> {
        let mut ep: SparseMatrix<UserIdentifier, Cual, HashSet<EffectivePermission>> =
            HashMap::new();
        assets.iter().for_each(|(_, asset)| {
            let asset_capabilities = get_capabilities_for_asset_type(asset.get_asset_type());
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

    fn get_effective_permissions(
        &self,
    ) -> SparseMatrix<UserIdentifier, Cual, HashSet<EffectivePermission>> {
        let mut final_eps = HashMap::new();
        let mut flow_eps = self.get_effective_permissions_for_asset(&self.coordinator.env.flows);
        // TODO: merge the reasons here when there are matching perms
        flow_eps.extend(self.get_implicit_permissions_for_asset(&self.coordinator.env.flows));
        let project_eps = self.get_effective_permissions_for_asset(&self.coordinator.env.projects);
        let lens_eps = self.get_effective_permissions_for_asset(&self.coordinator.env.lenses);
        let datasource_eps =
            self.get_effective_permissions_for_asset(&self.coordinator.env.datasources);
        let workbook_eps =
            self.get_effective_permissions_for_asset(&self.coordinator.env.workbooks);
        let metric_eps = self.get_effective_permissions_for_asset(&self.coordinator.env.metrics);
        let view_eps = self.get_effective_permissions_for_asset(&self.coordinator.env.views);

        final_eps.extend(flow_eps.into_iter());
        final_eps.extend(project_eps.into_iter());
        final_eps.extend(lens_eps.into_iter());
        final_eps.extend(datasource_eps.into_iter());
        final_eps.extend(workbook_eps.into_iter());
        final_eps.extend(metric_eps.into_iter());
        final_eps.extend(view_eps.into_iter());
        final_eps
    }

    fn object_to_jetty<O, J>(&self, obj_map: &HashMap<String, O>) -> Vec<J>
    where
        O: Into<J> + Clone,
    {
        obj_map.clone().into_values().map(|x| x.into()).collect()
    }
}

#[async_trait]
impl Connector for TableauConnector {
    /// Validates the configs and bootstraps a Tableau connection.
    ///
    /// Validates that the required fields are present to authenticate to
    /// Tableau. Stashes the credentials in the struct for use when
    /// connecting.
    async fn new(
        config: &ConnectorConfig,
        credentials: &CredentialsBlob,
        _client: Option<ConnectorClient>,
    ) -> Result<Box<Self>> {
        let mut creds = TableauCredentials::default();
        let mut required_fields = HashSet::from([
            "server_name".to_owned(),
            "site_name".to_owned(),
            "username".to_owned(),
            "password".to_owned(),
        ]);

        for (k, v) in credentials.iter() {
            match k.as_ref() {
                "server_name" => creds.server_name = v.to_string(),
                "site_name" => creds.site_name = v.to_string(),
                "username" => creds.username = v.to_string(),
                "password" => creds.password = v.to_string(),
                _ => (),
            }

            required_fields.remove(k);
        }

        if !required_fields.is_empty() {
            return Err(anyhow![
                "Snowflake config missing required fields: {:#?}",
                required_fields
            ]);
        }

        let tableau_connector = TableauConnector {
            config: config.config.to_owned(),
            coordinator: coordinator::Coordinator::new(creds).await,
        };

        Ok(Box::new(tableau_connector))
    }

    async fn check(&self) -> bool {
        todo!()
    }

    async fn get_data(&mut self) -> ConnectorData {
        let (groups, users, assets, tags, policies) = self.env_to_jetty_all();
        let effective_permissions = self.get_effective_permissions();
        ConnectorData::new(groups, users, assets, tags, policies, effective_permissions)
    }
}

#[cfg(test)]
pub(crate) async fn connector_setup() -> Result<crate::TableauConnector> {
    use anyhow::Context;
    use jetty_core::Connector;

    let j = jetty_core::jetty::Jetty::new().context("creating Jetty")?;
    let creds = jetty_core::jetty::fetch_credentials().context("fetching credentials from file")?;
    let config = &j.config.connectors[0];
    let tc = crate::TableauConnector::new(config, &creds["tableau"], None)
        .await
        .context("reading tableau credentials")?;
    Ok(*tc)
}
