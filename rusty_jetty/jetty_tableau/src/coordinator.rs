use std::{
    collections::{HashMap, HashSet},
    fs, io,
};

use anyhow::{anyhow, Context, Result};
use futures::StreamExt;
use serde::Deserialize;

use crate::nodes::{self, datasource};
use crate::rest;
use crate::TableauCredentials;

#[derive(Default, Deserialize)]
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

#[derive(Default)]
pub(crate) struct Coordinator {
    pub(crate) env: Environment,
    pub(crate) rest_client: rest::TableauRestClient,
}

impl Coordinator {
    /// Create a new Environment object with data read from a saved
    /// environment (if available) and a new rest client.
    pub(crate) fn new(creds: TableauCredentials) -> Self {
        Coordinator {
            env: read_environment_assets().unwrap_or_default(),
            rest_client: rest::TableauRestClient::new(creds),
        }
    }

    pub(crate) async fn update_env(&mut self) -> Result<()> {
        let datasources = nodes::datasource::get_basic_datasources(&self.rest_client).await?;

        // Get workbook basics
        let mut workbooks = nodes::workbook::get_basic_workbooks(&self.rest_client).await?;

        // for each workbook, get the datasources
        let fetches = futures::stream::iter(
            workbooks
                .iter_mut()
                .map(|(id, w)| self.get_workbook_datasources(w)),
        )
        .buffer_unordered(30)
        .collect::<Vec<_>>();
        let datasource_vectors = fetches.await.into_iter().collect::<Result<Vec<_>>>()?;

        // update datasources with datasources from workbooks
        for v in datasource_vectors {
            for d in v {
                self.env.datasources.entry(d.id.to_owned()).or_insert(d);
            }
        }

        // now update datasources as needed

        todo!()
    }

    /// Get datasources for a single workbook by pulling from the saved environment or
    /// fetching from Tableau (if necessary). Returns an updated
    async fn get_workbook_datasources(
        &self,
        wbook: &mut nodes::Workbook,
    ) -> Result<Vec<nodes::Datasource>> {
        if let Some(datasources) = self.get_workbook_datasources_from_env(wbook) {
            wbook.tableau_datasources = datasources.iter().map(|d| d.id.to_owned()).collect();
            Ok(datasources)
        } else {
            let datasources = wbook.fetch_datasources().await?;
            wbook.tableau_datasources = datasources.iter().map(|d| d.id.to_owned()).collect();
            Ok(datasources)
        }
    }

    /// Get datasources for a single workbook from the saved environment, if appropriate,
    /// else, return None
    fn get_workbook_datasources_from_env(
        &self,
        wbook: &nodes::Workbook,
    ) -> Option<Vec<nodes::Datasource>> {
        // If the workbook exists in the env and hasn't been modified, just
        // use the datasources already defined. Otherwise, return None.
        if let Some(env_wbook) = self.env.workbooks.get(&wbook.id) {
            if env_wbook.updated_at != wbook.updated_at {
                None
            } else {
                env_wbook
                    .tableau_datasources
                    .iter()
                    .map(|id| {
                        if let Some(datasource) = self.env.datasources.get(id) {
                            Some(datasource.to_owned())
                        } else {
                            None
                        }
                    })
                    .collect::<Option<Vec<_>>>()
            }
        } else {
            None
        }
    }

    /// If we already have up-to-date datasource info saved, get that.
    fn get_datasource_from_env(&self, datasource: &nodes::Datasource) -> Option<nodes::Datasource> {
        todo!()
    }

    /// Get up-to-date datasource info
    async fn get_datasource_details(
        &self,
        datasource: nodes::Datasource,
    ) -> Result<nodes::Datasource> {
        todo!()
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
