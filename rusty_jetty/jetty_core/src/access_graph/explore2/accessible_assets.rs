//! Utilities to return only part of a graph
//!

use std::collections::{HashMap, HashSet};

use crate::{
    access_graph::{AccessGraph},
    connectors::{
        nodes::{EffectivePermission, PermissionMode},
        UserIdentifier,
    },
    cual::Cual,
};

impl AccessGraph {
    /// Return accessible assets
    pub fn get_user_accessible_assets<'a>(
        &'a self,
        user: &UserIdentifier,
    ) -> HashMap<Cual, HashSet<&'a EffectivePermission>> {
        let perms = &self.effective_permissions[user];
        perms
            .iter()
            .filter_map(|(k, v)| {
                if v.iter().any(|p| p.mode == PermissionMode::Allow) {
                    Some((
                        k.to_owned(),
                        v.iter()
                            .filter(|p| p.mode == PermissionMode::Allow)
                            .collect(),
                    ))
                } else {
                    // Access not allowed
                    None
                }
            })
            .collect()
    }

    /// Return accessible assets by user
    pub fn get_users_with_access_to_asset<'a>(
        &'a self,
        asset: Cual,
    ) -> HashMap<UserIdentifier, HashSet<&'a EffectivePermission>> {
        let perms = get_access_by_asset(&self.effective_permissions, asset);
        perms
            .iter()
            .filter_map(|(k, v)| {
                if v.iter().any(|p| p.mode == PermissionMode::Allow) {
                    Some((
                        k.to_owned(),
                        v.iter()
                            .filter(|p| p.mode == PermissionMode::Allow)
                            .collect(),
                    ))
                } else {
                    // Access not allowed
                    None
                }
            })
            .collect::<HashMap<UserIdentifier, HashSet<&EffectivePermission>>>()
    }
}

fn get_access_by_asset(
    m: &HashMap<UserIdentifier, HashMap<Cual, HashSet<EffectivePermission>>>,
    cual: Cual,
) -> HashMap<UserIdentifier, &HashSet<EffectivePermission>> {
    m.iter()
        .filter_map(|(k, v)| v.get(&cual).and_then(|ep| Some((k.to_owned(), ep))))
        .collect::<HashMap<_, _>>()
}
