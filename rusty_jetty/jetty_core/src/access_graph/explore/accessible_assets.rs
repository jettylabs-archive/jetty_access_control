//! Utilities to return only part of a graph
//!

use std::collections::{HashMap, HashSet};

use crate::{
    access_graph::{
        graph::typed_indices::{AssetIndex, UserIndex},
        AccessGraph,
    },
    connectors::nodes::{EffectivePermission, PermissionMode},
};

impl AccessGraph {
    /// Return accessible assets
    pub fn get_user_accessible_assets(
        &self,
        user: UserIndex,
    ) -> HashMap<AssetIndex, HashSet<&EffectivePermission>> {
        let perms = &self.effective_permissions[&user];
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

    /// Return accessible assets by user. To be accessible, the asset must have at least one Allow privilege
    pub fn get_users_with_access_to_asset(
        &self,
        asset: AssetIndex,
    ) -> HashMap<UserIndex, HashSet<&EffectivePermission>> {
        let perms = get_access_by_asset(&self.effective_permissions, asset);
        perms
            .iter()
            .filter_map(|(k, v)| {
                // make sure that the user has an allow mode on at least one privilege
                if v.iter().any(|p| p.mode == PermissionMode::Allow) {
                    Some((
                        k.to_owned(),
                        v.iter()
                            // only show allow privileges
                            .filter(|p| p.mode == PermissionMode::Allow)
                            .collect(),
                    ))
                } else {
                    // Access not allowed
                    None
                }
            })
            .collect::<HashMap<UserIndex, HashSet<&EffectivePermission>>>()
    }
}

fn get_access_by_asset(
    m: &HashMap<UserIndex, HashMap<AssetIndex, HashSet<EffectivePermission>>>,
    asset: AssetIndex,
) -> HashMap<UserIndex, &HashSet<EffectivePermission>> {
    m.iter()
        .filter_map(|(k, v)| v.get(&asset).map(|ep| (k.to_owned(), ep)))
        .collect::<HashMap<_, _>>()
}
