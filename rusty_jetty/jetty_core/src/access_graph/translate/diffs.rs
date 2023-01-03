//! Types and functionality to convert diffs to a state that can be processed by connectors

/// default policy-specific diff functionality
pub mod default_policies;
/// Group-specific diff functionality
pub mod groups;
/// policy-specific diff functionality
pub mod policies;
/// User-specific diff functionality
pub mod users;

use crate::{jetty::ConnectorNamespace, write::GlobalDiffs};

use super::Translator;

/// Diffs in the namespace of the connectors
pub struct LocalConnectorDiffs {
    /// The group-specific diffs
    pub groups: Vec<groups::LocalDiff>,
    /// The users-specific diffs
    pub users: Vec<users::LocalDiff>,
    /// The default_policies-specific diffs
    pub default_policies: Vec<default_policies::LocalDiff>,
    /// The policies-specific diffs
    pub policies: Vec<policies::LocalDiff>,
}

impl Translator {
    /// Convert diffs to a connector-specific collection of diffs
    pub fn translate_diffs_to_local(
        &self,
        diffs: &GlobalDiffs,
        connector: &ConnectorNamespace,
    ) -> LocalConnectorDiffs {
        LocalConnectorDiffs {
            groups: diffs
                .groups
                .iter()
                .map(|g| self.translate_group_diff_to_local(g))
                .collect(),
            users: diffs
                .users
                .iter()
                .map(|g| self.translate_user_diff_to_local(g, connector))
                .collect::<Option<_>>()
                .unwrap_or_default(),
            default_policies: diffs
                .default_policies
                .iter()
                .map(|g| self.translate_default_policy_diff_to_local(g))
                .collect::<Option<_>>()
                .unwrap_or_default(),
            policies: diffs
                .policies
                .iter()
                .map(|g| self.translate_policy_diff_to_local(g))
                .collect(),
        }
    }
}
