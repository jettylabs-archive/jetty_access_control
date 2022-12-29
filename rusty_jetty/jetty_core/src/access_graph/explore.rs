//! Utilities for exploration of the graph.
//!

mod accessible_assets;
mod asset_paths_for_tag;
mod default_policy_targets;
mod extract_graph;
mod get_node;
mod matching_descendants;
mod matching_paths;
mod matching_paths_to_descendants;
mod tags_for_asset;
mod user_accessible_tags;

use petgraph::{stable_graph::NodeIndex, visit::IntoNodeReferences};

use crate::permissions::matrix::Merge;

use super::{AccessGraph, EdgeType, JettyNode};
pub use tags_for_asset::AssetTags;

/// A path from one node to another, including start and end nodes.
/// Inside, it's a Vec<JettyNode>
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct NodePath(Vec<NodeIndex>);

impl Merge for NodePath {}

/// A DiGraph derived from an AccessGraph
pub struct SubGraph(petgraph::graph::DiGraph<JettyNode, EdgeType>);

impl SubGraph {
    /// return the dot graph representation of a SubGraph
    pub fn dot(&self) -> petgraph::dot::Dot<&petgraph::Graph<JettyNode, EdgeType>> {
        petgraph::dot::Dot::new(&self.0)
    }
}

impl AccessGraph {
    /// Get all nodes from the graph
    pub fn get_nodes(&self) -> petgraph::stable_graph::NodeReferences<super::JettyNode> {
        self.graph().node_references()
    }

    /// Get a node path as a string
    pub fn path_as_string(&self, path: &NodePath) -> String {
        path.0
            .iter()
            .map(|idx| self[*idx].get_string_name())
            .collect::<Vec<_>>()
            .join(" ⇨ ")
    }

    /// Get a node path as a vector of JettyNodes
    pub fn path_as_jetty_nodes(&self, NodePath(path): &NodePath) -> Vec<&JettyNode> {
        path.iter().map(|&idx| &self[idx]).collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        access_graph::{
            cual_to_asset_name_test, AssetAttributes, GroupAttributes, NodeName, PolicyAttributes,
            UserAttributes,
        },
        cual::Cual,
        logging::debug,
    };

    use anyhow::Result;

    use super::*;

    #[test]
    fn get_matching_children_works() -> Result<()> {
        let ag = AccessGraph::new_dummy(
            &[
                &JettyNode::Asset(AssetAttributes::new(
                    Cual::new("mycual://a/a"),
                    Default::default(),
                )),
                &JettyNode::Policy(PolicyAttributes::new("policy".to_owned())),
                &JettyNode::User(UserAttributes::simple_new("user".to_owned())),
            ],
            &[
                (
                    NodeName::User("user".to_owned()),
                    NodeName::Policy {
                        name: "policy".to_owned(),
                        origin: Default::default(),
                    },
                    EdgeType::GrantedFrom,
                ),
                (
                    NodeName::Policy {
                        name: "policy".to_owned(),
                        origin: Default::default(),
                    },
                    cual_to_asset_name_test(Cual::new("mycual://a/a"), Default::default()),
                    EdgeType::Governs,
                ),
            ],
        );

        // Test Edge Matching
        let a = ag.get_matching_descendants(
            ag.get_untyped_index_from_name(&NodeName::User("user".to_owned()))
                .unwrap(),
            |n| matches!(n, EdgeType::MemberOf),
            |_| true,
            |_| true,
            None,
            None,
        );
        assert_eq!(a.len(), 0);

        // Test getting all children
        let a = ag.get_matching_descendants(
            ag.get_untyped_index_from_name(&NodeName::User("user".to_owned()))
                .unwrap(),
            |_| true,
            |_| true,
            |_| true,
            None,
            None,
        );
        assert_eq!(a.len(), 3);

        // Test target matching
        let a = ag.get_matching_descendants(
            ag.get_untyped_index_from_name(&NodeName::User("user".to_owned()))
                .unwrap(),
            |_| true,
            |_| true,
            |n| matches!(n, JettyNode::Asset(_)),
            None,
            None,
        );
        assert_eq!(a.len(), 1);

        // Test passthrough matching
        let a = ag.get_matching_descendants(
            ag.get_untyped_index_from_name(&NodeName::User("user".to_owned()))
                .unwrap(),
            |_| true,
            |n| matches!(n, JettyNode::Policy(_)),
            |n| matches!(n, JettyNode::Asset(_)),
            None,
            None,
        );
        assert_eq!(a.len(), 1);

        let a = ag.get_matching_descendants(
            ag.get_untyped_index_from_name(&NodeName::User("user".to_owned()))
                .unwrap(),
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
        a.iter().for_each(|p| debug!("{}", ag.path_as_string(p)));
        assert_eq!(a.len(), 2);

        Ok(())
    }
}
