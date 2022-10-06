#![allow(dead_code)]

mod coordinator;
mod file_parse;
mod nodes;
mod permissions;
mod rest;

use anyhow::{anyhow, Result};
use async_trait::async_trait;

use rest::TableauRestClient;
use serde::Deserialize;
use serde_json::json;

use jetty_core::{
    connectors::{
        nodes::ConnectorData,
        nodes::{self as jetty_nodes, EffectivePermission, SparseMatrix},
        ConnectorClient, UserIdentifier,
    },
    cual::{Cual, Cualable},
    jetty::{ConnectorConfig, CredentialsBlob},
    Connector,
};

use nodes::asset_to_policy::env_to_jetty_policies;
use permissions::PermissionManager;

use std::collections::{HashMap, HashSet};

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

    fn get_effective_permissions(
        &self,
    ) -> SparseMatrix<UserIdentifier, Cual, HashSet<EffectivePermission>> {
        let permission_manager = PermissionManager::new(&self.coordinator);
        let mut final_eps = HashMap::new();
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

    let j = jetty_core::jetty::Jetty::new().context("creating Jetty")?;
    let creds = jetty_core::jetty::fetch_credentials().context("fetching credentials from file")?;
    let config = &j.config.connectors[0];
    let tc = crate::TableauConnector::new(config, &creds["tableau"], None)
        .await
        .context("reading tableau credentials")?;
    Ok(*tc)
}
