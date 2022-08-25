//! Helpers to represent data on its way into the graph

use std::collections::HashSet;

use super::{
    connectors::nodes, AssetAttributes, EdgeType, GroupAttributes, JettyEdge, JettyNode, NodeName,
    UserAttributes,
};

/// All helper types implement NodeHelpers.
pub(crate) trait NodeHelper {
    /// Return a JettyNode from the helper
    fn get_node(&self) -> JettyNode;
    /// Return a set of JettyEdges from the helper
    fn get_edges(&self) -> HashSet<JettyEdge>;
}

/// Object used to populate group nodes and edges in the graph
#[derive(Default)]
pub(crate) struct Group {
    node: nodes::Group,
    connectors: Vec<String>,
}

impl NodeHelper for Group {
    fn get_node(&self) -> JettyNode {
        JettyNode::Group(GroupAttributes {
            name: self.node.name.to_owned(),
            metadata: self.node.metadata.to_owned(),
            connectors: self.connectors.to_owned(),
        })
    }

    fn get_edges(&self) -> HashSet<JettyEdge> {
        let mut hs = HashSet::<JettyEdge>::new();
        for v in &self.node.member_of {
            insert_edge_pair(
                &mut hs,
                NodeName::Group(self.node.name.to_owned()),
                NodeName::Group(v.to_owned()),
                EdgeType::MemberOf,
            );
        }
        for v in &self.node.includes_users {
            insert_edge_pair(
                &mut hs,
                NodeName::Group(self.node.name.to_owned()),
                NodeName::User(v.to_owned()),
                EdgeType::Includes,
            );
        }
        for v in &self.node.includes_groups {
            insert_edge_pair(
                &mut hs,
                NodeName::Group(self.node.name.to_owned()),
                NodeName::Group(v.to_owned()),
                EdgeType::Includes,
            );
        }
        for v in &self.node.granted_by {
            insert_edge_pair(
                &mut hs,
                NodeName::Group(self.node.name.to_owned()),
                NodeName::Group(v.to_owned()),
                EdgeType::GrantedBy,
            );
        }
        hs
    }
}

/// Object used to populate user nodes and edges in the graph
#[derive(Default)]
pub(crate) struct User {
    node: nodes::User,
    connectors: Vec<String>,
}

impl NodeHelper for User {
    fn get_node(&self) -> JettyNode {
        JettyNode::User(UserAttributes {
            name: self.node.name.to_owned(),
            identifiers: self.node.identifiers.to_owned(),
            metadata: self.node.metadata.to_owned(),
            connectors: self.connectors.to_owned(),
        })
    }

    fn get_edges(&self) -> HashSet<JettyEdge> {
        let mut hs = HashSet::<JettyEdge>::new();
        for v in &self.node.member_of {
            insert_edge_pair(
                &mut hs,
                NodeName::Group(self.node.name.to_owned()),
                NodeName::Group(v.to_owned()),
                EdgeType::MemberOf,
            );
        }
        for v in &self.node.granted_by {
            insert_edge_pair(
                &mut hs,
                NodeName::Group(self.node.name.to_owned()),
                NodeName::Group(v.to_owned()),
                EdgeType::GrantedBy,
            );
        }
        hs
    }
}

/// Object used to populate asset nodes and edges in the graph
#[derive(Default)]
pub(crate) struct Asset {
    node: nodes::Asset,
    connectors: Vec<String>,
}

impl NodeHelper for Asset {
    fn get_node(&self) -> JettyNode {
        JettyNode::Asset(AssetAttributes {
            name: self.node.name.to_owned(),
            asset_type: self.node.asset_type.to_owned(),
            metadata: self.node.metadata.to_owned(),
            connectors: self.connectors.to_owned(),
        })
    }

    fn get_edges(&self) -> HashSet<JettyEdge> {
        let mut hs = HashSet::<JettyEdge>::new();
        for v in &self.node.governed_by {
            insert_edge_pair(
                &mut hs,
                NodeName::Group(self.node.name.to_owned()),
                NodeName::Group(v.to_owned()),
                EdgeType::GovernedBy,
            );
        }
        for v in &self.node.child_of {
            insert_edge_pair(
                &mut hs,
                NodeName::Group(self.node.name.to_owned()),
                NodeName::Group(v.to_owned()),
                EdgeType::ChildOf,
            );
        }
        for v in &self.node.parent_of {
            insert_edge_pair(
                &mut hs,
                NodeName::Group(self.node.name.to_owned()),
                NodeName::Group(v.to_owned()),
                EdgeType::ParentOf,
            );
        }
        for v in &self.node.derived_from {
            insert_edge_pair(
                &mut hs,
                NodeName::Group(self.node.name.to_owned()),
                NodeName::Group(v.to_owned()),
                EdgeType::DerivedFrom,
            );
        }
        for v in &self.node.derived_to {
            insert_edge_pair(
                &mut hs,
                NodeName::Group(self.node.name.to_owned()),
                NodeName::Group(v.to_owned()),
                EdgeType::DerivedTo,
            );
        }
        for v in &self.node.tagged_as {
            insert_edge_pair(
                &mut hs,
                NodeName::Group(self.node.name.to_owned()),
                NodeName::Group(v.to_owned()),
                EdgeType::TaggedAs,
            );
        }
        hs
    }
}

/// Object used to populate tag nodes and edges in the graph
#[derive(Debug)]
pub(crate) struct Tag {
    node: nodes::Tag,
    connectors: Vec<String>,
}

/// Object used to populate policy nodes and edges in the graph
#[derive(Debug)]
pub(crate) struct Policy {
    node: nodes::Policy,
    connectors: Vec<String>,
}

fn insert_edge_pair(
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
        from: to.to_owned(),
        to: from.to_owned(),
        edge_type: super::get_edge_type_pair(&edge_type),
    });
}
