//! Tableau Connector for Jetty
//!

#![deny(missing_docs)]
#![allow(dead_code)]

mod coordinator;
mod file_parse;
mod nodes;
mod permissions;
mod rest;

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;

use coordinator::Environment;
use futures::{future::BoxFuture, Future, StreamExt};
use reqwest::RequestBuilder;
use rest::get_cual_prefix;
pub use rest::TableauRestClient;
use serde::{Deserialize, Serialize};
use serde_json::json;

use jetty_core::{
    access_graph::translate::diffs::{groups, LocalDiffs},
    connectors::{
        nodes::ConnectorData,
        nodes::{self as jetty_nodes, EffectivePermission, SparseMatrix},
        AssetType, ConnectorCapabilities, ConnectorClient, NewConnector, ReadCapabilities,
        WriteCapabilities,
    },
    cual::Cual,
    jetty::{ConnectorConfig, ConnectorManifest, CredentialsMap},
    logging::error,
    permissions::matrix::Merge,
    Connector,
};

use nodes::{asset_to_policy::env_to_jetty_policies, FromTableau};
use permissions::{
    consts::{
        DATASOURCE_CAPABILITIES, FLOW_CAPABILITIES, LENS_CAPABILITIES, METRIC_CAPABILITIES,
        PROJECT_CAPABILITIES, VIEW_CAPABILITIES, WORKBOOK_CAPABILITIES,
    },
    PermissionManager,
};

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    pin::Pin,
    sync::{Arc, Mutex},
};

/// Map wrapper for config values.
pub type TableauConfig = HashMap<String, String>;

/// Credentials for authenticating with Tableau.
///
/// The user sets these up by following Jetty documentation
/// and pasting their connection info into their connector config.
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct TableauCredentials {
    #[serde(flatten)]
    method: LoginMethod,
    /// Tableau server name like 10ay.online.tableau.com *without* the `https://`
    pub(crate) server_name: String,
    pub(crate) site_name: String,
}

/// Enum representing the different types of tableau login methods and the required information
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "login_method")]
#[serde(rename_all = "snake_case")]
pub enum LoginMethod {
    /// Log in using username and password
    UsernameAndPassword {
        /// Tableau username
        username: String,
        /// Tableau password
        password: String,
    },
    /// Log in using personal access token
    PersonalAccessToken {
        /// Tableau personal access token name
        token_name: String,
        /// Tableau personal access token secret
        secret: String,
    },
}

impl Default for LoginMethod {
    fn default() -> Self {
        Self::UsernameAndPassword {
            username: "name".to_owned(),
            password: "password".to_owned(),
        }
    }
}

impl TableauCredentials {
    /// Basic constructor.
    pub fn new(method: LoginMethod, server_name: String, site_name: String) -> Self {
        Self {
            method,
            server_name,
            site_name,
        }
    }

    /// Given a TableauCredentials object, return a CredentialsMap
    pub fn to_map(&self) -> CredentialsMap {
        let string_rep = serde_json::to_string(&self).unwrap();
        let map: CredentialsMap = serde_json::from_str(&string_rep).unwrap();
        map
    }

    /// Given a map of credentials, return a TableauCredentials object.
    pub(crate) fn from_map(m: &CredentialsMap) -> Result<Self> {
        let string_rep = serde_json::to_string(m)?;
        Ok(serde_json::from_str(&string_rep)?)
    }
}

/// Top-level connector struct.
#[allow(dead_code)]
#[derive(Default)]
pub struct TableauConnector {
    config: TableauConfig,
    coordinator: coordinator::Coordinator,
}

impl TableauConnector {
    /// Setup after creation. Fetch and update the local environment.
    pub async fn setup(&mut self) -> Result<()> {
        self.coordinator.update_env().await?;
        Ok(())
    }

