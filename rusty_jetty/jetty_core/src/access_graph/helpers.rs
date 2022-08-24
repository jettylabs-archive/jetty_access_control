//! Helpers to represent data on its way into the graph

use std::collections::HashMap;

use super::{connectors, connectors::nodes, AccessGraph};
use anyhow::{Ok, Result};
/// All helper types implement NodeHelpers.
pub trait NodeHelper {
    /// Register construct or update a node in the graph and
    /// stash the required edges in the edge cache
    fn register(&self, g: AccessGraph) -> Result<()>;
}

/// Object used to populate group nodes and edges in the graph
#[derive(Default)]
pub struct Group {
    pub node: nodes::Group,
    pub connectors: Vec<String>,
}

impl NodeHelper for Group {
    fn register(&self, g: AccessGraph) -> Result<()> {
        Ok(())
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
