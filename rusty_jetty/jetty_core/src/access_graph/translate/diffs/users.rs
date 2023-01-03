use std::collections::HashSet;

use crate::{access_graph::translate::Translator, jetty::ConnectorNamespace, write};

#[derive(Debug)]
/// A group-specific local diff
pub struct LocalDiff {
    /// the group being diffed
    pub user: String,
    /// The specifics of the diff
    pub group_membership: LocalDiffDetails,
}

#[derive(Debug)]
/// Outlines the diff type needed
pub struct LocalDiffDetails {
    /// the groups that the user should be added as a member of
    pub add: HashSet<String>,
    /// the groups that the user should be removed as a member of
    pub remove: HashSet<String>,
}

impl Translator {
    pub(super) fn translate_user_diff_to_local(
        &self,
        global_diff: &write::users::CombinedUserDiff,
        connector: &ConnectorNamespace,
    ) -> Option<LocalDiff> {
        global_diff.group_membership.as_ref().map(|group_membership| LocalDiff {
                user: self.translate_node_name_to_local(&global_diff.user, connector),
                group_membership: LocalDiffDetails {
                    add: group_membership
                        .add
                        .iter()
                        .map(|group| self.translate_node_name_to_local(group, connector))
                        .collect(),
                    remove: group_membership
                        .remove
                        .iter()
                        .map(|group| self.translate_node_name_to_local(group, connector))
                        .collect(),
                },
            })
    }
}