    /// Get the environment, but transformed into Jetty objects.
    fn env_to_jetty_all(
        &self,
    ) -> (
        Vec<jetty_nodes::RawGroup>,
        Vec<jetty_nodes::RawUser>,
        Vec<jetty_nodes::RawAsset>,
        Vec<jetty_nodes::RawTag>,
        Vec<jetty_nodes::RawPolicy>,
        Vec<jetty_nodes::RawDefaultPolicy>,
    ) {
        // Transform assets
        let flows: Vec<jetty_nodes::RawAsset> =
            self.object_to_jetty(&self.coordinator.env.flows, &self.coordinator.env);
        let projects = self.object_to_jetty(&self.coordinator.env.projects, &self.coordinator.env);
        let lenses = self.object_to_jetty(&self.coordinator.env.lenses, &self.coordinator.env);
        let datasources =
            self.object_to_jetty(&self.coordinator.env.datasources, &self.coordinator.env);
        let workbooks =
            self.object_to_jetty(&self.coordinator.env.workbooks, &self.coordinator.env);
        let metrics = self.object_to_jetty(&self.coordinator.env.metrics, &self.coordinator.env);
        let views = self.object_to_jetty(&self.coordinator.env.views, &self.coordinator.env);

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
        let all_policies = env_to_jetty_policies(&self.coordinator.env);

        // Get default policies for each project
        let default_policies = self
            .coordinator
            .env
            .projects
            .iter()
            .flat_map(|(_, project)| project.get_default_policies(&self.coordinator.env))
            .collect();

        (
            self.to_jetty(&self.coordinator.env.groups),
            self.to_jetty(&self.coordinator.env.users),
            all_assets,
            vec![], // self.object_to_jetty(&self.coordinator.env.tags);
            all_policies,
            default_policies,
        )
    }

    fn get_effective_permissions(
        &self,
    ) -> SparseMatrix<String, Cual, HashSet<EffectivePermission>> {
        let permission_manager = PermissionManager::new(&self.coordinator);
        let mut final_eps: SparseMatrix<String, Cual, HashSet<EffectivePermission>> =
            HashMap::new();
        let flow_eps =
            permission_manager.get_effective_permissions_for_asset(&self.coordinator.env.flows);
        let project_eps =
            permission_manager.get_effective_permissions_for_asset(&self.coordinator.env.projects);
        let lens_eps =
            permission_manager.get_effective_permissions_for_asset(&self.coordinator.env.lenses);
        let datasource_eps = permission_manager
            .get_effective_permissions_for_asset(&self.coordinator.env.datasources);
        let workbook_eps =
            permission_manager.get_effective_permissions_for_asset(&self.coordinator.env.workbooks);
        let metric_eps =
            permission_manager.get_effective_permissions_for_asset(&self.coordinator.env.metrics);
        let view_eps =
            permission_manager.get_effective_permissions_for_asset(&self.coordinator.env.views);

        final_eps.merge(flow_eps).unwrap();
        final_eps.merge(project_eps).unwrap();
        final_eps.merge(lens_eps).unwrap();
        final_eps.merge(datasource_eps).unwrap();
        final_eps.merge(workbook_eps).unwrap();
        final_eps.merge(metric_eps).unwrap();
        final_eps.merge(view_eps).unwrap();
        final_eps
    }

    fn to_jetty<O, J>(&self, obj_map: &HashMap<String, O>) -> Vec<J>
    where
        O: Into<J> + Clone,
    {
        obj_map.clone().into_values().map(|x| x.into()).collect()
    }

    fn object_to_jetty<O, J>(&self, obj_map: &HashMap<String, O>, env: &Environment) -> Vec<J>
    where
        J: FromTableau<O>,
        O: Clone,
    {
        obj_map
            .clone()
            .values()
            .map(|x| J::from(x.clone(), env))
            .collect()
    }

