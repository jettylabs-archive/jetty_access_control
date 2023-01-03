use std::collections::HashMap;

use crate::{access_graph::translate::Translator, cual::Cual, write};

#[derive(Debug)]
/// A group-specific local diff
pub struct LocalDiff {
    /// the asset being diffed
    pub asset: Cual,
    /// a map of users and user-specific changes for the asset
    pub users: HashMap<String, write::assets::diff::policies::DiffDetails>,
    /// a map of groups and group-specific changes for the asset
    pub groups: HashMap<String, write::assets::diff::policies::DiffDetails>,
}

impl Translator {
    pub(super) fn translate_policy_diff_to_local(
        &self,
        global_diff: &write::assets::diff::policies::PolicyDiff,
    ) -> LocalDiff {
        LocalDiff {
            asset: self.asset_name_to_cual(&global_diff.asset).unwrap(),
            users: global_diff
                .users
                .iter()
                .map(|(agent, details)| {
                    (
                        self.translate_node_name_to_local(agent, &global_diff.connector),
                        details.to_owned(),
                    )
                })
                .collect(),
            groups: global_diff
                .groups
                .iter()
                .map(|(agent, details)| {
                    (
                        self.translate_node_name_to_local(agent, &global_diff.connector),
                        details.to_owned(),
                    )
                })
                .collect(),
        }
    }
}
