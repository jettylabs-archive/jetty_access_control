//! # Access Graph
//!
//! `access_graph` is a library for modeling data access permissions and metadata as a graph.

mod graph;
mod helpers;

use crate::connectors::AssetType;

use super::connectors;
use std::collections::HashMap;
use std::collections::HashSet;

/// Attributes associated with a User node
#[derive(Debug)]
struct UserAttributes {
    name: String,
    identifiers: HashMap<connectors::UserIdentifier, String>,
    metadata: HashMap<String, String>,
    connectors: Vec<String>,
}

/// Attributes associated with a Group node
#[derive(Debug)]
struct GroupAttributes {
    name: String,
    metadata: HashMap<String, String>,
    connectors: Vec<String>,
}

#[derive(Debug)]
struct AssetAttributes {
    name: String,
    asset_type: AssetType,
    metadata: HashMap<String, String>,
    connectors: Vec<String>,
}

/// Enum of node types
#[derive(Debug)]
pub(crate) enum JettyNode {
    Group(GroupAttributes),
    User(UserAttributes),
    Asset(AssetAttributes),
}

/// Enum of edge types
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
enum EdgeType {
    MemberOf,
    Includes,
    GrantedBy,
    ChildOf,
    ParentOf,
    DerivedFrom,
    DerivedTo,
    TaggedAs,
    GovernedBy,
    AppliedTo,
    Governs,
    GrantedTo,
}

fn get_edge_type_pair(edge_type: &EdgeType) -> EdgeType {
    match edge_type {
        EdgeType::MemberOf => EdgeType::Includes,
        EdgeType::Includes => EdgeType::MemberOf,
        EdgeType::GrantedBy => EdgeType::GrantedTo,
        EdgeType::GrantedTo => EdgeType::GrantedBy,
        EdgeType::ChildOf => EdgeType::ParentOf,
        EdgeType::ParentOf => EdgeType::ChildOf,
        EdgeType::DerivedFrom => EdgeType::DerivedTo,
        EdgeType::DerivedTo => EdgeType::DerivedFrom,
        EdgeType::TaggedAs => EdgeType::AppliedTo,
        EdgeType::AppliedTo => EdgeType::TaggedAs,
        EdgeType::GovernedBy => EdgeType::Governs,
        EdgeType::Governs => EdgeType::GovernedBy,
    }
}

/// Mapping of node identifiers (like asset name) to their id in the graph
#[derive(PartialEq, Eq, Hash, Clone)]
enum NodeName {
    User(String),
    Group(String),
    Asset(String),
    Policy(String),
    Tag(String),
}

#[derive(Hash, Eq, PartialEq)]
pub(crate) struct JettyEdge {
    from: NodeName,
    to: NodeName,
    edge_type: EdgeType,
}

/// Representation of data access state
pub struct AccessGraph {
    /// The graph itself
    graph: graph::Graph,
    edge_cache: HashSet<JettyEdge>,
}

impl AccessGraph {}
