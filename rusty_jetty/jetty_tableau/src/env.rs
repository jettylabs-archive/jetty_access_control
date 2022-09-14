use std::{collections::HashMap, fs, io};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::nodes2;
use crate::rest;
use crate::TableauCredentials;

#[derive(Default, Deserialize)]
struct TableauAssets {
    pub users: HashMap<String, nodes2::User>,
    pub groups: HashMap<String, nodes2::Group>,
    pub projects: HashMap<String, nodes2::Project>,
    pub datasources: HashMap<String, nodes2::Datasource>,
    pub data_connections: HashMap<String, nodes2::DataConnection>,
    pub flows: HashMap<String, nodes2::Flow>,
    pub lenses: HashMap<String, nodes2::Lens>,
    pub metrics: HashMap<String, nodes2::Metric>,
    pub views: HashMap<String, nodes2::View>,
    pub workbooks: HashMap<String, nodes2::Workbook>,
}

#[derive(Default)]
pub(crate) struct Environment {
    assets: TableauAssets,
    pub(crate) rest_client: rest::TableauRestClient,
}

impl Environment {
    /// Create a new Environment object with data read from a saved
    /// environment (if available) and a new rest client.
    pub(crate) fn new(creds: TableauCredentials) -> Self {
        Environment {
            assets: read_environment_assets().unwrap_or_default(),
            rest_client: rest::TableauRestClient::new(creds),
        }
    }
}

/// Read and parse the saved Tableau environment asset information
fn read_environment_assets() -> Result<TableauAssets> {
    // Open the file in read-only mode with buffer.
    let file = fs::File::open("tableau_env.json").context("opening environment file")?;
    let reader = io::BufReader::new(file);

    let e = serde_json::from_reader(reader).context("parsing environment")?;

    // Return the `Environment`.
    Ok(e)
}
