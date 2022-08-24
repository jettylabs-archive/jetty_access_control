//! Helpers to represent data on its way into the graph

use std::collections::HashMap;

use super::{connectors, AccessGraph};
use anyhow::{Ok, Result};
use derivative::Derivative;
/// All helper types implement NodeHelpers.
pub trait NodeHelper {
    /// Register construct or update a node in the graph and
    /// stash the required edges in the edge cache
    fn register(&self, g: AccessGraph) -> Result<()>;
}

/// Object used to populate group nodes and edges in the graph
#[derive(Default)]
pub struct Group {
    pub name: String,
    pub metadata: HashMap<String, String>,
    pub member_of: Vec<String>,
    pub includes: Vec<String>,
    pub granted_by: Vec<String>,
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
    name: String,
    identifiers: HashMap<connectors::UserIdentifier, String>,
    metadata: HashMap<String, String>,
    member_of: Vec<String>,
    granted_by: Vec<String>,
    connectors: Vec<String>,
}

/// Object used to populate asset nodes and edges in the graph
#[derive(Default)]
pub struct Asset {
    name: String,
    asset_type: connectors::AssetType,
    metadata: HashMap<String, String>,
    governed_by: Vec<String>,
    child_of: Vec<String>,
    parent_of: Vec<String>,
    derived_from: Vec<String>,
    derived_to: Vec<String>,
    tagged_as: Vec<String>,
    connectors: Vec<String>,
}

/// Object used to populate tag nodes and edges in the graph
#[derive(Debug, Derivative)]
#[derivative(Default)]
pub struct Tag {
    name: String,
    value: Option<String>,
    #[derivative(Default(value = "true"))]
    pass_through_hierarchy: bool,
    pass_through_lineage: bool,
    applied_to: Vec<String>,
    connectors: Vec<String>,
    governed_by: Vec<String>,
}

/// Object used to populate policy nodes and edges in the graph
#[derive(Debug, Derivative)]
#[derivative(Default)]
pub struct Policy {
    name: String,
    priveleges: Vec<String>,
    governs_assets: Vec<String>,
    governs_tags: Vec<String>,
    grants_to_groups: Vec<String>,
    grants_to_users: Vec<String>,
    pass_through_hierarchy: bool,
    #[derivative(Default(value = "true"))]
    pass_through_lineage: bool,
    connectors: Vec<String>,
}
