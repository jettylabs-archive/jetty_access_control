use std::collections::HashSet;

use std::path::PathBuf;
use std::pin::Pin;
use std::{collections::HashMap, fs, io};

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use futures::StreamExt;
use futures::{join, Future};
use jetty_core::cual::Cual;
use jetty_core::logging::{error, warn};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::nodes::{self, Permissionable, ProjectId, TableauCualable};

use crate::origin::SourceOrigin;
use crate::rest::{self, TableauAssetType};
use crate::TableauCredentials;

/// Number of metadata request to run currently (e.g. permissions)
pub(crate) const CONCURRENT_METADATA_FETCHES: usize = 50;
/// Path to serialized version of the Tableau Env
const SERIALIZED_ENV_FILENAME: &str = "tableau_env.json";

/// The state of a tableau site. We use this to persist state and
/// enable incremental updates.
#[serde_as]
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
    #[serde_as(as = "HashMap<serde_with::json::JsonString, _>")]
    pub cual_id_map: HashMap<Cual, TableauAssetReference>,
}

#[derive(Hash, Default, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub(crate) struct TableauAssetReference {
    pub(crate) asset_type: TableauAssetType,
    pub(crate) id: String,
}

impl Environment {
    pub(crate) fn get_recursive_projects_for(&self, project_id: &ProjectId) -> Vec<String> {
        let ProjectId(id) = project_id;
        let this_project = self.projects.get(id);
        let mut res = vec![this_project
            .expect("getting project from env")
            .name
            .to_owned()];
        if let Some(ppid) = this_project.and_then(|proj| proj.parent_project_id.clone()) {
            res.append(&mut self.get_recursive_projects_for(&ppid));
        }
        res
    }

    /// given a group name, return the group id
    pub(crate) fn get_group_id_by_name(&self, group_name: &String) -> Option<String> {
        self.groups.iter().find_map(|(id, g)| {
            if &g.name == group_name {
                Some(id.to_owned())
            } else {
                None
            }
        })
    }
}

/// Implemented for asset types that have sources embedded in them: Workbooks, Flows, and Datasources
/// Makes it simpler to download these sources
#[async_trait]
pub(crate) trait HasSources {
    fn set_sources(&mut self, sources: (HashSet<SourceOrigin>, HashSet<SourceOrigin>));
}

/// Coordinator handles manages and updates the connector's representation
/// of a Tableau instance
#[derive(Default)]
pub(crate) struct Coordinator {
    /// The current representation of the Tableau environment
    pub(crate) env: Environment,
    /// A client to access the Tableau environment
    pub(crate) rest_client: rest::TableauRestClient,
    /// Directory where connector_specific data can be stored
    pub(crate) data_dir: Option<PathBuf>,
}

impl Coordinator {
    /// Create a new Coordinator object with data read from a saved
    /// environment (if available) and a new rest client.
    pub(crate) async fn new(creds: TableauCredentials, data_dir: Option<PathBuf>) -> Result<Self> {
        let env = if let Some(dir) = data_dir.clone() {
            read_environment_assets(dir).unwrap_or_default()
        } else {
            Default::default()
        };

        // Create a rest client. This tests the authentication.
        let rest_client = rest::TableauRestClient::new(creds).await;
        let rest_client = match rest_client {
            Ok(c) => c,
            Err(e) => {
                if e.to_string()
                    .contains("The personal access token you provided is invalid.")
                {
                    bail!("The personal access token you provided for Tableau is invalid. Valid tokens expire after 15 days without use. Please update the token in ~/.jetty/connectors.yaml");
                } else if e.to_string().contains("Signin Error") {
                    bail!("Tableau was unable to sign in. Please check the credentials in ~/.jetty/connectors.yaml.");
                }
                bail!(e);
            }
        };

        Ok(Coordinator {
            env,
            rest_client,
            data_dir,
        })
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
            // FUTURE: update all calls to create a cual to just use this. Probably easier to have it centralized
            cual_id_map: Default::default(),
        };

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
        let fetch_results = futures::stream::iter(permission_futures.into_iter().flatten())
            .buffer_unordered(CONCURRENT_METADATA_FETCHES)
            .collect::<Vec<_>>()
            .await;

        fetch_results.into_iter().for_each(|r| match r {
            Ok(_) => (),
            Err(e) => error!("problem fetching tableau permissions: {e}"),
        });

        // Default permission fetches
        let fetch_results = futures::stream::iter(
            self.get_default_permission_futures_for_projects(&mut new_env.projects, &new_env_clone),
        )
        .buffer_unordered(CONCURRENT_METADATA_FETCHES)
        .collect::<Vec<_>>()
        .await;

        fetch_results.into_iter().for_each(|r| match r {
            Ok(_) => (),
            Err(e) => error!("problem fetching tableau default permissions: {e}"),
        });

        // get group membership
        let group_membership_futures =
            self.get_groups_users(&mut new_env.groups, &new_env_clone.users);

        let fetch_results = futures::stream::iter(group_membership_futures)
            .buffer_unordered(CONCURRENT_METADATA_FETCHES)
            .collect::<Vec<_>>()
            .await;

        fetch_results.into_iter().for_each(|r| match r {
            Ok(_) => (),
            Err(e) => error!("problem fetching tableau groups: {e}"),
        });

