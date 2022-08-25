use std::collections::HashMap;
use std::net::UdpSocket;

use anyhow::{anyhow, Context, Result};
use graphviz_rust as graphviz;
use graphviz_rust::cmd::CommandArg;
use graphviz_rust::cmd::Format;
use graphviz_rust::printer::PrinterContext;
use petgraph::stable_graph::NodeIndex;
use petgraph::{dot, stable_graph::StableDiGraph};

use super::{EdgeType, JettyNode, NodeName};

pub struct Graph {
    graph: StableDiGraph<JettyNode, EdgeType>,
    /// A map of node identifiers to indecies
    nodes: HashMap<NodeName, u32>,
}

impl Graph {
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
    /// Check whether a given node already exists in the graph
    pub fn node_exists(&self, node: NodeName) -> Option<u32> {
        Some(10)
    }
    /// Adds a node to the graph. Maybe Make this generic. What is the input?
    pub fn add_node(&self, node: JettyNode) -> Result<()> {
        return Ok(());
    }

    /// Updates a node. Should return the updated node. Returns an
    /// error if the nodes are incompatible (would require overwriting values).
    /// To be compatible, metadata from each
    pub fn merge_nodes(&mut self, idx: u32, new: &JettyNode) -> Result<()> {
        // Fetch node from graph
        let idx: usize = idx.try_into().context("convert node index to usize")?;
        let node = &mut self.graph[NodeIndex::new(idx)];

        *node = node
            .merge_nodes(new)
            .context(format!["merging: {:?}, {:?}", node, new])?;
        Ok(())
    }

    /// Add edges from cache. Return an error if to/from doesn't exist
    pub fn add_edge(&self, edge: super::JettyEdge) -> Result<()> {
        Ok(())
    }
}
