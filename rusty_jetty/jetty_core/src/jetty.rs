//! Jetty Module
//!
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::{collections::HashMap, fmt::Display};

use anyhow::{anyhow, bail, Context, Result};

use log::debug;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use yaml_peg::serde as yaml;

use crate::access_graph::AccessGraph;
use crate::connectors::{AssetType, ConnectorCapabilities};
use crate::{project, Connector};

/// The user-defined namespace corresponding to the connector.
#[derive(Clone, Deserialize, Debug, Hash, PartialEq, Eq, Default, PartialOrd, Ord, Serialize)]
pub struct ConnectorNamespace(pub String);

impl Display for ConnectorNamespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Struct representing the jetty_config.yaml file.
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct JettyConfig {
    version: String,
    name: String,
    /// All connector configs defined.
    pub connectors: HashMap<ConnectorNamespace, ConnectorConfig>,
    /// Whether the user allows Jetty to collect usage data for analytics.
    #[serde(default = "default_allow_usage_stats")]
    pub allow_anonymous_usage_statistics: bool,
    /// The project id used for telemetry.
    #[serde(default = "new_project_id")]
    pub project_id: String,
}

/// Default to allow for anonymous usage statistics.
fn default_allow_usage_stats() -> bool {
    true
}

/// Create a new random project id. Should only ever be called once
/// per project.
pub fn new_project_id() -> String {
    Uuid::new_v4().to_string()
}

impl JettyConfig {
    /// New === default for this simple constructor.
    pub fn new() -> Self {
        Self {
            version: "0.0.1".to_owned(),
            allow_anonymous_usage_statistics: true,
            ..Default::default()
        }
    }

    /// Use the default filepath to ingest the Jetty config.
    pub fn read_from_file<P: AsRef<Path>>(path: P) -> Result<JettyConfig> {
        let config_raw = fs::read_to_string(&path).context("Reading file")?;
        let mut config =
            yaml::from_str::<JettyConfig>(&config_raw).context("Deserializing config")?;
        // Rewrite any newly created fields (project_id) to the config file.
        fs::write(
            path,
            yaml::to_string(&config[0]).context("Serializing config")?,
        )
        .context("Writing file back")?;

        config.pop().ok_or_else(|| anyhow!["failed"])
    }

    /// Set the project name.
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// Get the name
    pub fn get_name(&self) -> String {
        self.name.to_owned()
    }

    /// Convert this config to a yaml string.
    pub fn to_yaml(&self) -> Result<String> {
        yaml::to_string(self).map_err(anyhow::Error::from)
    }
}

/// Config for all connectors in this project.
#[derive(Clone, Deserialize, Serialize, Default, Debug)]
pub struct ConnectorConfig {
    /// The connector type
    #[serde(rename = "type")]
    pub connector_type: String,
    /// Additional configuration, specific to the connector
    #[serde(flatten)]
    pub config: HashMap<String, serde_json::Value>,
}

impl ConnectorConfig {
    /// Basic constructor
    pub fn new(connector_type: String, config: HashMap<String, serde_json::Value>) -> Self {
        Self {
            connector_type,
            config,
        }
    }
}

#[derive(Default, Debug)]
/// A struct representing the built-in characteristics of a connector.
pub struct ConnectorManifest {
    /// The capabilities of the connector.
    pub capabilities: ConnectorCapabilities,
    /// The asset type/privilege pairs that are allowed for a connector
    pub asset_privileges: HashMap<AssetType, HashSet<String>>,
}

/// Alias for HashMap to hold credentials information.
pub type CredentialsMap = HashMap<String, String>;

/// Fetch the credentials from the Jetty connectors config.
pub fn fetch_credentials(path: PathBuf) -> Result<HashMap<String, CredentialsMap>> {
    debug!("Trying to read credentials from {:?}", path);
    let credentials_raw = fs::read_to_string(path)?;
    let mut config = yaml::from_str::<HashMap<String, CredentialsMap>>(&credentials_raw)?;

    config
        .pop()
        .ok_or_else(|| anyhow!["failed to generate credentials"])
}

