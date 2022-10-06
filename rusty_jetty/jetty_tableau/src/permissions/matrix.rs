//! Operational utilities for dealing with the effective permissions matrix.
//!

use std::collections::{HashMap, HashSet};

use jetty_core::{
    connectors::{
        nodes::{EffectivePermission, SparseMatrix},
        UserIdentifier,
    },
    cual::Cual,
};

use anyhow::{bail, Result};

/// HashMap utility trait to assist with doing an insertion when it's easy,
/// and merging as needed when it's not.
pub(crate) trait InsertOrMerge<K, V> {
    /// Insert `key` into the map if it doesn't exist. Otherwise, merge
    /// `val` with what's already found at `key`.
    fn insert_or_merge(&mut self, key: K, val: V);
}

/// Top-level impl for a `SparseMatrix` like the one that holds effective
/// permissions for Tableau.
impl InsertOrMerge<UserIdentifier, HashMap<Cual, HashSet<EffectivePermission>>>
    for SparseMatrix<UserIdentifier, Cual, HashSet<EffectivePermission>>
{
    fn insert_or_merge(
        &mut self,
        key: UserIdentifier,
        new_asset_perms: HashMap<Cual, HashSet<EffectivePermission>>,
    ) {
        if let Some(user_perms) = self.get_mut(&key) {
            for (cual, new_perms) in new_asset_perms {
                user_perms.insert_or_merge(cual, new_perms);
            }
        } else {
            self.insert(key, new_asset_perms);
        }
    }
}

/// Inner impl for the SparseMatrix asset map from `CUAL` -> [`EffectivePermission`].
///
/// When there is a hash collision, use `EffectivePermission`'s merge impl to
/// gracefully merge them.
impl InsertOrMerge<Cual, HashSet<EffectivePermission>>
    for HashMap<Cual, HashSet<EffectivePermission>>
{
    fn insert_or_merge(&mut self, cual: Cual, new_perms: HashSet<EffectivePermission>) {
        let mut new_perms = new_perms.clone();
        if let Some(existing_user_asset_perms) = self.get_mut(&cual) {
            let mut merged_perms: HashSet<EffectivePermission> = existing_user_asset_perms
                .clone()
                .into_iter()
                .map(|mut existing_effective_permission| {
                    if let Some(new_ep) = new_perms.take(&existing_effective_permission) {
                        // Matched permissions. Merge mode and reasons.
                        existing_effective_permission.merge(new_ep.clone());
                    }
                    existing_effective_permission
                })
                .collect();
            // Add the remaining new permissions
            merged_perms.extend(new_perms);
            *existing_user_asset_perms = merged_perms;
        } else {
            self.insert(cual, new_perms);
        }
    }
}

/// Utility trait for merging two copies of the same struct. Like
/// `std::iter::Extend` except we can use it on types declared
/// outside this crate.
pub(crate) trait Merge<T> {
    fn merge(&mut self, other: T) -> Result<()>;
}

/// Top-level impl for a SparseMatrix. The incoming (`other`) matrix takes precedence
/// when there are clashes.
impl Merge<SparseMatrix<UserIdentifier, Cual, HashSet<EffectivePermission>>>
    for SparseMatrix<UserIdentifier, Cual, HashSet<EffectivePermission>>
{
    fn merge(
        &mut self,
        other: SparseMatrix<UserIdentifier, Cual, HashSet<EffectivePermission>>,
    ) -> Result<()> {
        for (uid, asset_map) in other {
            self.insert_or_merge(uid, asset_map);
        }
        Ok(())
    }
}

