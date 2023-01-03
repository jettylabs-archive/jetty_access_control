//! Utilities for exploration of the graph.
//!

use std::collections::HashSet;

use petgraph::{stable_graph::NodeIndex, Direction};

use super::{AccessGraph, EdgeType, JettyNode};

impl AccessGraph {
    /// Get all the children nodes up to a particular depth by following non-repeating paths given certain
    /// criteria. It only looks at outgoing edges.
    ///
    /// - `from` is the name of the starting node
    /// - `edge_matcher` is a function that must return true to be able to follow the edge
    /// - `passthrough_matcher` is a function that must return true for the path to pass through that node
    /// - `output_matcher` is a function that must return true for a node to be a destination
    /// - `min_depth` is the minimum depth at which a target may be found
    /// - `max_depth` is how deep to search for children. If None, will continue until it runs out of children to visit.

    pub fn get_matching_descendants<T: Into<NodeIndex>, X: FnOnce(&JettyNode) -> bool + Copy>(
        &self,
        from: T,
        edge_matcher: fn(&EdgeType) -> bool,
        passthrough_matcher: fn(&JettyNode) -> bool,
        target_matcher: X,
        min_depth: Option<usize>,
        max_depth: Option<usize>,
    ) -> Vec<NodeIndex> {
        let idx: NodeIndex = from.into();

        let max_depth = if let Some(l) = max_depth {
            l
        } else {
            self.graph.graph.node_count() - 1
        };

        let min_depth = min_depth.unwrap_or(0);

        // list of visited nodes
        let mut visited = HashSet::new();
        let mut results = vec![];

        self.get_matching_descendants_recursive(
            idx,
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
    #[allow(clippy::too_many_arguments)]
    fn get_matching_descendants_recursive<X: FnOnce(&JettyNode) -> bool + Copy>(
        &self,
        idx: NodeIndex,
        edge_matcher: fn(&EdgeType) -> bool,
        passthrough_matcher: fn(&JettyNode) -> bool,
        target_matcher: X,
        min_depth: usize,
        max_depth: usize,
        current_depth: usize,
        visited: &mut HashSet<NodeIndex>,
        results: &mut Vec<NodeIndex>,
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
            let node_weight = &self[child];
            // Are we beyond the minimum depth?
            if current_depth >= min_depth {
                // is it the target node type?
                if target_matcher(node_weight) {
                    results.push(child);
                }
            }

            // Is it a passthrough type?
            if passthrough_matcher(node_weight) {
                self.get_matching_descendants_recursive(
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

    /// Get adjacent nodes that match TargetMatcher that are connected by an edge matching edge_matcher
    pub fn get_matching_children<T: Into<NodeIndex>, X: FnOnce(&JettyNode) -> bool + Copy>(
        &self,
        from: T,
        edge_matcher: fn(&EdgeType) -> bool,
        target_matcher: X,
    ) -> Vec<NodeIndex> {
        self.get_matching_descendants(
            from,
            edge_matcher,
            |_| false,
            target_matcher,
            Some(1),
            Some(1),
        )
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        access_graph::{GroupAttributes, NodeName, UserAttributes},
        jetty::ConnectorNamespace,
    };

    use anyhow::Result;

    use super::*;

    #[test]
    fn get_matching_simple_paths_works() -> Result<()> {
        let ag = AccessGraph::new_dummy(
            &[
                &JettyNode::User(UserAttributes::simple_new("user".to_owned())),
                &JettyNode::Group(GroupAttributes::new("group1".to_owned())),
                &JettyNode::Group(GroupAttributes::new("group2".to_owned())),
                &JettyNode::Group(GroupAttributes::new("group3".to_owned())),
                &JettyNode::Group(GroupAttributes::new("group4".to_owned())),
            ],
            &[
                (
                    NodeName::User("user".to_owned()),
                    NodeName::Group {
                        name: "group1".to_owned(),
                        origin: ConnectorNamespace::default(),
                    },
                    EdgeType::MemberOf,
                ),
                (
                    NodeName::User("user".to_owned()),
                    NodeName::Group {
                        name: "group2".to_owned(),
                        origin: ConnectorNamespace::default(),
                    },
                    EdgeType::MemberOf,
                ),
                (
                    NodeName::Group {
                        name: "group2".to_owned(),
                        origin: ConnectorNamespace::default(),
                    },
                    NodeName::Group {
                        name: "group1".to_owned(),
                        origin: ConnectorNamespace::default(),
                    },
                    EdgeType::MemberOf,
                ),
                (
                    NodeName::Group {
                        name: "group2".to_owned(),
                        origin: ConnectorNamespace::default(),
                    },
                    NodeName::Group {
                        name: "group3".to_owned(),
                        origin: ConnectorNamespace::default(),
                    },
                    EdgeType::MemberOf,
                ),
                (
                    NodeName::Group {
                        name: "group2".to_owned(),
                        origin: ConnectorNamespace::default(),
                    },
                    NodeName::Group {
                        name: "group4".to_owned(),
                        origin: ConnectorNamespace::default(),
                    },
                    EdgeType::MemberOf,
                ),
                (
                    NodeName::Group {
                        name: "group3".to_owned(),
                        origin: ConnectorNamespace::default(),
                    },
                    NodeName::Group {
                        name: "group4".to_owned(),
                        origin: ConnectorNamespace::default(),
                    },
                    EdgeType::MemberOf,
                ),
                (
                    NodeName::Group {
                        name: "group4".to_owned(),
                        origin: ConnectorNamespace::default(),
                    },
                    NodeName::Group {
                        name: "group1".to_owned(),
                        origin: ConnectorNamespace::default(),
                    },
                    EdgeType::MemberOf,
                ),
            ],
        );

        // Test path generation
        let a = ag.all_matching_simple_paths(
            ag.get_untyped_index_from_name(&NodeName::User("user".to_owned()))
                .unwrap(),
            ag.get_untyped_index_from_name(&NodeName::Group {
                name: "group1".to_owned(),
                origin: ConnectorNamespace::default(),
            })
            .unwrap(),
            |_| true,
            |_| true,
            None,
            None,
        );
        assert_eq!(a.len(), 4);

        // Test depth limits
        let a = ag.all_matching_simple_paths(
            ag.get_untyped_index_from_name(&NodeName::User("user".to_owned()))
                .unwrap(),
            ag.get_untyped_index_from_name(&NodeName::Group {
                name: "group1".to_owned(),
                origin: ConnectorNamespace::default(),
            })
            .unwrap(),
            |_| true,
            |_| true,
            Some(2),
            Some(3),
        );
        assert_eq!(a.len(), 2);

        // Test depth limits again
        let a = ag.all_matching_simple_paths(
            ag.get_untyped_index_from_name(&NodeName::User("user".to_owned()))
                .unwrap(),
            ag.get_untyped_index_from_name(&NodeName::Group {
                name: "group1".to_owned(),
                origin: ConnectorNamespace::default(),
            })
            .unwrap(),
            |_| true,
            |_| true,
            Some(2),
            Some(2),
        );
        assert_eq!(a.len(), 1);

        // Test edge matching
        let a = ag.all_matching_simple_paths(
            ag.get_untyped_index_from_name(&NodeName::User("user".to_owned()))
                .unwrap(),
            ag.get_untyped_index_from_name(&NodeName::Group {
                name: "group1".to_owned(),
                origin: ConnectorNamespace::default(),
            })
            .unwrap(),
            |n| matches!(n, EdgeType::Other),
            |_| true,
            None,
            None,
        );
        assert_eq!(a.len(), 0);

        // Test passthrough matching
        let a = ag.all_matching_simple_paths(
            ag.get_untyped_index_from_name(&NodeName::User("user".to_owned()))
                .unwrap(),
            ag.get_untyped_index_from_name(&NodeName::Group {
                name: "group1".to_owned(),
                origin: ConnectorNamespace::default(),
            })
            .unwrap(),
            |_| true,
            |n| n.get_string_name() == *"::group2",
            None,
            None,
        );
        a.iter().for_each(|p| {
            dbg!(ag.path_as_string(p));
        });
        assert_eq!(a.len(), 2);

        Ok(())
    }
}