/// Represents Jetty Core in its entirety.
pub struct Jetty {
    /// The main jetty_config.yaml
    pub config: JettyConfig,
    // connector_config: HashMap<String, ConnectorCredentials>,
    /// The directory where data (such as the materialized graph) should be stored
    _data_dir: PathBuf,
    /// The access graph, if it exists
    pub access_graph: Option<AccessGraph>,
    /// The connectors
    pub connectors: HashMap<ConnectorNamespace, Box<dyn Connector>>,
}

impl Jetty {
    /// Convenience method for struct creation. Uses the default location for
    /// config files.
    // FUTURE: Remove this - it's used in one place in a test.
    pub fn new<P: AsRef<Path>>(
        jetty_config_path: P,
        data_dir: PathBuf,
        connectors: HashMap<ConnectorNamespace, Box<dyn Connector>>,
    ) -> Result<Self> {
        let config =
            JettyConfig::read_from_file(jetty_config_path).context("Reading Jetty Config file")?;

        let ag = load_access_graph(".")?;

        Ok(Jetty {
            config,
            _data_dir: data_dir,
            access_graph: ag,
            connectors,
        })
    }

    /// Convenience method for struct creation. Uses the default location for
    /// config files.
    pub fn new_with_config(
        config: JettyConfig,
        data_dir: PathBuf,
        connectors: HashMap<ConnectorNamespace, Box<dyn Connector>>,
        load_existing_graph: bool,
    ) -> Result<Self> {
        let ag = if load_existing_graph {
            load_access_graph(".")?
        } else {
            None
        };
        Ok(Jetty {
            config,
            _data_dir: data_dir,
            access_graph: ag,
            connectors,
        })
    }

    /// Getter for a reference to the connector manifests.
    pub fn connector_manifests(&self) -> HashMap<ConnectorNamespace, ConnectorManifest> {
        self.connectors
            .iter()
            .map(|(n, c)| (n.to_owned(), c.get_manifest()))
            .collect()
    }

    /// Getter for a reference to the access graph. Returns an error if no access graph has been created
    pub fn try_access_graph(&self) -> Result<&AccessGraph> {
        self.access_graph.as_ref().ok_or_else(|| {
            anyhow!("unable to find an existing access graph; try running `jetty fetch`")
        })
    }

    /// Getter for a mutable reference to the access graph. Returns an error if no access graph has been created
    pub fn try_access_graph_mut(&mut self) -> Result<&mut AccessGraph> {
        self.access_graph.as_mut().ok_or_else(|| {
            anyhow!("unable to find an existing access graph; try running `jetty fetch`")
        })
    }

    /// return whether a given connector name exists in the config
    pub(crate) fn has_connector(&self, connector: &ConnectorNamespace) -> bool {
        self.connectors.contains_key(connector)
    }

    /// Return a double HashMap of Connector, Asset Type, HashSet<Privileges strings>
    pub fn get_asset_type_privileges(
        &self,
    ) -> HashMap<ConnectorNamespace, HashMap<AssetType, HashSet<String>>> {
        let manifests = self.connector_manifests();
        manifests
            .iter()
            .map(|(c, m)| {
                (c.to_owned(), {
                    m.asset_privileges
                        .iter()
                        .map(|(asset_type, privs)| (asset_type.to_owned(), privs.to_owned()))
                        .collect::<HashMap<_, _>>()
                })
            })
            .collect()
    }
}

/// Load access graph from a file
fn load_access_graph<P: AsRef<Path>>(path_prefix: P) -> Result<Option<AccessGraph>> {
    // try to load the graph
    match AccessGraph::deserialize_graph(
        PathBuf::from(path_prefix.as_ref())
            .join(project::data_dir())
            .join(project::graph_filename()),
    ) {
        Ok(mut ag) => {
            // add the tags to the graph
            let tags_path = project::tags_cfg_path(&path_prefix);
            if tags_path.exists() {
                debug!("Getting tags from config.");
                let tag_config = std::fs::read_to_string(&tags_path);
                match tag_config {
                    Ok(c) => {
                        ag.add_tags(&c)?;
                    }
                    Err(e) => {
                        bail!(
                            "found tags file, but was unable to read {:?}\nerror: {}",
                            tags_path,
                            e
                        )
                    }
                };
            } else {
                debug!("No tags file found. Skipping ingestion.")
            };
            Ok(Some(ag))
        }
        Err(_e) => Ok(None),
    }
}
