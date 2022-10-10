//! Utilities for exploration of the graph.
//!

use petgraph::visit::IntoNodeReferences;

use super::AccessGraph;

impl AccessGraph {
    /// Get all nodes from the graph
    pub fn get_nodes(&self) -> petgraph::stable_graph::NodeReferences<super::JettyNode> {
        self.graph.graph.node_references()
    }
}
