//! Helpers to represent data on its way into the graph

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use super::{
    connectors, connectors::nodes, AccessGraph, EdgeType, GroupAttributes, JettyEdge, JettyNode,
    NodeName,
};
use anyhow::Result;
/// All helper types implement NodeHelpers.
pub trait NodeHelper {
    /// Return a JettyNode from the helper
    fn get_node(&self) -> JettyNode;
    /// Return a set of JettyEdges from the helper
    fn get_edges(&self) -> HashSet<JettyEdge>;
}

/// Object used to populate group nodes and edges in the graph
#[derive(Default)]
pub struct Group {
    pub node: nodes::Group,
    pub connectors: Vec<String>,
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
            hs.insert(JettyEdge {
                from: NodeName::Group(self.node.name.to_owned()),
                to: NodeName::Group(v.to_owned()),
                edge_type: EdgeType::MemberOf,
            });
        }
        hs
    }
}

/// Object used to populate user nodes and edges in the graph
#[derive(Default)]
pub struct User {
    node: nodes::User,
    connectors: Vec<String>,
}

/// Object used to populate asset nodes and edges in the graph
#[derive(Default)]
pub struct Asset {
    node: nodes::Asset,
    connectors: Vec<String>,
}

/// Object used to populate tag nodes and edges in the graph
#[derive(Debug)]
pub struct Tag {
    node: nodes::Tag,
    connectors: Vec<String>,
}

/// Object used to populate policy nodes and edges in the graph
#[derive(Debug)]
pub struct Policy {
    name: nodes::Policy,
    connectors: Vec<String>,
}
