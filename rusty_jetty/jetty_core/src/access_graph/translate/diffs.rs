//! Types and functionality to convert diffs to a state that can be processed by connectors

/// default policy-specific diff functionality
pub mod default_policies;
/// Group-specific diff functionality
pub mod groups;
/// policy-specific diff functionality
pub mod policies;
/// User-specific diff functionality
pub mod users;

use crate::write::GlobalConnectorDiffs;

use super::Translator;

/// Diffs in the namespace of the connectors
pub struct LocalConnectorDiffs {
    /// The group-specific diffs
    pub groups: Vec<groups::LocalDiff>,
}

impl Translator {
    /// Convert diffs to a connector-specific collection of diffs
    pub fn translate_diffs_to_local(&self, diffs: &GlobalConnectorDiffs) -> LocalConnectorDiffs {
        LocalConnectorDiffs {
            groups: diffs
                .groups
                .iter()
                .map(|g| self.translate_group_diff_to_local(g))
                .collect(),
        }
    }
}
