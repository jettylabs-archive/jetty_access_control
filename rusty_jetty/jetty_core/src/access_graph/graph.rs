//! Graph stuff
//!

use anyhow::{anyhow, Context, Result};
use graphviz_rust as graphviz;
use graphviz_rust::cmd::CommandArg;
use graphviz_rust::cmd::Format;
use graphviz_rust::printer::PrinterContext;
use petgraph::stable_graph::NodeIndex;
use petgraph::{dot, stable_graph::StableDiGraph};
use std::collections::HashMap;

use super::{EdgeType, JettyNode, NodeName};

/// The main graph wrapper
pub struct Graph {
    pub(crate) graph: StableDiGraph<JettyNode, EdgeType>,
    /// A map of node identifiers to indecies
    pub(crate) nodes: HashMap<NodeName, NodeIndex>,
}

impl Graph {
    /// Save a svg of the access graph to the specified filename
    pub fn visualize(&self, path: String) -> Result<String> {
        let my_dot = dot::Dot::new(&self.graph);
        let g = graphviz::parse(&format!["{:?}", my_dot])
            .map_err(|s| anyhow!(s))
            .context("failed to parse")?;
        let draw = graphviz::exec(
            g,
            &mut PrinterContext::default(),
            vec![CommandArg::Format(Format::Svg), CommandArg::Output(path)],
        )
        .context("failed to exec graphviz. do you need to install it?")?;
        Ok(draw)
    }
    /// Check whether a given node already exists in the graph
    pub fn get_node(&self, node: &NodeName) -> Option<&NodeIndex> {
        self.nodes.get(node)
    }
    /// Adds a node to the graph and returns the index.
    pub(crate) fn add_node(&mut self, node: &JettyNode) -> Result<NodeIndex> {
        let node_name = node.get_name();
        let idx = self.graph.add_node(node.to_owned());
        self.nodes.insert(node_name, idx);
        Ok(idx)
    }

    /// Updates a node. Should return the updated node. Returns an
    /// error if the nodes are incompatible (would require overwriting values).
    /// To be compatible, metadata from each
    pub(crate) fn merge_nodes(&mut self, idx: &NodeIndex, new: &JettyNode) -> Result<()> {
        // Fetch node from graph
        let node = &mut self.graph[*idx];

        *node = node
            .merge_nodes(new)
            .context(format!["merging: {:?}, {:?}", node, new])?;
        Ok(())
    }

    /// Add edges from cache. Return an error if to/from doesn't exist
    pub(crate) fn add_edge(&mut self, edge: super::JettyEdge) -> Result<()> {
        let to = self.get_node(&edge.to);
        if let None = to {
            return Err(anyhow![
                "Unable to find \"to\" node: {:?} for \"from\" {:?}",
                &edge.to,
                &edge.from
            ]);
        }
        let from = self.get_node(&edge.from);
        if let None = from {
            return Err(anyhow![
                "Unable to find \"from\" node: {:?} for \"to\" {:?}",
                &edge.from,
                &edge.to
            ]);
        }

        self.graph
            .add_edge(*from.unwrap(), *to.unwrap(), edge.edge_type);
        Ok(())
    }
}
