//! Types and functionality to convert diffs to a state that can be processed by connectors

use crate::write::Diffs;

use super::Translator;

/// Diffs in the namespace of the connectors
pub struct LocalDiffs {
    /// The group-specific diffs
    pub groups: Vec<groups::LocalDiff>,
}

/// Group-specific diff functionality
pub mod groups {
    use crate::{
        access_graph::translate::Translator,
        jetty::ConnectorNamespace,
        write::groups::{self, GroupMemberChanges},
    };
    use anyhow::Result;
    use std::collections::HashSet;

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
            members: LocalGroupMemberChanges,
        },
        /// Remove a group
        RemoveGroup,
        /// Update a group
        ModifyGroup {
            /// members that are added
            add: LocalGroupMemberChanges,
            /// members that are removed
            remove: LocalGroupMemberChanges,
        },
    }

    /// The specific changes within a diff
    #[derive(Debug)]
    pub struct LocalGroupMemberChanges {
        /// users
        pub users: HashSet<String>,
        /// groups
        pub groups: HashSet<String>,
    }

    impl Translator {
        pub(super) fn translate_group_diff_to_local(
            &self,
            global_diff: &groups::Diff,
        ) -> LocalDiff {
            LocalDiff {
                group_name: self
                    .translate_node_name_to_local(&global_diff.group_name, &global_diff.connector),
                details: match &global_diff.details {
                    groups::DiffDetails::AddGroup { members } => LocalDiffDetails::AddGroup {
                        members: self.translate_group_member_changes_to_local(
                            &members,
                            &global_diff.connector,
                        ),
                    },
                    groups::DiffDetails::RemoveGroup => LocalDiffDetails::RemoveGroup,
                    groups::DiffDetails::ModifyGroup { add, remove } => {
                        LocalDiffDetails::ModifyGroup {
                            add: self.translate_group_member_changes_to_local(
                                &add,
                                &global_diff.connector,
                            ),
                            remove: self.translate_group_member_changes_to_local(
                                &remove,
                                &global_diff.connector,
                            ),
                        }
                    }
                },
            }
        }

        fn translate_group_member_changes_to_local(
            &self,
            global_changes: &GroupMemberChanges,
            connector: &ConnectorNamespace,
        ) -> LocalGroupMemberChanges {
            LocalGroupMemberChanges {
                users: global_changes
                    .users
                    .iter()
                    .map(|user| self.translate_node_name_to_local(&user, &connector))
                    .collect(),
                groups: global_changes
                    .groups
                    .iter()
                    .map(|user| self.translate_node_name_to_local(&user, &connector))
                    .collect(),
            }
        }
    }
}

impl Translator {
    /// Convert diffs to a connector-specific collection of diffs
    pub fn translate_diffs_to_local(&self, diffs: &Diffs) -> LocalDiffs {
        LocalDiffs {
            groups: diffs
                .groups
                .iter()
                .map(|g| self.translate_group_diff_to_local(g))
                .collect(),
        }
    }
}