    fn generate_request_plan(&self, diffs: &LocalDiffs) -> Result<Vec<Vec<String>>> {
        let mut batch1 = Vec::new();
        let mut batch2 = Vec::new();

        let base_url = format![
            "https://{}/api/{}/sites/{}/",
            self.coordinator.rest_client.get_server_name(),
            self.coordinator.rest_client.get_api_version(),
            self.coordinator.rest_client.get_site_id()?,
        ];
        // Starting with groups
        let group_diffs = &diffs.groups;
        for diff in group_diffs {
            match &diff.details {
                groups::LocalDiffDetails::AddGroup { members } => {
                    // Request to create the group

                    batch1.push(format!(
                        r#"POST {base_url}groups
body:
  {{
    "group": {{
      "name": {},
    }}
  }}"#,
                        diff.group_name
                    ));

                    // Requests to add users
                    for user in &members.users {
                        batch1.push(format!(
                            r#"POST {base_url}groups/<new group_id for {}>/users
body:
  {{
    "user": {{
      "id": {user},
    }}
  }}"#,
                            diff.group_name
                        ));
                    }
                }
                groups::LocalDiffDetails::RemoveGroup => {
                    // get the group_id
                    let group_id = self
                        .coordinator
                        .env
                        .get_group_id_by_name(&diff.group_name)
                        .ok_or(anyhow!(
                            "can't delete group {}: group doesn't exist",
                            &diff.group_name
                        ))?;

                    batch1.push(format!(
                        "DELETE {base_url}groups/{group_id}\n## {group_id} is the id for {}\n",
                        diff.group_name
                    ));
                }
                groups::LocalDiffDetails::ModifyGroup { add, remove } => {
                    // get the group_id
                    let group_id = self
                        .coordinator
                        .env
                        .get_group_id_by_name(&diff.group_name)
                        .ok_or(anyhow!(
                            "can't delete group {}: group doesn't exist",
                            &diff.group_name
                        ))?;

                    // Add users
                    for user in &add.users {
                        batch2.push(format!(
                            r#"POST {base_url}groups/{group_id}/users
body:
  {{
    "user": {{
      "id": {user},
    }}
  }}"#
                        ));
                    }

                    // Remove users
                    for user in &remove.users {
                        batch2.push(format!(
                            r#"DELETE {base_url}groups/{group_id}/users/{user}"#
                        ));
                    }
                }
            }
        }
        Ok(vec![batch1, batch2])
    }

