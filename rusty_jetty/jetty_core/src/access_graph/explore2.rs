//! Utilities for exploration of the graph.
//!

use std::collections::HashSet;

use petgraph::{stable_graph::NodeIndex, visit::IntoNodeReferences, Direction};

use super::{AccessGraph, EdgeType, JettyNode, NodeName};

impl AccessGraph {
    /// Get all nodes from the graph
    pub fn get_nodes(&self) -> petgraph::stable_graph::NodeReferences<super::JettyNode> {
        self.graph.graph.node_references()
    }

    /// Get all the children nodes up to a particular depth by following non-repeating paths given certain
    /// criteria. It only looks at outgoing edges.
    ///
    /// `from` is the name of the starting node
    /// `edge_matcher` is a function that must return true to be able to follow the edge
    /// `passthrough_matcher` is a function that must return true for the path to pass through that node
    /// `output_matcher` is a function that must return true for a node to be a destination
    /// `min_depth` is the minimum depth at which a target may be found
    /// `max_depth` is how deep to search for children. If None, will continue until it runs out of children to visit.
    pub fn get_matching_children(
        &self,
        from: &NodeName,
        edge_matcher: fn(&EdgeType) -> bool,
        passthrough_matcher: fn(&JettyNode) -> bool,
        target_matcher: fn(&JettyNode) -> bool,
        min_depth: Option<usize>,
        max_depth: Option<usize>,
    ) -> Vec<JettyNode> {
        let idx = self.graph.nodes.get(from).unwrap();

        let max_depth = if let Some(l) = max_depth {
            l
        } else {
            self.graph.graph.node_count() - 1
        };

        let min_depth = min_depth.unwrap_or(0);

        // list of visited nodes
        let mut visited = HashSet::new();
        let mut results = vec![];

        self.get_matching_children_recursive(
            *idx,
            edge_matcher,
            passthrough_matcher,
            target_matcher,
            min_depth,
            max_depth,
            0,
            &mut visited,
            &mut results,
        );

        results
    }

    /// Start with a node, then get all of its children. If they're the target type, add them to the result.
    /// If not the target, keep going.
    fn get_matching_children_recursive(
        &self,
        idx: NodeIndex,
        edge_matcher: fn(&EdgeType) -> bool,
        passthrough_matcher: fn(&JettyNode) -> bool,
        target_matcher: fn(&JettyNode) -> bool,
        min_depth: usize,
        max_depth: usize,
        current_depth: usize,
        visited: &mut HashSet<NodeIndex>,
        results: &mut Vec<JettyNode>,
    ) {
        let legal_connections = self
            .graph
            .graph
            .edges_directed(idx, Direction::Outgoing)
            .filter(|e| edge_matcher(e.weight()))
            .map(|e| petgraph::visit::EdgeRef::target(&e));

        // Update depth because we're now looking at the children
        let current_depth = current_depth + 1;

        // Did we go too deep?
        if current_depth > max_depth {
            return;
        }

        for child in legal_connections {
            // Has it already been inserted?
            if !visited.insert(child) {
                continue;
            }

            // Get the node we're looking at
            let node_weight = &self.graph.graph[child];
            // Are we beyond the minimum depth?
            if current_depth >= min_depth {
                // is it the target node type?
                if target_matcher(node_weight) {
                    results.push(node_weight.to_owned());
                }
            }

            // Is it a passthrough type?
            if passthrough_matcher(node_weight) {
                self.get_matching_children_recursive(
                    child,
                    edge_matcher,
                    passthrough_matcher,
                    target_matcher,
                    min_depth,
                    max_depth,
                    current_depth,
                    visited,
                    results,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use crate::{
        access_graph::{AssetAttributes, PolicyAttributes, UserAttributes},
        connectors::AssetType,
        cual::Cual,
    };

    use anyhow::Result;

    use super::*;

    #[test]
    fn follow_passthrough_works() -> Result<()> {
        let test_asset = JettyNode::Asset(AssetAttributes {
            cual: Cual::new("my_cual".to_owned()),
            asset_type: AssetType::default(),
            metadata: HashMap::new(),
            connectors: HashSet::new(),
        });

        let ag = AccessGraph::new_dummy(
            &[
                &test_asset,
                &JettyNode::Policy(PolicyAttributes::new("policy".to_owned())),
                &JettyNode::User(UserAttributes::new("user".to_owned())),
            ],
            &[
                (
                    NodeName::User("user".to_owned()),
                    NodeName::Policy("policy".to_owned()),
                    EdgeType::GrantedBy,
                ),
                (
                    NodeName::Policy("policy".to_owned()),
                    NodeName::Asset("my_cual".to_owned()),
                    EdgeType::Governs,
                ),
            ],
        );

        // Test Edge Matching
        let a = ag.get_matching_children(
            &NodeName::User("user".to_owned()),
            |n| matches!(n, EdgeType::GrantedBy),
            |_| true,
            |_| true,
            None,
            None,
        );
        assert_eq!(a.len(), 0);

        // Test getting all children
        let a = ag.get_matching_children(
            &NodeName::User("user".to_owned()),
            |_| true,
            |_| true,
            |_| true,
            None,
            None,
        );
        assert_eq!(a.len(), 2);

        // Test target matching
        let a = ag.get_matching_children(
            &NodeName::User("user".to_owned()),
            |_| true,
            |_| true,
            |n| matches!(n, JettyNode::Asset(_)),
            None,
            None,
        );
        assert_eq!(a.len(), 1);

        // Test passthrough matching
        let a = ag.get_matching_children(
            &NodeName::User("user".to_owned()),
            |n| matches!(n, EdgeType::Other),
            |n| matches!(n, JettyNode::Policy(_)),
            |n| matches!(n, JettyNode::Asset(_)),
            None,
            None,
        );
        assert_eq!(a.len(), 1);

        let a = ag.get_matching_children(
            &NodeName::User("user".to_owned()),
            |n| matches!(n, EdgeType::Other),
            |n| matches!(n, JettyNode::User(_)),
            |n| matches!(n, JettyNode::Asset(_)),
            None,
            None,
        );
        assert_eq!(a.len(), 0);
        Ok(())
    }
}
