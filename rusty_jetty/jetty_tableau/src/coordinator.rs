use std::collections::HashSet;

use std::pin::Pin;
use std::{collections::HashMap, fs, io};

use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::join;
use futures::StreamExt;
use jetty_core::logging::error;
use serde::{Deserialize, Serialize};

use crate::nodes::{self, Permissionable, ProjectId};

use crate::rest;
use crate::TableauCredentials;

/// Number of assets to download concurrently
const CONCURRENT_ASSET_DOWNLOADS: usize = 25;
/// Number of metadata request to run currently (e.g. permissions)
const CONCURRENT_METADATA_FETCHES: usize = 100;
/// Path to serialized version of the Tableau Env
const SERIALIZED_ENV_PATH: &str = "tableau_env.json";

/// Conversion from Tableau types.
pub(crate) trait FromTableau<T> {
    fn from(val: T, env: &Environment) -> Self;
}

/// The state of a tableau site. We use this to persist state and
/// enable incremental updates.
#[derive(Default, Deserialize, Serialize, Debug, Clone)]
pub(crate) struct Environment {
    pub users: HashMap<String, nodes::User>,
    pub groups: HashMap<String, nodes::Group>,
    pub projects: HashMap<String, nodes::Project>,
    pub datasources: HashMap<String, nodes::Datasource>,
    pub flows: HashMap<String, nodes::Flow>,
    pub lenses: HashMap<String, nodes::Lens>,
    pub metrics: HashMap<String, nodes::Metric>,
    pub views: HashMap<String, nodes::View>,
    pub workbooks: HashMap<String, nodes::Workbook>,
}

impl Environment {
    pub(crate) fn get_recursive_projects_for(&self, project_id: &ProjectId) -> Vec<ProjectId> {
        let ProjectId(id) = project_id;
        let mut res = vec![project_id.clone()];
        if let Some(ppid) = self
            .projects
            .get(id)
            .and_then(|proj| proj.parent_project_id.clone())
        {
            res.append(&mut self.get_recursive_projects_for(&ppid))
        }
        res
    }
}

/// Implemented for asset types that have sources embedded in them: Workbooks, Flows, and Datasources
/// Makes it simpler to download these sources
#[async_trait]
pub(crate) trait HasSources {
    /// Get id of asset
    fn id(&self) -> &String;
    /// Get name of asset
    fn name(&self) -> &String;
    /// Get updated_at
    fn updated_at(&self) -> &String;
    /// Get sources
    fn sources(&self) -> (HashSet<String>, HashSet<String>);
    /// Fetch sources for an asset
    async fn fetch_sources(
        &self,
        coord: &Coordinator,
    ) -> Result<(HashSet<String>, HashSet<String>)>;
    fn set_sources(&mut self, sources: (HashSet<String>, HashSet<String>));

    /// Update sources for an asset
    async fn update_sources<T: HasSources + Sync + Send>(
        &mut self,
        coord: &Coordinator,
        env_assets: &HashMap<String, T>,
    ) -> Result<()> {
        let id = self.id().to_owned();
        match env_assets.get(&id) {
            Some(old_asset) if old_asset.updated_at() == self.updated_at() => {
                self.set_sources(old_asset.sources());
                anyhow::Ok(())
            }
            _ => {
                let x = self.fetch_sources(coord);
                self.set_sources(x.await?);
                Ok(())
            }
        }
    }
}

/// Coordinator handles manages and updates the connector's representation
/// of a Tableau instance
#[derive(Default)]
pub(crate) struct Coordinator {
    /// The current representation of the Tableau environment
    pub(crate) env: Environment,
    /// A client to access the Tableau environment
    pub(crate) rest_client: rest::TableauRestClient,
}

impl Coordinator {
    /// Create a new Coordinator object with data read from a saved
    /// environment (if available) and a new rest client.
    pub(crate) async fn new(creds: TableauCredentials) -> Self {
        Coordinator {
            env: read_environment_assets().unwrap_or_default(),
            rest_client: rest::TableauRestClient::new(creds).await,
        }
    }

    /// Create a dummy coordinator without a working rest client. The environment will be read from
    /// a file, if available, but cannot be updated.
    #[cfg(test)]
    pub(crate) fn new_dummy() -> Self {
        Coordinator {
            env: read_environment_assets().unwrap_or_default(),
            rest_client: rest::TableauRestClient::new_dummy(),
        }
    }

