//! Write user-configured groups and permissions back to the data stack.

pub mod assets;
pub mod config;
pub mod diff;
pub mod groups;
mod parser_common;
pub(crate) mod tag_parser;
pub mod users;
mod utils;

use anyhow::Result;

use std::collections::{HashMap, HashSet};

use crate::{jetty::ConnectorNamespace, Jetty};

use self::assets::diff::{default_policies::DefaultPolicyDiff, policies::PolicyDiff};

/// A collection of diffs to be sent to the connectors
pub struct GlobalDiffs {
    /// All the group-level diffs
    pub groups: Vec<groups::Diff>,
    /// All the user-levelgroup membership diffs
    pub users: Vec<users::CombinedUserDiff>,
    /// All the connector-managed default policies
    pub default_policies: Vec<DefaultPolicyDiff>,
    /// All the policies
    pub policies: Vec<PolicyDiff>,
}

impl GlobalDiffs {
    /// Split diffs into a HashMap of diffs, by connector
    pub fn split_by_connector(&self) -> HashMap<ConnectorNamespace, GlobalDiffs> {
        let user_map = split_diff_vec_by_connector(&self.users);
        let group_map = split_diff_vec_by_connector(&self.groups);
        let policy_map = split_diff_vec_by_connector(&self.policies);
        let default_policy_map = split_diff_vec_by_connector(&self.default_policies);

        let mut connectors: HashSet<_> = user_map.keys().collect();
        connectors.extend(group_map.keys());
        connectors.extend(policy_map.keys());
        connectors.extend(default_policy_map.keys());

        let mut res = HashMap::new();
        for conn in connectors {
            res.insert(
                conn.to_owned(),
                GlobalDiffs {
                    groups: group_map.get(conn).cloned().unwrap_or_default(),
                    users: user_map.get(conn).cloned().unwrap_or_default(),
                    policies: policy_map.get(conn).cloned().unwrap_or_default(),
                    default_policies: default_policy_map.get(conn).cloned().unwrap_or_default(),
                },
            );
        }
        res
    }
}

fn split_diff_vec_by_connector<T: Clone + SplitByConnector>(
    obj: &[T],
) -> HashMap<ConnectorNamespace, Vec<T>> {
    let mut res: HashMap<ConnectorNamespace, Vec<T>> = HashMap::new();
    for u in obj.iter() {
        res = u
            .split_by_connector()
            .iter()
            .fold(res, |mut acc, (conn, diff)| {
                acc.entry(conn.to_owned())
                    .or_insert_with(Default::default)
                    .push(*diff.to_owned());
                acc
            })
    }
    res
}

trait SplitByConnector {
    fn split_by_connector(&self) -> HashMap<ConnectorNamespace, Box<Self>>;
}

trait UpdateConfig {
    fn update_user_name(&mut self, old: &str, new: &str) -> Result<bool>;
    fn remove_user_name(&mut self, name: &str) -> Result<bool>;
    fn update_group_name(&mut self, old: &str, new: &str) -> Result<bool>;
    fn remove_group_name(&mut self, name: &str) -> Result<bool>;
}

/// update configuration files for the relvant node types
pub fn remove_group_name(jetty: &Jetty, name: &str) -> Result<()> {
    users::remove_group_name(jetty, name)?;
    groups::remove_group_name(jetty, name)?;
    assets::remove_group_name(jetty, name)?;
    Ok(())
}

/// update configuration files for the relvant node types
pub fn remove_user_name(jetty: &Jetty, name: &str) -> Result<()> {
    users::remove_user_name(jetty, name)?;
    groups::remove_user_name(jetty, name)?;
    assets::remove_user_name(jetty, name)?;
    Ok(())
}

/// update configuration files for the relvant node types
pub fn update_group_name(jetty: &Jetty, old: &str, new: &str) -> Result<()> {
    users::update_group_name(jetty, old, new)?;
    groups::update_group_name(jetty, old, new)?;
    assets::update_group_name(jetty, old, new)?;
    Ok(())
}

/// update configuration files for the relvant node types
pub fn update_user_name(jetty: &Jetty, old: &str, new: &str) -> Result<()> {
    users::update_user_name(jetty, old, new)?;
    groups::update_user_name(jetty, old, new)?;
    assets::update_user_name(jetty, old, new)?;
    Ok(())
}
