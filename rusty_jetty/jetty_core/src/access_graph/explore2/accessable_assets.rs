//! Utilities to return only part of a graph
//!

use std::collections::{HashMap, HashSet};

use petgraph::{stable_graph::NodeIndex, visit::EdgeRef};

use crate::{
    access_graph::{AccessGraph, EdgeType, JettyNode, NodeName},
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
    ) -> HashMap<Cual, &'a HashSet<EffectivePermission>> {
        let perms = &self.effective_permissions[user];
        perms
            .iter()
            .filter_map(|(k, v)| {
                if v.iter().any(|p| p.mode == PermissionMode::Allow) {
                    Some((k.to_owned(), v))
                } else {
                    None
                }
            })
            .collect::<HashMap<Cual, &HashSet<EffectivePermission>>>()
    }
}
