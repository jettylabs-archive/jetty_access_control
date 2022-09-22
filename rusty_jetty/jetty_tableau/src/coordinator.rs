use std::collections::HashSet;
use std::ops::IndexMut;
use std::pin::Pin;
use std::{collections::HashMap, fs, io};

use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::join;
use futures::StreamExt;
use serde::{Deserialize, Serialize};

use crate::nodes::{self, Permissionable};
use crate::rest;
use crate::TableauCredentials;

const CONCURRENT_ASSET_DOWNLOADS: usize = 25;
const CONCURRENT_METADATA_FETCHES: usize = 100;
const SERIALIZED_ENV_PATH: &str = "tableau_env.json";

#[derive(Default, Deserialize, Serialize, Debug)]
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

#[async_trait]
pub(crate) trait HasSources {
    fn id(&self) -> &String;
    fn name(&self) -> &String;
    fn updated_at(&self) -> &String;
    fn sources(&self) -> (HashSet<String>, HashSet<String>);
    async fn fetch_sources(
        &self,
        coord: &Coordinator,
    ) -> Result<(HashSet<String>, HashSet<String>)>;
    fn set_sources(&mut self, sources: (HashSet<String>, HashSet<String>));
}
#[derive(Default)]
pub(crate) struct Coordinator {
    pub(crate) env: Environment,
    pub(crate) rest_client: rest::TableauRestClient,
}

impl Coordinator {
    /// Create a new Environment object with data read from a saved
    /// environment (if available) and a new rest client.
    pub(crate) async fn new(creds: TableauCredentials) -> Self {
        Coordinator {
            env: read_environment_assets().unwrap_or_default(),
            rest_client: rest::TableauRestClient::new(creds).await,
        }
    }

    #[cfg(test)]
    pub(crate) fn new_dummy() -> Self {
        Coordinator {
            env: read_environment_assets().unwrap_or_default(),
            rest_client: rest::TableauRestClient::new_dummy(),
        }
    }

    #[allow(dead_code)]
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
                println!("unable to fetch users: {}", e);
                Default::default()
            }),
            projects: resources.1.unwrap_or_else(|e| {
                println!("unable to fetch projects: {}", e);
                Default::default()
            }),
            datasources: resources.0.unwrap_or_else(|e| {
                println!("unable to fetch datasources: {}", e);
                Default::default()
            }),
            flows: resources.8.unwrap_or_else(|e| {
                println!("unable to fetch flows: {}", e);
                Default::default()
            }),
            lenses: resources.7.unwrap_or_else(|e| {
                println!("unable to fetch lenses: {}", e);
                Default::default()
            }),
            metrics: resources.6.unwrap_or_else(|e| {
                println!("unable to fetch metrics: {}", e);
                Default::default()
            }),
            views: resources.3.unwrap_or_else(|e| {
                println!("unable to fetch views: {}", e);
                Default::default()
            }),
            workbooks: resources.2.unwrap_or_else(|e| {
                println!("unable to fetch workbooks: {}", e);
                Default::default()
            }),
            groups: resources.5.unwrap_or_else(|e| {
                println!("unable to fetch groups: {}", e);
                Default::default()
            }),
        };

        // Now, make sure that assets sources are all up to date

        self.update_sources_from_map(&mut new_env.flows, &self.env.flows)
            .await;
        self.update_sources_from_map(&mut new_env.datasources, &self.env.datasources)
            .await;
        self.update_sources_from_map(&mut new_env.workbooks, &self.env.workbooks)
            .await;

        // Now update permissions
        let x = vec![
            self.get_permission_futures_from_map(&mut new_env.datasources),
            self.get_permission_futures_from_map(&mut new_env.flows),
            self.get_permission_futures_from_map(&mut new_env.lenses),
            self.get_permission_futures_from_map(&mut new_env.metrics),
            self.get_permission_futures_from_map(&mut new_env.projects),
            self.get_permission_futures_from_map(&mut new_env.views),
            self.get_permission_futures_from_map(&mut new_env.workbooks),
        ];

        let fetches = futures::stream::iter(x.into_iter().flatten())
            .buffer_unordered(CONCURRENT_METADATA_FETCHES)
            .collect::<Vec<_>>()
            .await;

        // get group membership
        self.get_groups_users(&mut new_env.groups).await;

        // update self.env
        self.env = new_env;

        // serialize as JSON
        fs::write(
            SERIALIZED_ENV_PATH,
            serde_json::to_string_pretty(&self.env).unwrap(),
        )?;

        Ok(())
    }

    pub(crate) async fn get_groups_users(&self, groups: &mut HashMap<String, nodes::Group>) {
        let fetches = futures::stream::iter(
            groups
                .iter_mut()
                .map(|(_, v)| v.get_users(&self.rest_client)),
        )
        .buffer_unordered(CONCURRENT_METADATA_FETCHES)
        .collect::<Vec<_>>()
        .await;
    }

    fn update_sources<T: HasSources>(
        env_assets: &mut Vec<&mut T>,
        new_sources: Vec<Result<(HashSet<String>, HashSet<String>)>>,
    ) {
        for i in 0..env_assets.len() {
            match &new_sources[i] {
                Ok(sources) => {
                    env_assets.index_mut(i).set_sources(sources.to_owned());
                }
                Err(e) => println!(
                    "unable to get sources for datasource {}: {}",
                    env_assets[i].name(),
                    e
                ),
            }
        }
    }

    async fn update_sources_from_map<T: HasSources>(
        &self,
        new_assets: &mut HashMap<String, T>,
        old_assets: &HashMap<String, T>,
    ) {
        let mut assets_vec = new_assets.values_mut().collect::<Vec<_>>();

        let fetches = futures::stream::iter(
            assets_vec
                .iter_mut()
                .map(|d| self.get_sources(&old_assets, d)),
        )
        .buffered(CONCURRENT_ASSET_DOWNLOADS)
        .collect::<Vec<_>>();

        let asset_sources = fetches.await;

        Self::update_sources(&mut assets_vec, asset_sources);
    }

    fn get_permission_futures_from_map<'a, T: Permissionable + Send>(
        &'a self,
        new_assets: &'a mut HashMap<String, T>,
    ) -> Vec<
        Pin<
            Box<
                dyn futures::Future<Output = std::result::Result<(), anyhow::Error>>
                    + std::marker::Send
                    + '_,
            >,
        >,
    > {
        let fetches: Vec<_> = new_assets
            .iter_mut()
            .map(|(_, v)| v.update_permissions(&self.rest_client))
            .collect();
        fetches
    }

    async fn get_sources<T: HasSources>(
        &self,
        env_assets: &HashMap<String, T>,
        new_asset: &T,
    ) -> Result<(HashSet<String>, HashSet<String>)> {
        match env_assets.get(new_asset.id()) {
            Some(old_asset) if old_asset.updated_at() == new_asset.updated_at() => Ok((
                old_asset.sources().0.to_owned(),
                old_asset.sources().0.to_owned(),
            )),
            _ => new_asset.fetch_sources(&self).await,
        }
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

        tc.coordinator.update_env().await;

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
