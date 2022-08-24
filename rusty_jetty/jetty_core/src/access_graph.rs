//! # Access Graph
//!
//! `access_graph` is a library for modeling data access permissions and metadata as a graph.

mod helpers;

use super::connectors;
use std::collections::HashMap;

use anyhow::{anyhow, Context, Result, *};

use graphviz_rust as graphviz;
use graphviz_rust::cmd::CommandArg;
use graphviz_rust::cmd::Format;
use graphviz_rust::printer::PrinterContext;
use petgraph::dot;
use petgraph::stable_graph::StableDiGraph;

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
#[derive(Debug)]
enum JettyEdge {
    MemberOf,
    Includes,
}

/// Mapping of node identifiers (like asset name) to their id in the graph
enum NodeID {
    User(String),
    Group(String),
    Asset(String),
    Policy(String),
    Tag(String),
}

/// Representation of data access state
pub struct AccessGraph {
    /// The graph itself
    graph: StableDiGraph<JettyNode, JettyEdge>,
    /// A map of node identifiers to indecies
    nodes: HashMap<NodeID, u32>,
}

impl AccessGraph {
    /// Save a svg of the access graph to the specified filename
    pub fn visualize(&self, path: String) -> Result<String> {
        let my_dot = dot::Dot::new(&self.graph);
        let g = graphviz::parse(&format!["{:?}", my_dot]).map_err(|s| anyhow!(s))?;
        let draw = graphviz::exec(
            g,
            &mut PrinterContext::default(),
            vec![CommandArg::Format(Format::Svg), CommandArg::Output(path)],
        )?;
        Ok(draw)
    }
}
