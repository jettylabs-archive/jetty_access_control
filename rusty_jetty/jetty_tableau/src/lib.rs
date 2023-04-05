//! Tableau Connector for Jetty
//!

#![deny(missing_docs)]

mod coordinator;
mod lineage;
mod nodes;
mod origin;
mod permissions;
pub(crate) mod rest;
mod write;

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;

use coordinator::Environment;
use futures::StreamExt;
use rest::get_cual_prefix;
pub use rest::TableauRestClient;
use serde::{Deserialize, Serialize};
use serde_json::json;

use jetty_core::{
    access_graph::translate::diffs::LocalConnectorDiffs,
    connectors::{
        nodes as jetty_nodes, nodes::ConnectorData, AssetType, ConnectorCapabilities,
        ConnectorClient, NewConnector, ReadCapabilities, WriteCapabilities,
    },
    cual::Cual,
    jetty::{ConnectorConfig, ConnectorManifest, CredentialsMap},
    logging::error,
    Connector,
};

use nodes::{asset_to_policy::env_to_jetty_policies, FromTableau};
use permissions::consts::{
    DATASOURCE_CAPABILITIES, FLOW_CAPABILITIES, LENS_CAPABILITIES, METRIC_CAPABILITIES,
    PROJECT_CAPABILITIES, VIEW_CAPABILITIES, WORKBOOK_CAPABILITIES,
};

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

/// Map wrapper for config values.
pub type TableauConfig = HashMap<String, serde_json::Value>;

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
#[derive(Default)]
pub struct TableauConnector {
    _config: TableauConfig,
    coordinator: coordinator::Coordinator,
}

impl TableauConnector {
    /// Setup after creation. Fetch and update the local environment.
    pub async fn setup(&mut self) -> Result<()> {
        self.coordinator.update_env().await?;
        Ok(())
    }

    /// Get the environment, but transformed into Jetty objects.
    #[allow(clippy::type_complexity)]
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
            _config: config.config.to_owned(),
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

        // Actually, don't get effective permissions for now. It's too slow.
        // let effective_permissions = self.get_effective_permissions();

        let effective_permissions = Default::default();
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

    fn plan_changes(&self, diffs: &LocalConnectorDiffs) -> Vec<String> {
        match self.generate_request_plan(diffs) {
            Ok(plan) => plan.flatten_to_string_vec(),
            Err(err) => {
                error!("Unable to generate plan for Tableau: {err}");
                vec![]
            }
        }
    }

    async fn apply_changes(&self, diffs: &LocalConnectorDiffs) -> Result<String> {
        let mut success_counter = 0;
        let mut failure_counter = 0;
        // This is designed in such a way that each query_set may be run concurrently.
        let futures = self.generate_plan_futures(diffs)?;

        for request_set in [futures.0, futures.1, futures.2] {
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
            "{success_counter} successful requests\n{failure_counter} failed requests"
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use dirs::home_dir;
    use jetty_core::{fetch_credentials, jetty::ConnectorNamespace, Jetty};

    use super::*;

    pub(crate) async fn get_live_tableau_connector() -> Result<Box<TableauConnector>> {
        let jetty = Jetty::new(
            "jetty_config.yaml",
            Path::new("data").into(),
            Default::default(),
        )?;
        let creds = fetch_credentials(home_dir().unwrap().join(".jetty/connectors.yaml"))?;

        crate::TableauConnector::new(
            &jetty.config.connectors[&ConnectorNamespace("tableau".to_owned())],
            &creds["tableau"],
            Some(ConnectorClient::Core),
            None,
        )
        .await
    }
}
