use std::{collections::HashMap, fs, io};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::nodes;
use crate::rest;
use crate::TableauCredentials;

#[derive(Default, Deserialize)]
struct TableauAssets {
    pub users: HashMap<String, nodes::User>,
    pub groups: HashMap<String, nodes::Group>,
    pub projects: HashMap<String, nodes::Project>,
    pub datasources: HashMap<String, nodes::Datasource>,
    pub data_connections: HashMap<String, nodes::DataConnection>,
    pub flows: HashMap<String, nodes::Flow>,
    pub lenses: HashMap<String, nodes::Lens>,
    pub metrics: HashMap<String, nodes::Metric>,
    pub views: HashMap<String, nodes::View>,
    pub workbooks: HashMap<String, nodes::Workbook>,
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
