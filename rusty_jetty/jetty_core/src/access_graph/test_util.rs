//! Utilities for testing
//!

use std::collections::HashMap;

use super::graph::Graph;

pub(crate) fn new_graph() -> Graph {
    Graph {
        graph: petgraph::stable_graph::StableDiGraph::new(),
        nodes: HashMap::new(),
    }
}
