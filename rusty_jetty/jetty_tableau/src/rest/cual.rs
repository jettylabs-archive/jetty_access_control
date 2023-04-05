use std::{fmt::Display, sync::Once};

use anyhow::{bail, Context, Ok, Result};

use jetty_core::{cual::Cual, logging::error};
use serde::{Deserialize, Serialize};

use crate::{coordinator::Environment, nodes::ProjectId};

static mut CUAL_PREFIX: String = String::new();
static INIT_CUAL_PREFIX: Once = Once::new();

#[derive(PartialEq, Eq, Hash, Clone, Debug, PartialOrd, Ord, Deserialize, Serialize, Default)]
pub(crate) enum TableauAssetType {
    #[default]
    Project,
    Datasource,
    Flow,
    Workbook,
    Lens,
    Metric,
    View,
}

impl Display for TableauAssetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TableauAssetType {
    /// Used for cual construction, the str representation of
    /// the asset type helps identify the asset within Tableau.
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            TableauAssetType::Project => "project",
            TableauAssetType::Datasource => "datasource",
            TableauAssetType::Flow => "flow",
            TableauAssetType::Workbook => "workbook",
            TableauAssetType::Lens => "lens",
            TableauAssetType::Metric => "metric",
            TableauAssetType::View => "view",
        }
    }

    /// Get an asset type from str
    pub(crate) fn from_str(s: &str) -> Result<TableauAssetType> {
        match s {
            "project" => Ok(TableauAssetType::Project),
            "datasource" => Ok(TableauAssetType::Datasource),
            "flow" => Ok(TableauAssetType::Flow),
            "workbook" => Ok(TableauAssetType::Workbook),
            "lens" => Ok(TableauAssetType::Lens),
            "metric" => Ok(TableauAssetType::Metric),
            "view" => Ok(TableauAssetType::View),
            _ => bail!("invalid asset type: {}", s),
        }
    }

    /// At times we need to compose a URL, so the category helps give us the right
    /// url information
    pub(crate) fn as_category_str(&self) -> &'static str {
        match self {
            TableauAssetType::Project => "projects",
            TableauAssetType::Datasource => "datasources",
            TableauAssetType::Flow => "flows",
            TableauAssetType::Workbook => "workbooks",
            TableauAssetType::Lens => "lenses",
            TableauAssetType::Metric => "metrics",
            TableauAssetType::View => "views",
        }
    }
}

pub(crate) fn get_tableau_cual(
    asset_type: TableauAssetType,
    name: &str,
    parent_project_id: Option<&ProjectId>,
    // Direct parent ID for views, metrics, and lenses.
    immediate_parent_id: Option<&str>,
    env: &Environment,
) -> Result<Cual> {
    if let Some(ppid) = parent_project_id {
        let mut parents = env
            .get_recursive_projects_for(ppid)
            .into_iter()
            // Recursive projects are given starting with the immediate parent.
            // Reversing them will give us the top parent first.
            .rev()
            .map(|name| urlencoding::encode(&name).into_owned())
            .collect::<Vec<_>>();
        // most assets are children of projects, but Views, Metrics, and Lenses are special.
        let parent_path = match asset_type {
            TableauAssetType::View => {
                // views are children of workbooks
                let parent_workbook = env
                    .workbooks
                    .get(
                        &immediate_parent_id
                            .expect("getting parent workbook for view")
                            .to_owned(),
                    ).map(|w| w.name.clone())
                    .unwrap_or_else(|| {
                        error!(
                            "Getting parent workbook from env; name: {} parent: {:?}. Will use \"Unknown\" instead",
                            &name, &immediate_parent_id
                        );
                        "Unknown".to_string()
                        }
                    );
                parents.push(urlencoding::encode(&parent_workbook).into_owned());
                parents.join("/")
            }
            TableauAssetType::Metric => {
                // metrics are children of views
                let parent_view = env
                    .views
                    .get(
                        &immediate_parent_id
                            .expect("getting parent view for metric")
                            .to_owned(),
                    )
                    .unwrap_or_else(|| {
                        panic!("Getting parent view {immediate_parent_id:#?} for metric {name:#?}")
                    });
                let grandparent_workbook = env
                    .workbooks
                    .get(&parent_view.workbook_id)
                    .expect("getting grandparent workbook for metric");

                let mut direct_parents = vec![
                    urlencoding::encode(&grandparent_workbook.name).into_owned(),
                    urlencoding::encode(&parent_view.name).into_owned(),
                ];
                parents.append(&mut direct_parents);
                parents.join("/")
            }
            TableauAssetType::Lens => {
                // lenses are children of datasources
                let parent_ds = env
                    .datasources
                    .get(
                        &immediate_parent_id
                            .expect("getting parent id for lens")
                            .to_owned(),
                    )
                    .expect("Getting parent datasource for lens")
                    .name
                    .clone();
                parents.push(urlencoding::encode(&parent_ds).into_owned());
                parents.join("/")
            }
            TableauAssetType::Workbook
            | TableauAssetType::Project
            | TableauAssetType::Datasource
            | TableauAssetType::Flow => parents.join("/"),
        };
        Ok(Cual::new(&format!(
            "{}/{}/{}?type={}",
            get_cual_prefix()?,
            parent_path,
            urlencoding::encode(name),
            asset_type
        )))
        .context("Getting tableau CUAL")
    } else {
        // An asset without a parent is inferred to be a top-level project.
        Ok(Cual::new(&format!(
            "{}/{}?type={}",
            get_cual_prefix()?,
            urlencoding::encode(name),
            asset_type
        )))
        .context("Getting tableau CUAL")
    }
}

