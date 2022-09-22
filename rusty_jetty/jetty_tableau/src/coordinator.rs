use std::collections::HashSet;
use std::ops::IndexMut;
use std::{collections::HashMap, fs, io};

use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::join;
use futures::StreamExt;
use serde::Deserialize;

use crate::nodes::{self, Permissionable};
use crate::rest;
use crate::TableauCredentials;

#[derive(Default, Deserialize, Debug)]
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
        // Datasources
        let mut datasources_vec = new_env.datasources.values_mut().collect::<Vec<_>>();

        let fetches = futures::stream::iter(
            datasources_vec
                .iter_mut()
                .map(|d| self.get_sources(&self.env.datasources, d)),
        )
        .buffered(30)
        .collect::<Vec<_>>();

        let datasource_sources = fetches.await;

        Self::update_sources(&mut datasources_vec, datasource_sources);

        // Workbooks
        let mut workbooks_vec = new_env.workbooks.values_mut().collect::<Vec<_>>();

        let fetches = futures::stream::iter(
            workbooks_vec
                .iter_mut()
                .map(|d| self.get_sources(&self.env.workbooks, d)),
        )
        .buffered(30)
        .collect::<Vec<_>>();

        let workbook_sources = fetches.await;

        Self::update_sources(&mut workbooks_vec, workbook_sources);

        // Flows
        let mut flows_vec = new_env.flows.values_mut().collect::<Vec<_>>();

        let fetches = futures::stream::iter(
            flows_vec
                .iter_mut()
                .map(|d| self.get_sources(&self.env.flows, d)),
        )
        .buffered(30)
        .collect::<Vec<_>>();

        let flow_sources = fetches.await;

        Self::update_sources(&mut flows_vec, flow_sources);

        // update permissions
        let fetches = futures::stream::iter(
            new_env
                .datasources
                .iter_mut()
                .map(|(_, v)| v.update_permissions(&self.rest_client)),
        )
        .buffer_unordered(100)
        .collect::<Vec<_>>()
        .await;

        // update self.env
        // serialize as JSON

        Ok(())
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
    let file = fs::File::open("tableau_env.json").context("opening environment file")?;
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
        Ok(())
    }
}
