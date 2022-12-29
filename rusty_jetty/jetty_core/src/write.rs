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

/// A collection of diffs
pub struct Diffs {
    /// All the group-level diffs
    pub groups: Vec<groups::Diff>,
}

impl Diffs {
    /// Split diffs into a HashMap of diffs, by connector
    pub fn split_by_connector(&self) -> HashMap<ConnectorNamespace, Diffs> {
        // go through each field of Diffs, starting with groups
        let group_maps = self.groups.iter().fold(HashMap::new(), |mut acc, x| {
            acc.entry(&x.connector)
                .and_modify(|vec_diffs: &mut Vec<groups::Diff>| vec_diffs.push(x.to_owned()))
                .or_insert(vec![x.to_owned()]);
            acc
        });

        let mut res: HashMap<ConnectorNamespace, Diffs> = HashMap::new();

        // now put the groups into the proper fields
        for (conn, diffs) in group_maps {
            res.entry(conn.clone())
                .and_modify(|the_diff| the_diff.groups = diffs.clone())
                .or_insert(Diffs { groups: diffs });
        }

        res
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
