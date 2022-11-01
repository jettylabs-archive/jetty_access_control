//! Utilities for testing
//!
#![cfg(test)]

use anyhow::Result;



use super::{graph::Graph, EdgeType, JettyEdge, JettyNode, NodeName};

/// Abstract some of the boilerplate and make it easy to spin up a new graph
/// quickly.
pub(crate) fn new_graph_with(
    nodes: &[&JettyNode],
    edges: &[(NodeName, NodeName, EdgeType)],
) -> Result<Graph> {
    let mut graph = new_graph();

    for node in nodes {
        graph.add_node(node)?;
    }
    for edge in edges {
        graph.add_edge(JettyEdge::new(edge.0.clone(), edge.1.clone(), edge.2));
    }

    Ok(graph)
}

pub(crate) fn new_graph() -> Graph {
    Graph {
        graph: petgraph::stable_graph::StableDiGraph::new(),
        nodes: Default::default(),
        node_ids: Default::default(),
    }
}