    /// Get current environment state from Tableau Online
    pub(crate) async fn update_env(&mut self) -> Result<()> {
        // Fetch all the basic resources. Make them into an iterable to make it easier to run concurrently
        let resources = join!(
            nodes::datasource::get_basic_datasources(&self.rest_client),
            nodes::project::get_basic_projects(&self.rest_client),
            nodes::workbook::get_basic_workbooks(&self.rest_client),
            nodes::view::get_basic_views(&self.rest_client),
            nodes::user::get_basic_users(&self.rest_client),
            nodes::group::get_basic_groups(&self.rest_client),
            nodes::metric::get_basic_metrics(&self.rest_client),
            nodes::lens::get_basic_lenses(&self.rest_client),
            nodes::flow::get_basic_flows(&self.rest_client),
        );

        let mut new_env = Environment {
            users: resources.4.unwrap_or_else(|e| {
                error!("unable to fetch users: {}", e);
                Default::default()
            }),
            projects: resources.1.unwrap_or_else(|e| {
                error!("unable to fetch projects: {}", e);
                Default::default()
            }),
            datasources: resources.0.unwrap_or_else(|e| {
                error!("unable to fetch datasources: {}", e);
                Default::default()
            }),
            flows: resources.8.unwrap_or_else(|e| {
                error!("unable to fetch flows: {}", e);
                Default::default()
            }),
            lenses: resources.7.unwrap_or_else(|e| {
                error!("unable to fetch lenses: {}", e);
                Default::default()
            }),
            metrics: resources.6.unwrap_or_else(|e| {
                error!("unable to fetch metrics: {}", e);
                Default::default()
            }),
            views: resources.3.unwrap_or_else(|e| {
                error!("unable to fetch views: {}", e);
                Default::default()
            }),
            workbooks: resources.2.unwrap_or_else(|e| {
                error!("unable to fetch workbooks: {}", e);
                Default::default()
            }),
            groups: resources.5.unwrap_or_else(|e| {
                error!("unable to fetch groups: {}", e);
                Default::default()
            }),
        };

        // Now, make sure that assets sources are all up to date

        let source_futures = vec![
            self.get_source_futures_from_map(&mut new_env.flows, &self.env.flows),
            self.get_source_futures_from_map(&mut new_env.datasources, &self.env.datasources),
            self.get_source_futures_from_map(&mut new_env.workbooks, &self.env.workbooks),
        ];

        // Source fetches
        futures::stream::iter(source_futures.into_iter().flatten())
            .buffer_unordered(CONCURRENT_ASSET_DOWNLOADS)
            .collect::<Vec<_>>()
            .await;

        // Clone the env so we don't try to both immutably and mutably borrow at the same time.
        let new_env_clone = new_env.clone();
        // Now update permissions. NOTE: This must happen AFTER getting groups and users.
        let permission_futures = vec![
            self.get_permission_futures_from_map(&mut new_env.datasources, &new_env_clone),
            self.get_permission_futures_from_map(&mut new_env.flows, &new_env_clone),
            self.get_permission_futures_from_map(&mut new_env.lenses, &new_env_clone),
            self.get_permission_futures_from_map(&mut new_env.metrics, &new_env_clone),
            self.get_permission_futures_from_map(&mut new_env.projects, &new_env_clone),
            self.get_permission_futures_from_map(&mut new_env.views, &new_env_clone),
            self.get_permission_futures_from_map(&mut new_env.workbooks, &new_env_clone),
        ];

        // Permission fetches
        futures::stream::iter(permission_futures.into_iter().flatten())
            .buffer_unordered(CONCURRENT_METADATA_FETCHES)
            .collect::<Vec<_>>()
            .await;

        // get group membership
        self.get_groups_users(&mut new_env.groups, &new_env.users)
            .await;

        // update self.env
        self.env = new_env;

        // serialize as JSON
        fs::write(
            SERIALIZED_ENV_PATH,
            serde_json::to_string_pretty(&self.env).unwrap(),
        )?;

        Ok(())
    }

    /// Get all the users for a map of groups. Returns a Vec of results that
    /// can be checked to know if any group membership was not fetched successfully
    async fn get_groups_users(
        &self,
        groups: &mut HashMap<String, nodes::Group>,
        users: &HashMap<String, nodes::User>,
    ) -> Vec<Result<(), anyhow::Error>> {
        futures::stream::iter(
            groups
                .iter_mut()
                .map(|(_, v)| v.update_users(&self.rest_client, users)),
        )
        .buffer_unordered(CONCURRENT_METADATA_FETCHES)
        .collect::<Vec<_>>()
        .await
    }

    /// Return a Vec of futures (sort of - look at return type) that will fetch futures from
    /// a map of assets
    fn get_source_futures_from_map<'a, T: HasSources + Send + Sync>(
        &'a self,
        new_assets: &'a mut HashMap<String, T>,
        old_assets: &'a HashMap<String, T>,
    ) -> Vec<
        Pin<
            Box<
                dyn futures::Future<Output = std::result::Result<(), anyhow::Error>>
                    + std::marker::Send
                    + '_,
            >,
        >,
    > {
        let fetches = new_assets
            .values_mut()
            .map(|d| d.update_sources(self, old_assets))
            .collect::<Vec<_>>();
        fetches
    }

    /// Return a Vec of futures that will request permissions for a collection of assets
    fn get_permission_futures_from_map<'a, T: Permissionable + Send>(
        &'a self,
        new_assets: &'a mut HashMap<String, T>,
        env: &'a Environment,
    ) -> Vec<
        Pin<
            Box<
                dyn futures::Future<Output = std::result::Result<(), anyhow::Error>>
                    + std::marker::Send
                    + '_,
            >,
        >,
    > {
        let fetches = new_assets
            .values_mut()
            .map(|v| v.update_permissions(&self.rest_client, env))
            .collect::<Vec<_>>();
        fetches
    }
}

/// Read and parse the saved Tableau environment asset information
fn read_environment_assets() -> Result<Environment> {
    // Open the file in read-only mode with buffer.
    let file = fs::File::open(SERIALIZED_ENV_PATH).context("opening environment file")?;
    let reader = io::BufReader::new(file);

    let e = serde_json::from_reader(reader).context("parsing environment")?;

    // Return the `Environment`.
    Ok(e)
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_update_env() -> Result<()> {
        let mut tc = crate::connector_setup()
            .await
            .context("running tableau connector setup")?;

        tc.coordinator.update_env().await?;

        let total_assets = tc.coordinator.env.datasources.len()
            + tc.coordinator.env.flows.len()
            + tc.coordinator.env.groups.len()
            + tc.coordinator.env.lenses.len()
            + tc.coordinator.env.metrics.len()
            + tc.coordinator.env.projects.len()
            + tc.coordinator.env.users.len()
            + tc.coordinator.env.views.len()
            + tc.coordinator.env.workbooks.len();
        dbg!(total_assets);
        Ok(())
    }
}
