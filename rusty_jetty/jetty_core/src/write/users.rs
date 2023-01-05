//! Functionality to manage the write path for users

pub mod bootstrap;
pub mod diff;
pub mod parser;
mod update;

use std::collections::{BTreeSet, HashMap};

use anyhow::{Context, Result};
use glob::glob;
use serde::{Deserialize, Serialize};

use crate::{jetty::ConnectorNamespace, project};

pub use diff::{get_membership_diffs, CombinedUserDiff};
pub use parser::get_validated_file_config_map;
pub(crate) use update::{remove_group_name, remove_user_name, update_group_name, update_user_name};

use super::UpdateConfig;

/// Struct representing user configuration files
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserYaml {
    name: String,
    identifiers: HashMap<ConnectorNamespace, String>,
    #[serde(
        skip_serializing_if = "BTreeSet::is_empty",
        default,
        rename = "member of"
    )]
    member_of: BTreeSet<String>,
}

impl UpdateConfig for UserYaml {
    fn update_user_name(&mut self, old: &str, new: &str) -> Result<bool> {
        if self.name == old {
            self.name = new.to_owned();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// No-op: if the name in the config is a match, delete the config file (must happen in
    /// the caller).
    fn remove_user_name(&mut self, _name: &str) -> Result<bool> {
        Ok(true)
    }

    fn update_group_name(&mut self, old: &str, new: &str) -> Result<bool> {
        if self.member_of.remove(old) {
            self.member_of.insert(new.to_string());
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn remove_group_name(&mut self, name: &str) -> Result<bool> {
        Ok(self.member_of.remove(name))
    }
}

/// Get the paths of all asset config files
pub(crate) fn get_config_paths() -> Result<glob::Paths> {
    // collect the paths to all the config files
    glob(
        format!(
            // the user files can be in whatever directory the user would like
            "{}/**/*.y*ml",
            project::users_cfg_root_path_local().to_string_lossy()
        )
        .as_str(),
    )
    .context("trouble generating config file paths")
}
