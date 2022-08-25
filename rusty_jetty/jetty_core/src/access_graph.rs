//! # Access Graph
//!
//! `access_graph` is a library for modeling data access permissions and metadata as a graph.

mod graph;
mod helpers;

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

/// Enum of node types
#[derive(Debug)]
enum JettyNode {
    Group(GroupAttributes),
    User(UserAttributes),
}

/// Enum of edge types
#[derive(PartialEq, Eq, Hash, Debug)]
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

/// Mapping of node identifiers (like asset name) to their id in the graph
#[derive(PartialEq, Eq, Hash)]
enum NodeName {
    User(String),
    Group(String),
    Asset(String),
    Policy(String),
    Tag(String),
}

#[derive(Hash, Eq, PartialEq)]
struct JettyEdge {
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