/// Merge impl for combining two `EffectivePermission`s. The second (`other`) permission
/// takes precedence when there are clashes.
impl Merge<EffectivePermission> for EffectivePermission {
    /// Should only be called for EffectivePermissions with the same privilege.
    fn merge(&mut self, other: EffectivePermission) -> Result<()> {
        if !(self.privilege == other.privilege) {
            bail!("effective permission privileges didn't match");
        } else if self.mode == other.mode {
            // If the mode is the same, we can combine
            // reasons to give a comprehensive list.
            self.reasons.extend(other.reasons);
        } else {
            // Combine them. The "other" effective permission takes precedence.
            self.mode = other.mode;
            self.reasons = other.reasons;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use jetty_core::connectors::nodes::{EffectivePermission, PermissionMode};

    use anyhow::Result;

    #[test]
    fn test_merge_effective_permissions_works() -> Result<()> {
        let mut ep1 = EffectivePermission::new("priv1".to_owned(), PermissionMode::Allow, vec![]);
        let ep2 = EffectivePermission::new("priv1".to_owned(), PermissionMode::Deny, vec![]);
        ep1.merge(ep2)?;
        assert_eq!(ep1.mode, PermissionMode::Deny);
        Ok(())
    }

    #[test]
    fn test_merge_effective_permissions_with_mismatched_privileges_fails() -> Result<()> {
        let mut ep1 = EffectivePermission::new("priv1".to_owned(), PermissionMode::Allow, vec![]);
        let ep2 = EffectivePermission::new("priv2".to_owned(), PermissionMode::Deny, vec![]);
        assert!(ep1.merge(ep2).is_err());
        Ok(())
    }

    #[test]
    fn test_merge_effective_permissions_reason_precedence_is_correct() -> Result<()> {
        let mut ep1 = EffectivePermission::new(
            "priv1".to_owned(),
            PermissionMode::Allow,
            vec!["reason".to_owned()],
        );
        let ep2 = EffectivePermission::new(
            "priv1".to_owned(),
            PermissionMode::Deny,
            vec!["another reason".to_owned()],
        );
        ep1.merge(ep2)?;
        assert_eq!(ep1.reasons, vec!["another reason".to_owned()]);
        Ok(())
    }

    #[test]
    fn test_merge_effective_permissions_merges_reasons_when_mode_matches() -> Result<()> {
        let mut ep1 = EffectivePermission::new(
            "priv1".to_owned(),
            PermissionMode::Allow,
            vec!["reason".to_owned()],
        );
        let ep2 = EffectivePermission::new(
            "priv1".to_owned(),
            PermissionMode::Allow,
            vec!["another reason".to_owned()],
        );
        ep1.merge(ep2)?;
        assert_eq!(
            ep1.reasons,
            vec!["reason".to_owned(), "another reason".to_owned()]
        );
        Ok(())
    }

    #[test]
    fn test_insert_or_merge_for_matrix_inserts() {
        let mut matrix = HashMap::new();
        matrix.insert_or_merge(UserIdentifier::Email("".to_owned()), HashMap::new());
        assert_eq!(
            matrix,
            HashMap::from([(UserIdentifier::Email("".to_owned()), HashMap::new())])
        );
    }

    #[test]
    fn test_insert_or_merge_for_matrix_merges() {
        let mut matrix = HashMap::from([(
            UserIdentifier::Email("".to_owned()),
            HashMap::from([(
                Cual::new("my_cual".to_owned()),
                HashSet::from([EffectivePermission::default()]),
            )]),
        )]);
        matrix.insert_or_merge(
            UserIdentifier::Email("".to_owned()),
            HashMap::from([(
                Cual::new("my_cual2".to_owned()),
                HashSet::from([EffectivePermission::default()]),
            )]),
        );
        assert_eq!(
            matrix,
            HashMap::from([(
                UserIdentifier::Email("".to_owned()),
                HashMap::from([
                    (
                        Cual::new("my_cual2".to_owned()),
                        HashSet::from([EffectivePermission::default()]),
                    ),
                    (
                        Cual::new("my_cual".to_owned()),
                        HashSet::from([EffectivePermission::default()]),
                    )
                ])
            )])
        );
    }

    #[test]
    fn test_insert_or_merge_for_matrix_merges_inner() {
        let mut matrix = HashMap::from([(
            UserIdentifier::Email("".to_owned()),
            HashMap::from([(
                Cual::new("my_cual".to_owned()),
                HashSet::from([EffectivePermission::default()]),
            )]),
        )]);
        matrix.insert_or_merge(
            UserIdentifier::Email("".to_owned()),
            HashMap::from([(
                Cual::new("my_cual".to_owned()),
                HashSet::from([EffectivePermission::new(
                    "priv".to_owned(),
                    PermissionMode::None,
                    vec![],
                )]),
            )]),
        );
        assert_eq!(
            matrix,
            HashMap::from([(
                UserIdentifier::Email("".to_owned()),
                HashMap::from([(
                    Cual::new("my_cual".to_owned()),
                    HashSet::from([
                        EffectivePermission::default(),
                        EffectivePermission::new("priv".to_owned(), PermissionMode::None, vec![],)
                    ]),
                )])
            )])
        );
    }
}
