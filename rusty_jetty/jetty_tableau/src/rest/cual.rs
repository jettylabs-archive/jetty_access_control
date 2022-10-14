use std::sync::Once;

use anyhow::{bail, Context, Ok, Result};

use jetty_core::cual::Cual;

use crate::{coordinator::Environment, nodes::ProjectId};

static mut CUAL_PREFIX: String = String::new();
static INIT_CUAL_PREFIX: Once = Once::new();

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub(crate) enum TableauAssetType {
    Project,
    Datasource,
    Flow,
    Workbook,
    Lens,
    Metric,
    View,
}

impl TableauAssetType {
    /// Used for cual construction, the str representation of
    /// the asset type helps identify the asset within Tableau.
    fn as_str(&self) -> &'static str {
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
}

pub(crate) fn get_tableau_cual(
    asset_type: TableauAssetType,
    name: &str,
    parent_project_id: Option<&ProjectId>,
    env: &Environment,
) -> Result<Cual> {
    if let Some(ppid) = parent_project_id {
        let projects = env
            .get_recursive_projects_for(ppid)
            .into_iter()
            // Recursive projects are given starting with the immediate parent.
            // Reversing them will give us the top parent first.
            .rev()
            .map(|pid| {
                let ProjectId(id) = pid;
                id
            })
            .collect::<Vec<_>>();
        let project_path = projects.join("/");
        Ok(Cual::new(format!(
            "{}/{}/{}",
            get_cual_prefix()?,
            project_path,
            name
        )))
        .context("Getting tableau CUAL")
    } else {
        Ok(Cual::new(format!("{}/{}", get_cual_prefix()?, name))).context("Getting tableau CUAL")
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
            // TODO: figure out if there's a more sensible separator here
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
    use std::collections::HashMap;

    use crate::nodes::Project;

    use super::*;

    #[test]
    fn tableau_cual_works() -> Result<()> {
        set_cual_prefix("server", "site");
        let mut env = Environment::default();
        env.projects = HashMap::from([
            (
                "id1".to_owned(),
                Project {
                    parent_project_id: Some(ProjectId("id2".to_owned())),
                    ..Default::default()
                },
            ),
            (
                "id2".to_owned(),
                Project {
                    parent_project_id: None,
                    ..Default::default()
                },
            ),
        ]);
        let cual = get_tableau_cual(
            TableauAssetType::Flow,
            "my_flow_yo",
            Some(&ProjectId("id1".to_owned())),
            &env,
        )?;
        assert_eq!(
            cual,
            Cual::new("tableau://server@site/id2/id1/my_flow_yo".to_owned())
        );
        Ok(())
    }

    #[test]
    fn tableau_cual_works_with_no_parent() -> Result<()> {
        set_cual_prefix("server", "site");
        let env = Environment::default();
        let cual = get_tableau_cual(TableauAssetType::Project, "grandpappy_project", None, &env)?;
        assert_eq!(
            cual,
            Cual::new("tableau://server@site/grandpappy_project".to_owned())
        );
        Ok(())
    }
}
