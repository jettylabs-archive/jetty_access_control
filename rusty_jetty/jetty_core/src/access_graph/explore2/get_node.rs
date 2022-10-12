//! Utilities to return only part of a graph
//!

use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, Result};
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
    pub fn get_node<'a>(&'a self, node_name: &NodeName) -> Result<&'a JettyNode> {
        let idx = self
            .graph
            .get_node(node_name)
            .ok_or_else(|| anyhow!("unable to find node"))?;
        Ok(&self.graph.graph[*idx])
    }
}
