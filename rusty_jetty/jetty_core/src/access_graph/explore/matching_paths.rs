//! Utilities for exploration of the graph.
//!

use indexmap::IndexSet;
use petgraph::{stable_graph::NodeIndex, Direction};

use super::{AccessGraph, EdgeType, JettyNode, NodePath};

impl AccessGraph {
    /// Return all the matching paths from one node to another. Specify filter functions
    /// to match edges and passthrough nodes
    pub fn all_matching_simple_paths<T: Into<NodeIndex> + Copy>(
        &self,
        from: T,
        to: T,
        edge_matcher: fn(&EdgeType) -> bool,
        passthrough_matcher: fn(&JettyNode) -> bool,
        min_depth: Option<usize>,
        max_depth: Option<usize>,
    ) -> Vec<NodePath> {
        let from_idx: NodeIndex = from.into();
        let to_idx = to.into();

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
            from_idx,
            to_idx,
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
    #[allow(clippy::too_many_arguments)]
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
        results: &mut Vec<NodePath>,
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
                    let path = visited.iter().cloned().collect::<Vec<_>>();
                    results.push(NodePath(path));
                    visited.pop();
                    continue;
                }
            }

            // Get the node we're looking at
            let node_weight = &self[child];
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

    use crate::access_graph::{GroupAttributes, NodeName, UserAttributes};

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
                        origin: Default::default(),
                    },
                    EdgeType::MemberOf,
                ),
                (
                    NodeName::User("user".to_owned()),
                    NodeName::Group {
                        name: "group2".to_owned(),
                        origin: Default::default(),
                    },
                    EdgeType::MemberOf,
                ),
                (
                    NodeName::Group {
                        name: "group2".to_owned(),
                        origin: Default::default(),
                    },
                    NodeName::Group {
                        name: "group1".to_owned(),
                        origin: Default::default(),
                    },
                    EdgeType::MemberOf,
                ),
                (
                    NodeName::Group {
                        name: "group2".to_owned(),
                        origin: Default::default(),
                    },
                    NodeName::Group {
                        name: "group3".to_owned(),
                        origin: Default::default(),
                    },
                    EdgeType::MemberOf,
                ),
                (
                    NodeName::Group {
                        name: "group2".to_owned(),
                        origin: Default::default(),
                    },
                    NodeName::Group {
                        name: "group4".to_owned(),
                        origin: Default::default(),
                    },
                    EdgeType::MemberOf,
                ),
                (
                    NodeName::Group {
                        name: "group3".to_owned(),
                        origin: Default::default(),
                    },
                    NodeName::Group {
                        name: "group4".to_owned(),
                        origin: Default::default(),
                    },
                    EdgeType::MemberOf,
                ),
                (
                    NodeName::Group {
                        name: "group4".to_owned(),
                        origin: Default::default(),
                    },
                    NodeName::Group {
                        name: "group1".to_owned(),
                        origin: Default::default(),
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
                origin: Default::default(),
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
                origin: Default::default(),
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
                origin: Default::default(),
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
                origin: Default::default(),
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
                origin: Default::default(),
            })
            .unwrap(),
            |_| true,
            |n| n.get_string_name() == *"::group2",
            None,
            None,
        );
        a.iter()
            .for_each(|p| crate::logging::debug!("{}", ag.path_as_string(p)));
        assert_eq!(a.len(), 2);

        Ok(())
    }
}