        // update the cual map
        build_cual_map(&mut new_env);
        // update self.env
        self.env = new_env;

        // update the lineage
        if let Err(e) = self.update_lineage().await {
            warn!("problem updating Tableau lineage: {e}");
        };

        // serialize as JSON
        if let Some(dir) = &self.data_dir {
            let file_path = dir.join(SERIALIZED_ENV_FILENAME);
            if let Some(p) = file_path.parent() {
                fs::create_dir_all(p)?
            };
            fs::write(file_path, serde_json::to_string_pretty(&self.env).unwrap())?;
        }
        Ok(())
    }

    /// Get all the users for a map of groups. Returns a Vec of results that
    /// can be checked to know if any group membership was not fetched successfully
    fn get_groups_users<'b>(
        &'b self,
        groups: &'b mut HashMap<String, nodes::Group>,
        users: &'b HashMap<String, nodes::User>,
    ) -> Vec<impl futures::Future<Output = Result<()>> + 'b> {
        let fetches = groups
            .values_mut()
            .map(|d| d.update_users(&self.rest_client, users))
            .collect::<Vec<_>>();
        fetches
    }

    /// Return a Vec of futures that will request permissions for a collection of assets
    #[allow(clippy::type_complexity)]
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

    /// Return a Vec of futures that will request permissions for a collection of assets
    fn get_default_permission_futures_for_projects<'a>(
        &'a self,
        projects: &'a mut HashMap<String, nodes::Project>,
        env: &'a Environment,
    ) -> Vec<impl Future<Output = Result<(), anyhow::Error>> + 'a> {
        let fetches = projects
            .values_mut()
            .map(|v| v.update_default_permissions(&self.rest_client, env))
            .collect::<Vec<_>>();
        fetches
    }
}

/// Read and parse the saved Tableau environment asset information
fn read_environment_assets(data_dir: PathBuf) -> Result<Environment> {
    // Open the file in read-only mode with buffer.
    let file = fs::File::open(data_dir.join(SERIALIZED_ENV_FILENAME))
        .context("opening environment file")?;
    let reader = io::BufReader::new(file);

    let e = serde_json::from_reader(reader).context("parsing environment")?;

    // Return the `Environment`.
    Ok(e)
}

fn build_cual_map(env: &mut Environment) {
    for (id, node) in &env.projects {
        env.cual_id_map.insert(
            node.cual(env),
            TableauAssetReference {
                asset_type: TableauAssetType::Project,
                id: id.to_owned(),
            },
        );
    }
    for (id, node) in &env.datasources {
        env.cual_id_map.insert(
            node.cual(env),
            TableauAssetReference {
                asset_type: TableauAssetType::Datasource,
                id: id.to_owned(),
            },
        );
    }
    for (id, node) in &env.flows {
        env.cual_id_map.insert(
            node.cual(env),
            TableauAssetReference {
                asset_type: TableauAssetType::Flow,
                id: id.to_owned(),
            },
        );
    }
    for (id, node) in &env.lenses {
        env.cual_id_map.insert(
            node.cual(env),
            TableauAssetReference {
                asset_type: TableauAssetType::Lens,
                id: id.to_owned(),
            },
        );
    }
    for (id, node) in &env.metrics {
        env.cual_id_map.insert(
            node.cual(env),
            TableauAssetReference {
                asset_type: TableauAssetType::Metric,
                id: id.to_owned(),
            },
        );
    }
    for (id, node) in &env.views {
        env.cual_id_map.insert(
            node.cual(env),
            TableauAssetReference {
                asset_type: TableauAssetType::View,
                id: id.to_owned(),
            },
        );
    }
    for (id, node) in &env.workbooks {
        env.cual_id_map.insert(
            node.cual(env),
            TableauAssetReference {
                asset_type: TableauAssetType::Workbook,
                id: id.to_owned(),
            },
        );
    }
}

#[cfg(test)]
mod test {
    use crate::nodes::Project;

    use super::*;

    #[test]
    fn test_get_recursive_projects_for_yields_correct_order() {
        let mut env = Environment::default();
        env.projects = HashMap::from([
            (
                "project".to_owned(),
                Project::new(
                    ProjectId("project".to_owned()),
                    "p0".to_owned(),
                    String::new(),
                    Some(ProjectId("project1".to_owned())),
                    None,
                    vec![],
                    Default::default(),
                    Default::default(),
                ),
            ),
            (
                "project1".to_owned(),
                Project::new(
                    ProjectId("project1".to_owned()),
                    "p1".to_owned(),
                    String::new(),
                    Some(ProjectId("project2".to_owned())),
                    None,
                    vec![],
                    Default::default(),
                    Default::default(),
                ),
            ),
            (
                "project2".to_owned(),
                Project::new(
                    ProjectId("project2".to_owned()),
                    "p2".to_owned(),
                    String::new(),
                    None,
                    None,
                    vec![],
                    Default::default(),
                    Default::default(),
                ),
            ),
        ]);
        let parents = env.get_recursive_projects_for(&ProjectId("project".to_owned()));
        assert_eq!(
            parents,
            vec!["p0".to_owned(), "p1".to_owned(), "p2".to_owned()]
        );
    }
}
