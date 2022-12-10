//! Helpers to represent data on its way into the graph

use std::collections::HashSet;

use super::{EdgeType, JettyEdge, JettyNode, NodeName};

/// All helper types implement NodeHelpers.
pub(crate) trait NodeHelper {
    /// Return a JettyNode from the helper
    fn get_node(&self) -> Option<JettyNode>;
    /// Return a set of JettyEdges from the helper
    fn get_edges(&self) -> HashSet<JettyEdge>;
}

pub(crate) fn insert_edge_pair(
    hs: &mut HashSet<JettyEdge>,
    from: NodeName,
    to: NodeName,
    edge_type: EdgeType,
) {
    hs.insert(JettyEdge {
        from: from.to_owned(),
        to: to.to_owned(),
        edge_type: edge_type.to_owned(),
    });
    hs.insert(JettyEdge {
        from: to,
        to: from,
        edge_type: super::get_edge_type_pair(&edge_type),
    });
}