// Accessing a `static mut` is unsafe much of the time, but if we do so
// in a synchronized fashion (e.g., write once or read all) then we're
// good to go!
//
// This function will only set the string once, and will
// otherwise always effectively be a no-op.
pub(crate) fn set_cual_prefix(server_name: &str, site_name: &str) {
    unsafe {
        INIT_CUAL_PREFIX.call_once(|| {
            CUAL_PREFIX = format!("tableau://{}@{}", &server_name, &site_name);
        });
    }
}

pub(crate) fn get_cual_prefix<'a>() -> Result<&'a str> {
    if INIT_CUAL_PREFIX.is_completed() {
        // CUAL_PREFIX is set by a Once and is safe to use after initialization.
        unsafe { Ok(&CUAL_PREFIX) }
    } else {
        bail!("cual prefix was not yet set")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use crate::nodes::{Project, View, Workbook};

    use super::*;

    #[test]
    fn tableau_cual_works() -> Result<()> {
        set_cual_prefix("dummy-server", "dummy-site");
        let mut env = Environment::default();
        env.projects = HashMap::from([
            (
                "id1".to_owned(),
                Project {
                    name: "name1".to_owned(),
                    parent_project_id: Some(ProjectId("id2".to_owned())),
                    ..Default::default()
                },
            ),
            (
                "id2".to_owned(),
                Project {
                    name: "name2".to_owned(),
                    parent_project_id: None,
                    ..Default::default()
                },
            ),
        ]);
        let cual = get_tableau_cual(
            TableauAssetType::Flow,
            "my_flow_yo",
            Some(&ProjectId("id1".to_owned())),
            None,
            &env,
        )?;
        assert_eq!(
            cual,
            Cual::new("tableau://dummy-server@dummy-site/name2/name1/my_flow_yo?type=flow")
        );
        Ok(())
    }

    #[test]
    fn tableau_cual_works_with_no_parent() -> Result<()> {
        set_cual_prefix("dummy-server", "dummy-site");
        let env = Environment::default();
        let cual = get_tableau_cual(
            TableauAssetType::Project,
            "grandpappy_project",
            None,
            None,
            &env,
        )?;
        assert_eq!(
            cual,
            Cual::new("tableau://dummy-server@dummy-site/grandpappy_project?type=project")
        );
        Ok(())
    }

    #[test]
    fn metric_tableau_cual_works() -> Result<()> {
        set_cual_prefix("dummy-server", "dummy-site");
        let env = Environment {
            projects: HashMap::from([
                (
                    "project".to_owned(),
                    Project::new(
                        ProjectId("project".to_owned()),
                        "projecty".to_owned(),
                        "owner".to_owned(),
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
                        "projecta wojecta".to_owned(),
                        "owner".to_owned(),
                        None,
                        None,
                        vec![],
                        Default::default(),
                        Default::default(),
                    ),
                ),
            ]),
            views: HashMap::from([(
                "view".to_owned(),
                View::new(
                    "view".to_owned(),
                    "room with a view".to_owned(),
                    "wb".to_owned(),
                    String::new(),
                    ProjectId(String::new()),
                    String::new(),
                    vec![],
                ),
            )]),
            workbooks: HashMap::from([(
                "wb".to_owned(),
                Workbook::new(
                    "wb".to_owned(),
                    "book work".to_owned(),
                    String::new(),
                    ProjectId("project".to_owned()),
                    HashSet::new(),
                    String::new(),
                    vec![],
                ),
            )]),
            ..Default::default()
        };
        let cual = get_tableau_cual(
            TableauAssetType::Metric,
            "metric station",
            Some(&ProjectId("project".to_owned())),
            Some("view"),
            &env,
        )?;
        assert_eq!(
            cual,
            Cual::new("tableau://dummy-server@dummy-site/projecta%20wojecta/projecty/book%20work/room%20with%20a%20view/metric%20station?type=metric")
        );
        Ok(())
    }
}
