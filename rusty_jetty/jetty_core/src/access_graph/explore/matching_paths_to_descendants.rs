//! Utilities for exploration of the graph.
//!

use std::collections::{HashMap, HashSet};

use indexmap::IndexSet;
use petgraph::{stable_graph::NodeIndex, Direction};

use super::{AccessGraph, EdgeType, JettyNode, NodePath};

impl AccessGraph {
    /// Return the descendent node and matching paths from a provided node to all of its matching descendants.
    /// Specify filter functions to match edges and passthrough nodes.
    pub fn all_matching_simple_paths_to_descendants<T: Into<NodeIndex>>(
        &self,
        from: T,
        edge_matcher: fn(&EdgeType) -> bool,
        passthrough_matcher: fn(&JettyNode) -> bool,
        target_matcher: fn(&JettyNode) -> bool,
        min_depth: Option<usize>,
        max_depth: Option<usize>,
    ) -> HashMap<NodeIndex, HashSet<NodePath>> {
        let from_idx: NodeIndex = from.into();

        let max_depth = if let Some(l) = max_depth {
            l
        } else {
            self.graph.graph.node_count() - 1
        };

        let min_depth = min_depth.unwrap_or(0);

        // list of visited nodes
        let mut visited = IndexSet::from([(from_idx.to_owned())]);
        let mut results = HashMap::new();

        self.all_matching_simple_paths_to_descendants_recursive(
            from_idx,
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

    /// Returns a Vec of Vec<JettyNodes> representing the matching non-cyclic paths
    /// between two nodes
    #[allow(clippy::too_many_arguments)]
    fn all_matching_simple_paths_to_descendants_recursive(
        &self,
        from_idx: NodeIndex,
        edge_matcher: fn(&EdgeType) -> bool,
        passthrough_matcher: fn(&JettyNode) -> bool,
        target_matcher: fn(&JettyNode) -> bool,
        min_depth: usize,
        max_depth: usize,
        current_depth: usize,
        visited: &mut IndexSet<NodeIndex>,
        results: &mut HashMap<NodeIndex, HashSet<NodePath>>,
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
            // Has it already been inserted/visited?
            if !visited.insert(child) {
                continue;
            }

            // Get the node we're looking at
            let node_weight = &self[child];

            // Are we beyond the minimum depth?
            if current_depth >= min_depth {
                // is it the target node? if so, add the path to the results
                if target_matcher(node_weight) {
                    let path = visited.iter().cloned().collect::<Vec<_>>();
                    let x = results.get_mut(&child);
                    match x {
                        Some(p) => {
                            p.insert(NodePath(path));
                        }
                        None => {
                            results.insert(child, HashSet::from([NodePath(path)]));
                        }
                    };
                }
            }
            // Is it a passthrough type?
            if passthrough_matcher(node_weight) {
                self.all_matching_simple_paths_to_descendants_recursive(
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
            visited.pop();
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::access_graph::{GroupAttributes, NodeName, UserAttributes};

    use anyhow::{anyhow, Result};

    use super::*;

    fn get_test_graph() -> AccessGraph {
        AccessGraph::new_dummy(
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
        )
    }

    #[test]
    fn multiple_paths_to_same_node_works() -> Result<()> {
        let ag = get_test_graph();
        let from_index = ag
            .get_untyped_index_from_name(&NodeName::User("user".to_owned()))
            .ok_or(anyhow!("unable to find matching node"))?;

        // Test getting multiple paths to the same node
        let a = ag.all_matching_simple_paths_to_descendants(
            from_index,
            |_| true,
            |_| true,
            |n| n.get_string_name() == *"::group4",
            None,
            None,
        );
        a.iter().for_each(|(_, p)| {
            p.iter()
                .for_each(|q| crate::logging::debug!("{}", ag.path_as_string(q)))
        });
        assert_eq!(a.len(), 1);
        assert_eq!(a.values().next().map(|v| v.len()), Some(6));

        Ok(())
    }

    #[test]
    fn gets_all_children() -> Result<()> {
        let ag = get_test_graph();
        let from_index = ag
            .get_untyped_index_from_name(&NodeName::User("user".to_owned()))
            .ok_or(anyhow!("unable to find matching node"))?;

        // Test getting multiple paths to the same node
        let a = ag.all_matching_simple_paths_to_descendants(
            from_index,
            |_| true,
            |_| true,
            |_| true,
            None,
            None,
        );
        assert_eq!(a.len(), 4);

        Ok(())
    }
}
