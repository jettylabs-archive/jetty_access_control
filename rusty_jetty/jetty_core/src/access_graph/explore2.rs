//! Utilities for exploration of the graph.
//!

use std::hash::Hash;
use std::{collections::HashSet, iter::from_fn};

use indexmap::IndexSet;
use petgraph::{
    stable_graph::NodeIndex,
    visit::{IntoNeighborsDirected, IntoNodeReferences, NodeCount},
    Direction,
};

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

    fn all_matching_simple_paths(
        &self,
        from: &NodeName,
        to: &NodeName,
        edge_matcher: fn(&EdgeType) -> bool,
        passthrough_matcher: fn(&JettyNode) -> bool,
        min_depth: Option<usize>,
        max_depth: Option<usize>,
    ) -> Vec<Vec<JettyNode>> {
        let from_idx = self.graph.nodes.get(from).unwrap();
        let to_idx = self.graph.nodes.get(to).unwrap();

        let max_depth = if let Some(l) = max_depth {
            l
        } else {
            self.graph.graph.node_count() - 1
        };

        let min_depth = min_depth.unwrap_or(0);

        // list of visited nodes
        let mut visited = IndexSet::from([(from_idx.to_owned())]);
        let mut results = vec![];

        self.all_matching_simple_paths_recursive(
            *from_idx,
            *to_idx,
            edge_matcher,
            passthrough_matcher,
            min_depth,
            max_depth,
            0,
            &mut visited,
            &mut results,
        );

        results
    }

    /// Returns a Vec of Vec<JettyNodes> representing the matching non-cyclic paths
    /// between two nodes
    fn all_matching_simple_paths_recursive(
        &self,
        from_idx: NodeIndex,
        to_idx: NodeIndex,
        edge_matcher: fn(&EdgeType) -> bool,
        passthrough_matcher: fn(&JettyNode) -> bool,
        min_depth: usize,
        max_depth: usize,
        current_depth: usize,
        visited: &mut IndexSet<NodeIndex>,
        results: &mut Vec<Vec<JettyNode>>,
    ) {
        let legal_connections = self
            .graph
            .graph
            .edges_directed(from_idx, Direction::Outgoing)
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

            // Are we beyond the minimum depth?
            if current_depth >= min_depth {
                // is it the target node? if so, add the path to the results, pop
                // the node from visited and carry on with the next child
                if child == to_idx {
                    let path = visited
                        .iter()
                        .cloned()
                        .map(|i| self.graph.graph[i].to_owned())
                        .collect::<Vec<_>>();
                    results.push(path);
                    visited.pop();
                    continue;
                }
            }

            // Get the node we're looking at
            let node_weight = &self.graph.graph[child];
            // Is it a passthrough type?
            if passthrough_matcher(node_weight) {
                self.all_matching_simple_paths_recursive(
                    child,
                    to_idx,
                    edge_matcher,
                    passthrough_matcher,
                    min_depth,
                    max_depth,
                    current_depth,
                    visited,
                    results,
                );
            }
            visited.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use crate::{
        access_graph::{AssetAttributes, GroupAttributes, PolicyAttributes, UserAttributes},
        connectors::AssetType,
        cual::Cual,
    };

    use anyhow::Result;

    use super::*;

    #[test]
    fn get_matching_children_works() -> Result<()> {
        let ag = AccessGraph::new_dummy(
            &[
                &JettyNode::Asset(AssetAttributes::new(Cual::new("my_cual".to_owned()))),
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
            |n| matches!(n, EdgeType::MemberOf),
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
            |_| true,
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

    #[test]
    fn get_matching_simple_paths_works() -> Result<()> {
        let ag = AccessGraph::new_dummy(
            &[
                &JettyNode::User(UserAttributes::new("user".to_owned())),
                &JettyNode::Group(GroupAttributes::new("group1".to_owned())),
                &JettyNode::Group(GroupAttributes::new("group2".to_owned())),
                &JettyNode::Group(GroupAttributes::new("group3".to_owned())),
                &JettyNode::Group(GroupAttributes::new("group4".to_owned())),
            ],
            &[
                (
                    NodeName::User("user".to_owned()),
                    NodeName::Group("group1".to_owned()),
                    EdgeType::MemberOf,
                ),
                (
                    NodeName::User("user".to_owned()),
                    NodeName::Group("group2".to_owned()),
                    EdgeType::MemberOf,
                ),
                (
                    NodeName::Group("group2".to_owned()),
                    NodeName::Group("group1".to_owned()),
                    EdgeType::MemberOf,
                ),
                (
                    NodeName::Group("group2".to_owned()),
                    NodeName::Group("group3".to_owned()),
                    EdgeType::MemberOf,
                ),
                (
                    NodeName::Group("group2".to_owned()),
                    NodeName::Group("group4".to_owned()),
                    EdgeType::MemberOf,
                ),
                (
                    NodeName::Group("group3".to_owned()),
                    NodeName::Group("group4".to_owned()),
                    EdgeType::MemberOf,
                ),
                (
                    NodeName::Group("group4".to_owned()),
                    NodeName::Group("group1".to_owned()),
                    EdgeType::MemberOf,
                ),
            ],
        );

        // Test path generation
        let a = ag.all_matching_simple_paths(
            &NodeName::User("user".to_owned()),
            &NodeName::Group("group1".to_owned()),
            |_| true,
            |_| true,
            None,
            None,
        );
        assert_eq!(a.len(), 4);

        // Test depth limits
        let a = ag.all_matching_simple_paths(
            &NodeName::User("user".to_owned()),
            &NodeName::Group("group1".to_owned()),
            |_| true,
            |_| true,
            Some(2),
            Some(3),
        );
        assert_eq!(a.len(), 2);

        // Test depth limits again
        let a = ag.all_matching_simple_paths(
            &NodeName::User("user".to_owned()),
            &NodeName::Group("group1".to_owned()),
            |_| true,
            |_| true,
            Some(2),
            Some(2),
        );
        assert_eq!(a.len(), 1);

        // Test edge matching
        let a = ag.all_matching_simple_paths(
            &NodeName::User("user".to_owned()),
            &NodeName::Group("group1".to_owned()),
            |n| matches!(n, EdgeType::Other),
            |_| true,
            None,
            None,
        );
        assert_eq!(a.len(), 0);

        // Test passthrough matching
        let a = ag.all_matching_simple_paths(
            &NodeName::User("user".to_owned()),
            &NodeName::Group("group1".to_owned()),
            |_| true,
            |n| n.get_string_name() == "group2".to_owned(),
            None,
            None,
        );
        assert_eq!(a.len(), 2);

        Ok(())
    }
}
