//! Operational utilities for dealing with the effective permissions matrix.
//!

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use crate::connectors::nodes::{EffectivePermission, SparseMatrix};

use anyhow::{bail, Result};

/// HashMap utility trait to assist with doing an insertion when it's easy,
/// and merging as needed when it's not.
pub trait InsertOrMerge<K, V> {
    /// Insert `key` into the map if it doesn't exist. Otherwise, merge
    /// `val` with what's already found at `key`.
    fn insert_or_merge(&mut self, key: K, val: V);
}

/// Top-level impl for a `SparseMatrix` like the one that holds effective
/// permissions
impl<T, U, V> InsertOrMerge<T, HashMap<U, HashSet<V>>> for SparseMatrix<T, U, HashSet<V>>
where
    T: PartialEq + Hash + Eq,
    U: PartialEq + Hash + Eq,

    V: Merge + Clone + Eq + Hash,
{
    fn insert_or_merge(&mut self, key: T, new_asset_perms: HashMap<U, HashSet<V>>) {
        if let Some(user_perms) = self.get_mut(&key) {
            for (second_key, new_perms) in new_asset_perms {
                user_perms.insert_or_merge(second_key, new_perms);
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
impl<U, V> InsertOrMerge<U, HashSet<V>> for HashMap<U, HashSet<V>>
where
    U: PartialEq + Hash + Eq,
    V: Merge + Clone + Eq + Hash,
{
    fn insert_or_merge(&mut self, key: U, value: HashSet<V>) {
        let mut new_perms = value;
        if let Some(existing_values) = self.get_mut(&key) {
            let mut merged_perms: HashSet<V> = existing_values
                .clone()
                .into_iter()
                .map(|mut existing_value| {
                    if let Some(new_ep) = new_perms.take(&existing_value) {
                        // Matched permissions. Merge mode and reasons.
                        existing_value.merge(new_ep).unwrap();
                    }
                    existing_value
                })
                .collect();
            // Add the remaining new permissions
            merged_perms.extend(new_perms);
            *existing_values = merged_perms;
        } else {
            self.insert(key, new_perms);
        }
    }
}

/// Utility trait for merging two copies of the same struct. Like
/// `std::iter::Extend` except we can use it on types declared
/// outside this crate.
pub trait Merge {
    /// Merge two instances of a struct. Like
    /// `std::iter::Extend` except we can use it on types declared
    /// outside this crate.
    ///
    /// The default implementation simply keeps the value of self, unchanged.
    fn merge(&mut self, other: Self) -> Result<()>
    where
        Self: Clone,
    {
        // we don't actually need to do anything with other in the default case
        let _ = other;
        Ok(())
    }
}

/// Top-level impl for a SparseMatrix. The incoming (`other`) matrix takes precedence
/// when there are clashes.
impl<T, U, V> Merge for SparseMatrix<T, U, HashSet<V>>
where
    T: PartialEq + Hash + Eq,
    U: PartialEq + Hash + Eq,
    V: Merge + PartialEq + Hash + Eq + Clone,
{
    fn merge(&mut self, other: SparseMatrix<T, U, HashSet<V>>) -> Result<()> {
        for (uid, asset_map) in other {
            self.insert_or_merge(uid, asset_map);
        }
        Ok(())
    }
}

/// Merge a HashMap<T, HashSet<U>> by extending HashSet<U> when there is a HashMap
/// key collision.
///
/// Don't even think about using this with U: EffectivePermission 🐉.
/// EffectivePermission Hash and PartialEq are specialized for that type - this will lead
/// to unexpected results.
impl<T, U> Merge for HashMap<T, HashSet<U>>
where
    T: PartialEq + Hash + Eq,
    U: Merge + PartialEq + Hash + Eq + Clone,
{
    fn merge(&mut self, other: HashMap<T, HashSet<U>>) -> Result<()> {
        for (k, v) in other {
            self.entry(k)
                .and_modify(|o| o.extend(v.to_owned()))
                .or_insert(v);
        }
        Ok(())
    }
}

/// Trait to insert into a nested HashMap
pub trait DoubleInsert<K, Y, V> {
    /// Insert `key1` into the map if it doesn't exist. Insert `key2` if it doesn't exist, with value V.
    /// Will override any previous value
    fn double_insert(&mut self, key1: K, key2: Y, val: V) -> Option<V>;
}

impl<K, Y, V> DoubleInsert<K, Y, V> for SparseMatrix<K, Y, V>
where
    K: Hash + Eq,
    Y: Hash + Eq,
{
    fn double_insert(&mut self, key1: K, key2: Y, val: V) -> Option<V> {
        let x = self.entry(key1).or_default();
        x.insert(key2, val)
    }
}

/// Merge impl for combining two `EffectivePermission`s. The second (`other`) permission
/// takes precedence when there are clashes.
impl Merge for EffectivePermission {
    /// Should only be called for EffectivePermissions with the same privilege.
    fn merge(&mut self, other: EffectivePermission) -> Result<()> {
        if self.privilege != other.privilege {
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

    use crate::connectors::nodes::{EffectivePermission, PermissionMode};
    use crate::connectors::UserIdentifier;
    use crate::cual::Cual;

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
        matrix.insert_or_merge(
            UserIdentifier::Email("".to_owned()),
            HashMap::<String, HashSet<EffectivePermission>>::new(),
        );
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
                Cual::new("mycual://a"),
                HashSet::from([EffectivePermission::default()]),
            )]),
        )]);
        matrix.insert_or_merge(
            UserIdentifier::Email("".to_owned()),
            HashMap::from([(
                Cual::new("mycual2://a"),
                HashSet::from([EffectivePermission::default()]),
            )]),
        );
        assert_eq!(
            matrix,
            HashMap::from([(
                UserIdentifier::Email("".to_owned()),
                HashMap::from([
                    (
                        Cual::new("mycual2://a"),
                        HashSet::from([EffectivePermission::default()]),
                    ),
                    (
                        Cual::new("mycual://a"),
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
                Cual::new("mycual://a"),
                HashSet::from([EffectivePermission::default()]),
            )]),
        )]);
        matrix.insert_or_merge(
            UserIdentifier::Email("".to_owned()),
            HashMap::from([(
                Cual::new("mycual://a"),
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
                    Cual::new("mycual://a"),
                    HashSet::from([
                        EffectivePermission::default(),
                        EffectivePermission::new("priv".to_owned(), PermissionMode::None, vec![],)
                    ]),
                )])
            )])
        );
    }
}
