//! Write user-configured groups and permissions back to the data stack.

pub mod assets;
pub mod groups;
pub mod new_groups;
mod parser_common;
pub(crate) mod tag_parser;
pub mod users;
mod utils;

use anyhow::Result;

use std::collections::HashMap;

pub use groups::get_group_diff;

use crate::{jetty::ConnectorNamespace, Jetty};

use self::assets::diff::{default_policies::DefaultPolicyDiff, policies::PolicyDiff};

/// A collection of diffs to be sent to the connectors
pub struct GlobalConnectorDiffs {
    /// All the group-level diffs
    pub groups: Vec<new_groups::Diff>,
    /// All the user-levelgroup membership diffs
    pub users: Vec<users::CombinedUserDiff>,
    /// All the connector-managed default policies
    pub default_policies: Vec<DefaultPolicyDiff>,
    /// All the policies
    pub policies: Vec<PolicyDiff>,
}

impl GlobalConnectorDiffs {
    /// Split diffs into a HashMap of diffs, by connector
    pub fn split_by_connector(&self) -> HashMap<ConnectorNamespace, GlobalConnectorDiffs> {
        todo!()
    }
}

trait UpdateConfig {
    fn update_user_name(&mut self, old: &String, new: &str) -> Result<bool>;
    fn remove_user_name(&mut self, name: &String) -> Result<bool>;
    fn update_group_name(&mut self, old: &String, new: &str) -> Result<bool>;
    fn remove_group_name(&mut self, name: &String) -> Result<bool>;
}

/// update configuration files for the relvant node types
pub fn remove_group_name(jetty: &Jetty, name: &String) -> Result<()> {
    users::remove_group_name(jetty, name)?;
    new_groups::remove_group_name(jetty, name)?;
    assets::remove_group_name(jetty, name)?;
    Ok(())
}

/// update configuration files for the relvant node types
pub fn remove_user_name(jetty: &Jetty, name: &String) -> Result<()> {
    users::remove_user_name(jetty, name)?;
    new_groups::remove_user_name(jetty, name)?;
    assets::remove_user_name(jetty, name)?;
    Ok(())
}

/// update configuration files for the relvant node types
pub fn update_group_name(jetty: &Jetty, old: &String, new: &String) -> Result<()> {
    users::update_group_name(jetty, old, new)?;
    new_groups::update_group_name(jetty, old, new)?;
    assets::update_group_name(jetty, old, new)?;
    Ok(())
}

/// update configuration files for the relvant node types
pub fn update_user_name(jetty: &Jetty, old: &String, new: &String) -> Result<()> {
    users::update_user_name(jetty, old, new)?;
    new_groups::update_user_name(jetty, old, new)?;
    assets::update_user_name(jetty, old, new)?;
    Ok(())
}
