//! Utilities to return only part of a graph
//!

use anyhow::{anyhow, Result};

use crate::access_graph::{AccessGraph, JettyNode, NodeName};

impl AccessGraph {
    /// Return a node when given a name
    pub fn get_node<'a>(&'a self, node_name: &NodeName) -> Result<&'a JettyNode> {
        let idx = self
            .get_untyped_index_from_name(node_name)
            .ok_or_else(|| anyhow!("unable to find node"))?;
        Ok(&self[idx])
    }
}
