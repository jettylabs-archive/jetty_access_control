//! Utilities for exploration of the graph.
//!

mod matching_children;
mod matching_paths;
mod matching_paths_to_children;

use std::fmt::Display;

use petgraph::visit::IntoNodeReferences;

use super::{AccessGraph, EdgeType, JettyNode, NodeName};

/// A path from one node to another, including start and end nodes.
/// Inside, it's a Vec<JettyNode>
pub struct NodePath(Vec<JettyNode>);

impl Display for NodePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|n| n.get_string_name())
                .collect::<Vec<_>>()
                .join(" ⇨ ")
        )
    }
}

impl AccessGraph {
    /// Get all nodes from the graph
    pub fn get_nodes(&self) -> petgraph::stable_graph::NodeReferences<super::JettyNode> {
        self.graph.graph.node_references()
    }
}
