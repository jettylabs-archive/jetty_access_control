use std::collections::HashMap;

use crate::{
    access_graph::translate::Translator,
    cual::Cual,
    write::{
        self,
        assets::{self, diff::default_policies::DefaultPolicyDiffDetails},
    },
};

#[derive(Debug)]
/// A group-specific local diff
pub struct LocalDiff {
    /// the asset being diffed
    pub asset: Cual,
    /// The wildcard path that the default policy matches
    pub path: String,
    /// The type of asset that the policy applies to
    pub asset_type: String,
    /// a map of users and user-specific changes for the asset
    /// Note: The type is the same as a regular policy because we don't need to
    /// pass the extra info about whether this is connector_managed
    pub users: HashMap<String, write::assets::diff::policies::DiffDetails>,
    /// a map of groups and group-specific changes for the asset
    pub groups: HashMap<String, write::assets::diff::policies::DiffDetails>,
}

impl Translator {
    pub(super) fn translate_default_policy_diff_to_local(
        &self,
        global_diff: &write::assets::diff::default_policies::DefaultPolicyDiff,
    ) -> Option<LocalDiff> {
        let res = LocalDiff {
            asset: self.asset_name_to_cual(&global_diff.asset).unwrap(),
            path: global_diff.path.to_owned(),
            asset_type: global_diff.asset_type.to_string(),
            users: global_diff
                .users
                .iter()
                .filter_map(|(agent, details)| {
                    translate_diff_details(details).map(|translated_details| {
                        (
                            self.translate_node_name_to_local(agent, &global_diff.connector),
                            translated_details,
                        )
                    })
                })
                .collect(),
            groups: global_diff
                .groups
                .iter()
                .filter_map(|(agent, details)| {
                    translate_diff_details(details).map(|translated_details| {
                        (
                            self.translate_node_name_to_local(agent, &global_diff.connector),
                            translated_details,
                        )
                    })
                })
                .collect(),
        };

        Some(res)
    }
}

fn translate_diff_details(
    default_policy_details: &DefaultPolicyDiffDetails,
) -> Option<write::assets::diff::policies::DiffDetails> {
    Some(match default_policy_details {
        DefaultPolicyDiffDetails::Add { add } => {
            if !add.connector_managed {
                return None;
            }
            write::assets::diff::policies::DiffDetails::AddAgent {
                add: translate_policy_state(add),
            }
        }
        DefaultPolicyDiffDetails::Remove { remove } => {
            write::assets::diff::policies::DiffDetails::RemoveAgent {
                remove: translate_policy_state(remove),
            }
        }
        DefaultPolicyDiffDetails::Modify {
            add,
            remove,
            connector_managed,
        } => {
            if !match &connector_managed {
                assets::diff::default_policies::ConnectorManagementDiff::Changed(v) => *v,
                assets::diff::default_policies::ConnectorManagementDiff::Unchanged(v) => *v,
            } {
                return None;
            }
            write::assets::diff::policies::DiffDetails::ModifyAgent {
                add: translate_policy_state(add),
                remove: translate_policy_state(remove),
            }
        }
    })
}

fn translate_policy_state(
    default_policy_state: &assets::DefaultPolicyState,
) -> assets::PolicyState {
    assets::PolicyState {
        privileges: default_policy_state.privileges.iter().cloned().collect(),
        metadata: default_policy_state.metadata.to_owned(),
    }
}