    fn generate_plan_futures<'a>(
        &'a self,
        diffs: &'a LocalDiffs,
    ) -> Result<Vec<Vec<Pin<Box<dyn Future<Output = Result<()>> + '_ + Send>>>>> {
        let mut batch1: Vec<BoxFuture<_>> = Vec::new();
        let mut batch2: Vec<BoxFuture<_>> = Vec::new();

        let group_map: HashMap<String, String> = self
            .coordinator
            .env
            .groups
            .iter()
            .map(|(_name, g)| (g.name.to_owned(), g.id.to_owned()))
            .collect();

        let group_map_mutex = Arc::new(Mutex::new(group_map));

        // Starting with groups
        let group_diffs = &diffs.groups;
        for diff in group_diffs {
            match &diff.details {
                groups::LocalDiffDetails::AddGroup { members } => {
                    // start by creating the group
                    batch1.push(Box::pin(self.create_group_and_add_to_env(
                        &diff.group_name,
                        group_map_mutex.clone(),
                    )));
                    for user in &members.users {
                        batch2.push(Box::pin(self.add_user_to_group(
                            user,
                            &diff.group_name,
                            Arc::clone(&group_map_mutex),
                        )))
                    }
                }
                groups::LocalDiffDetails::RemoveGroup => {
                    // get the group_id
                    let temp_group_map = group_map_mutex.lock().unwrap();
                    let group_id = temp_group_map
                        .get(&diff.group_name)
                        .ok_or(anyhow!("Unable to find group id for {}", &diff.group_name))?;

                    let req = self.coordinator.rest_client.build_request(
                        format!("groups/{group_id}"),
                        None,
                        reqwest::Method::DELETE,
                    )?;
                    batch1.push(Box::pin(request_builder_to_unit_result(req)))
                }
                groups::LocalDiffDetails::ModifyGroup { add, remove } => {
                    // Add users
                    for user in &add.users {
                        batch2.push(Box::pin(self.add_user_to_group(
                            user,
                            &diff.group_name,
                            group_map_mutex.clone(),
                        )))
                    }
                    // Remove users
                    // get the group_id
                    let temp_group_map = group_map_mutex.lock().unwrap();
                    let group_id = temp_group_map
                        .get(&diff.group_name)
                        .ok_or(anyhow!("Unable to find group id for {}", &diff.group_name))?;

                    for user in &remove.users {
                        let req = self.coordinator.rest_client.build_request(
                            format!("groups/{group_id}/users/{user}"),
                            None,
                            reqwest::Method::DELETE,
                        )?;
                        batch1.push(Box::pin(request_builder_to_unit_result(req)))
                    }
                }
            }
        }
        Ok(vec![batch1, batch2])
    }

    async fn create_group_and_add_to_env(
        &self,
        group_name: &String,
        group_map: Arc<Mutex<HashMap<String, String>>>,
    ) -> Result<()> {
        let req_body = json!({"group": { "name": group_name }});
        let req = self.coordinator.rest_client.build_request(
            "groups/".to_string(),
            Some(req_body),
            reqwest::Method::POST,
        )?;
        let resp = req.send().await?.json::<serde_json::Value>().await?;

        let group_id = rest::get_json_from_path(&resp, &vec!["group".to_owned(), "id".to_owned()])?
            .as_str()
            .ok_or_else(|| anyhow!["unable to get new id for {group_name}"])?
            .to_string();

        // update the environment so that when users look for this group in the future, they are able to find it!
        let mut locked_group_map = group_map.lock().unwrap();
        locked_group_map.insert(group_name.to_owned(), group_id);
        Ok(())
    }

    /// Function to add users to a group, deferring the group lookup until it's needed. This
    /// allows it to work for new groups
    async fn add_user_to_group(
        &self,
        user: &String,
        group_name: &String,
        group_map: Arc<Mutex<HashMap<String, String>>>,
    ) -> Result<()> {
        // get the group_id
        let mut group_id = "".to_owned();
        {
            let temp_group_map = group_map.lock().unwrap();
            group_id = temp_group_map
                .get(group_name)
                .ok_or(anyhow!("Unable to find group id for {}", group_name))?
                .to_owned();
        }

        // Add the user
        let req_body = json!({"user": {"id": user}});
        self.coordinator
            .rest_client
            .build_request(
                format!("groups/{group_id}/users"),
                Some(req_body),
                reqwest::Method::POST,
            )?
            .send()
            .await?;

        Ok(())
    }
}

async fn request_builder_to_unit_result(req: RequestBuilder) -> Result<()> {
    req.send().await?;
    Ok(())
}

#[async_trait]
impl NewConnector for TableauConnector {
    /// Validates the configs and bootstraps a Tableau connection.
    ///
    /// Validates that the required fields are present to authenticate to
    /// Tableau. Stashes the credentials in the struct for use when
    /// connecting.
    async fn new(
        config: &ConnectorConfig,
        credentials: &CredentialsMap,
        _client: Option<ConnectorClient>,
        data_dir: Option<PathBuf>,
    ) -> Result<Box<Self>> {
        let creds = TableauCredentials::from_map(credentials)?;

        let tableau_connector = TableauConnector {
            config: config.config.to_owned(),
            coordinator: coordinator::Coordinator::new(creds, data_dir).await?,
        };

        Ok(Box::new(tableau_connector))
    }
}

#[async_trait]
impl Connector for TableauConnector {
    async fn check(&self) -> bool {
        todo!()
    }

