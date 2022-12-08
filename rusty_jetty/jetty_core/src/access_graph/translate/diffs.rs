//! Types and functionality to convert diffs to a state that can be processed by connectors

pub struct LocalDiffs {
    groups: Vec<groups::LocalDiff>,
}

mod groups {
    use crate::{
        access_graph::translate::Translator,
        jetty::ConnectorNamespace,
        write::groups::{self, GroupMemberChanges},
    };
    use anyhow::Result;
    use std::collections::HashSet;

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

    #[derive(Debug)]
    pub struct LocalGroupMemberChanges {
        /// users
        pub users: HashSet<String>,
        /// groups
        pub groups: HashSet<String>,
    }

    impl Translator {
        fn translate_group_diff_to_local(
            &self,
            global_diff: &groups::Diff,
            connector: &ConnectorNamespace,
        ) -> Result<LocalDiff> {
            Ok(LocalDiff {
                group_name: self.translate_node_name_to_local(&global_diff.group_name, &connector),
                details: match &global_diff.details {
                    groups::DiffDetails::AddGroup { members } => LocalDiffDetails::AddGroup {
                        members: self.translate_group_member_changes_to_local(&members, connector),
                    },
                    groups::DiffDetails::RemoveGroup => LocalDiffDetails::RemoveGroup,
                    groups::DiffDetails::ModifyGroup { add, remove } => {
                        LocalDiffDetails::ModifyGroup {
                            add: self.translate_group_member_changes_to_local(&add, connector),
                            remove: self
                                .translate_group_member_changes_to_local(&remove, connector),
                        }
                    }
                },
            })
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
