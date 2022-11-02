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
use rest::get_cual_prefix;
pub use rest::TableauRestClient;
use serde::Deserialize;
use serde_json::json;

use jetty_core::{
    connectors::{
        nodes::ConnectorData,
        nodes::{self as jetty_nodes, EffectivePermission, SparseMatrix},
        ConnectorClient, NewConnector,
    },
    cual::Cual,
    jetty::{ConnectorConfig, CredentialsMap},
    permissions::matrix::Merge,
    Connector,
};

use nodes::{asset_to_policy::env_to_jetty_policies, FromTableau};
use permissions::PermissionManager;

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

/// Map wrapper for config values.
pub type TableauConfig = HashMap<String, String>;

/// Credentials for authenticating with Tableau.
///
/// The user sets these up by following Jetty documentation
/// and pasting their connection info into their connector config.
#[derive(Deserialize, Debug, Default)]
pub struct TableauCredentials {
    username: String,
    password: String,
    /// Tableau server name like 10ay.online.tableau.com *without* the `https://`
    server_name: String,
    site_name: String,
}

impl TableauCredentials {
    /// Basic constructor.
    pub fn new(username: String, password: String, server_name: String, site_name: String) -> Self {
        Self {
            username,
            password,
            server_name,
            site_name,
        }
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

        (
            self.to_jetty(&self.coordinator.env.groups),
            self.to_jetty(&self.coordinator.env.users),
            all_assets,
            vec![], // self.object_to_jetty(&self.coordinator.env.tags);
            all_policies,
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
            coordinator: coordinator::Coordinator::new(creds, data_dir).await,
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
        let (groups, users, assets, tags, policies) = self.env_to_jetty_all();
        let effective_permissions = self.get_effective_permissions();
        ConnectorData::new(
            groups,
            users,
            assets,
            tags,
            policies,
            Default::default(),
            effective_permissions,
            Some(
                get_cual_prefix()
                    .context("tableau cual prefix not yet set")
                    .unwrap()
                    .to_owned(),
            ),
        )
    }
}