    async fn get_data(&mut self) -> ConnectorData {
        self.setup().await.unwrap();
        let (groups, users, assets, tags, policies, default_policies) = self.env_to_jetty_all();
        let effective_permissions = self.get_effective_permissions();
        ConnectorData {
            groups,
            users,
            assets,
            tags,
            policies,
            default_policies,
            asset_references: Default::default(),
            effective_permissions,
            cual_prefix: Some(
                get_cual_prefix()
                    .context("tableau cual prefix not yet set")
                    .unwrap()
                    .to_owned(),
            ),
        }
    }

    fn get_manifest(&self) -> ConnectorManifest {
        let asset_privileges = [
            (
                AssetType("workbook".to_owned()),
                WORKBOOK_CAPABILITIES
                    .iter()
                    .flat_map(|v| [format!("Allow{v}"), format!("Deny{v}")])
                    .collect(),
            ),
            (
                AssetType("lens".to_owned()),
                LENS_CAPABILITIES
                    .iter()
                    .flat_map(|v| [format!("Allow{v}"), format!("Deny{v}")])
                    .collect(),
            ),
            (
                AssetType("datasource".to_owned()),
                DATASOURCE_CAPABILITIES
                    .iter()
                    .flat_map(|v| [format!("Allow{v}"), format!("Deny{v}")])
                    .collect(),
            ),
            (
                AssetType("flow".to_owned()),
                FLOW_CAPABILITIES
                    .iter()
                    .flat_map(|v| [format!("Allow{v}"), format!("Deny{v}")])
                    .collect(),
            ),
            (
                AssetType("metric".to_owned()),
                METRIC_CAPABILITIES
                    .iter()
                    .flat_map(|v| [format!("Allow{v}"), format!("Deny{v}")])
                    .collect(),
            ),
            (
                AssetType("project".to_owned()),
                PROJECT_CAPABILITIES
                    .iter()
                    .flat_map(|v| [format!("Allow{v}"), format!("Deny{v}")])
                    .collect(),
            ),
            (
                AssetType("view".to_owned()),
                VIEW_CAPABILITIES
                    .iter()
                    .flat_map(|v| [format!("Allow{v}"), format!("Deny{v}")])
                    .collect(),
            ),
        ]
        .into();

        ConnectorManifest {
            capabilities: ConnectorCapabilities {
                read: HashSet::from([
                    ReadCapabilities::Assets,
                    ReadCapabilities::Groups,
                    ReadCapabilities::Policies {
                        default_policies: true,
                    },
                    ReadCapabilities::Users,
                ]),
                write: HashSet::from([
                    WriteCapabilities::Groups { nested: false },
                    WriteCapabilities::Policies {
                        default_policies: true,
                    },
                ]),
            },
            asset_privileges,
        }
    }

    fn plan_changes(&self, diffs: &LocalDiffs) -> Vec<String> {
        match self.generate_request_plan(diffs) {
            Ok(plan) => plan.into_iter().flatten().collect(),
            Err(err) => {
                error!("Unable to generate plan for Tableau: {err}");
                vec![]
            }
        }
    }

    async fn apply_changes(&self, diffs: &LocalDiffs) -> Result<String> {
        let mut success_counter = 0;
        let mut failure_counter = 0;
        // This is designed in such a way that each query_set may be run concurrently.
        for request_set in self.generate_plan_futures(diffs)? {
            let results = futures::stream::iter(request_set)
                .buffered(coordinator::CONCURRENT_METADATA_FETCHES)
                .collect::<Vec<_>>()
                .await;

            for result in results {
                match result {
                    Err(e) => {
                        error!("{:?}", e);
                        failure_counter += 1;
                    }
                    Ok(_) => {
                        success_counter += 1;
                    }
                }
            }
        }
        Ok(format!(
            "{} successful requests\n{} failed requests",
            success_counter, failure_counter
        ))
    }
}
