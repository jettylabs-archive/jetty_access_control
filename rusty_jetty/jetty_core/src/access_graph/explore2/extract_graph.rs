//! Utilities to return only part of a graph
//!

use std::collections::HashMap;

use petgraph::{stable_graph::NodeIndex, visit::EdgeRef};

use super::SubGraph;
use crate::access_graph::{AccessGraph, EdgeType, JettyNode, NodeName};

impl AccessGraph {
    /// Extract the graph surrounding a node to max_depth
    pub fn extract_graph(&self, from: &NodeName, max_depth: usize) -> SubGraph {
        let idx = self.graph.nodes.get(from).unwrap();
        let mut final_graph: petgraph::graph::DiGraph<JettyNode, EdgeType> = petgraph::Graph::new();

        let new_idx = final_graph.add_node(self.graph.graph[*idx].to_owned());

        self.add_children(idx, &new_idx, max_depth, &mut final_graph);

        SubGraph(final_graph)
    }

    fn add_children(
        &self,
        source_idx: &NodeIndex,
        new_idx: &NodeIndex,
        max_depth: usize,
        graph: &mut petgraph::graph::DiGraph<JettyNode, EdgeType>,
    ) {
        let old_graph = &self.graph.graph;
        // if we've already gone deep enough, don't go any deeper
        if max_depth == 0 {
            return;
        }

        // Otherwise, get the children, insert them
        let neighbors = old_graph.neighbors_undirected(*source_idx);

        let mut old_new_map: HashMap<NodeIndex, NodeIndex> = HashMap::new();
        for o in neighbors.clone() {
            let w = &old_graph[o];
            if old_new_map.contains_key(&o) {
                continue;
            }
            let n = graph.add_node(w.to_owned());

            old_new_map.insert(o, n);
        }

        // And then insert edges to the new

        // outgoing edges
        let edges = old_graph.edges_directed(*source_idx, petgraph::Direction::Outgoing);
        for o in edges {
            graph.add_edge(
                *new_idx,
                *old_new_map.get(&o.target()).unwrap(),
                o.weight().to_owned(),
            );
        }
        // incoming edges
        let edges = old_graph.edges_directed(*source_idx, petgraph::Direction::Incoming);
        for o in edges {
            graph.add_edge(
                *old_new_map.get(&o.source()).unwrap(),
                *new_idx,
                o.weight().to_owned(),
            );
        }

        for n in neighbors {
            self.add_children(&n, old_new_map.get(&n).unwrap(), max_depth - 1, graph)
        }
    }
}
#[cfg(test)]
mod tests {

    use crate::access_graph::{GroupAttributes, UserAttributes};

    use anyhow::Result;
    use petgraph::algo::is_isomorphic_matching;

    use super::*;

    fn get_test_graph() -> AccessGraph {
        AccessGraph::new_dummy(
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
                (
                    NodeName::Group("group4".to_owned()),
                    NodeName::Group("group3".to_owned()),
                    EdgeType::Includes,
                ),
            ],
        )
    }

    #[test]
    fn extract_graph_works() -> Result<()> {
        let ag = get_test_graph();

        let sub_graph = AccessGraph::new_dummy(
            &[
                &JettyNode::Group(GroupAttributes::new("group2".to_owned())),
                &JettyNode::Group(GroupAttributes::new("group3".to_owned())),
                &JettyNode::Group(GroupAttributes::new("group4".to_owned())),
            ],
            &[
                (
                    NodeName::Group("group2".to_owned()),
                    NodeName::Group("group3".to_owned()),
                    EdgeType::MemberOf,
                ),
                (
                    NodeName::Group("group3".to_owned()),
                    NodeName::Group("group4".to_owned()),
                    EdgeType::MemberOf,
                ),
                (
                    NodeName::Group("group4".to_owned()),
                    NodeName::Group("group3".to_owned()),
                    EdgeType::Includes,
                ),
            ],
        );

        let SubGraph(extracted) = ag.extract_graph(&NodeName::Group("group3".to_owned()), 1);

        assert!(is_isomorphic_matching(
            &extracted,
            &Into::<petgraph::graph::DiGraph<JettyNode, EdgeType>>::into(sub_graph.graph.graph),
            |w1, w2| w1.get_node_name() == w2.get_node_name(),
            |e1, e2| e1 == e2
        ));

        Ok(())
    }
}
