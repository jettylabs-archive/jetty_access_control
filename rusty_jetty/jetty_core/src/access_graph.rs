//! # Access Graph
//!
//! `access_graph` is a library for modeling data access permissions and metadata as a graph.

mod helpers;

use std::collections::HashMap;

use anyhow;

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
struct NodeMap {
    users: HashMap<String, usize>,
    groups: HashMap<String, usize>,
    assets: HashMap<String, usize>,
    policies: HashMap<String, usize>,
    tags: HashMap<String, usize>,
}

/// Representation of data access state
pub struct AccessGraph {
    /// The graph itself
    graph: StableDiGraph<JettyNode, JettyEdge>,
    /// A map of node identifiers to indecies
    nodes: NodeMap,
}

impl AccessGraph {
    /// Save a svg of the access graph to the specified filename
    pub fn visualize(&self, path: String) -> anyhow::Result<String> {
        let my_dot = dot::Dot::new(&self.graph);
        let g = graphviz::parse(&format!["{:?}", my_dot]).unwrap();
        let draw = graphviz::exec(
            g,
            &mut PrinterContext::default(),
            vec![CommandArg::Format(Format::Svg), CommandArg::Output(path)],
        )?;
        Ok(draw)
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
