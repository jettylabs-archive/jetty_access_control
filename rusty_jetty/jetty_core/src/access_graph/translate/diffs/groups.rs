use crate::{
    access_graph::{translate::Translator, NodeName},
    jetty::ConnectorNamespace,
    write::groups,
};

use std::collections::{BTreeSet, HashSet};

#[derive(Debug)]
/// A group-specific local diff
pub struct LocalDiff {
    /// the group being diffed
    pub group_name: String,
    /// The specifics of the diff
    pub details: LocalDiffDetails,
}

#[derive(Debug)]
/// Outlines the diff type needed
pub enum LocalDiffDetails {
    /// Add a group
    AddGroup {
        /// the members of the group
        member_of: HashSet<String>,
    },
    /// Remove a group
    RemoveGroup,
    /// Update a group
    ModifyGroup {
        /// groups that are added as members
        add_member_of: HashSet<String>,
        /// groups that are removed as members
        remove_member_of: HashSet<String>,
    },
}

impl Translator {
    pub(super) fn translate_group_diff_to_local(&self, global_diff: &groups::Diff) -> LocalDiff {
        LocalDiff {
            group_name: self
                .translate_node_name_to_local(&global_diff.group_name, &global_diff.connector),
            details: match &global_diff.details {
                groups::diff::DiffDetails::AddGroup { member_of } => LocalDiffDetails::AddGroup {
                    member_of: self
                        .translate_group_member_changes_to_local(member_of, &global_diff.connector),
                },
                groups::diff::DiffDetails::RemoveGroup => LocalDiffDetails::RemoveGroup,
                groups::diff::DiffDetails::ModifyGroup {
                    add_member_of,
                    remove_member_of,
                } => LocalDiffDetails::ModifyGroup {
                    add_member_of: self.translate_group_member_changes_to_local(
                        add_member_of,
                        &global_diff.connector,
                    ),
                    remove_member_of: self.translate_group_member_changes_to_local(
                        remove_member_of,
                        &global_diff.connector,
                    ),
                },
            },
        }
    }

    fn translate_group_member_changes_to_local(
        &self,
        global_changes: &BTreeSet<NodeName>,
        connector: &ConnectorNamespace,
    ) -> HashSet<String> {
        global_changes
            .iter()
            .map(|group| self.translate_node_name_to_local(group, connector))
            .collect()
    }
}
