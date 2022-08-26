//! # Access Graph
//!
//! `access_graph` is a library for modeling data access permissions and metadata as a graph.

mod graph;
mod helpers;

use crate::connectors::AssetType;

use self::helpers::NodeHelper;
use self::helpers::ProcessedConnectorData;

use super::connectors;
use core::hash::Hash;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;

use anyhow::{anyhow, Context, Result};

/// Attributes associated with a User node

#[derive(Debug, Clone)]
pub(crate) struct UserAttributes {
    name: String,
    identifiers: HashMap<connectors::UserIdentifier, String>,
    other_identifiers: HashSet<String>,
    metadata: HashMap<String, String>,
    connectors: HashSet<String>,
}

impl UserAttributes {
    fn merge_attributes(&self, new_attributes: &UserAttributes) -> Result<UserAttributes> {
        let name = merge_matched_field(&self.name, &new_attributes.name)
            .context("field: UserAttributes.name")?;
        let identifiers = merge_map(&self.identifiers, &new_attributes.identifiers)
            .context("field: UserAttributes.identifiers")?;
        let other_identifiers =
            merge_set(&self.other_identifiers, &new_attributes.other_identifiers);
        let metadata = merge_map(&self.metadata, &new_attributes.metadata)
            .context("field: UserAttributes.metadata")?;
        let connectors = merge_set(&self.connectors, &new_attributes.connectors);
        Ok(UserAttributes {
            name,
            identifiers,
            other_identifiers,
            metadata,
            connectors,
        })
    }
}

/// Attributes associated with a Group node

#[derive(Debug, Clone)]
pub(crate) struct GroupAttributes {
    name: String,
    metadata: HashMap<String, String>,
    connectors: HashSet<String>,
}

impl GroupAttributes {
    fn merge_attributes(&self, new_attributes: &GroupAttributes) -> Result<GroupAttributes> {
        let name = merge_matched_field(&self.name, &new_attributes.name)
            .context("field: GroupAttributes.name")?;
        let metadata = merge_map(&self.metadata, &new_attributes.metadata)
            .context("field: GroupAttributes.metadata")?;
        let connectors = merge_set(&self.connectors, &new_attributes.connectors);
        Ok(GroupAttributes {
            name,
            metadata,
            connectors,
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct AssetAttributes {
    name: String,
    asset_type: AssetType,
    metadata: HashMap<String, String>,
    connectors: HashSet<String>,
}

impl AssetAttributes {
    fn merge_attributes(&self, new_attributes: &AssetAttributes) -> Result<AssetAttributes> {
        let name = merge_matched_field(&self.name, &new_attributes.name)
            .context("field: AssetAttributes.name")?;
        let asset_type = merge_matched_field(&self.asset_type, &self.asset_type)
            .context("field: AssetAttributes.asset_type")?;
        let metadata = merge_map(&self.metadata, &new_attributes.metadata)
            .context("field: AssetAttributes.metadata")?;
        let connectors = merge_set(&self.connectors, &new_attributes.connectors);
        Ok(AssetAttributes {
            name,
            asset_type,
            metadata,
            connectors,
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TagAttributes {
    name: String,
    value: Option<String>,
    pass_through_hierarchy: bool,
    pass_through_lineage: bool,
    connectors: HashSet<String>,
}

impl TagAttributes {
    fn merge_attributes(&self, new_attributes: &TagAttributes) -> Result<TagAttributes> {
        let name = merge_matched_field(&self.name, &new_attributes.name)
            .context("field: TagAttributes.name")?;
        let value = merge_matched_field(&self.value, &new_attributes.value)
            .context("field: TagAttributes.value")?;
        let pass_through_hierarchy = merge_matched_field(
            &self.pass_through_hierarchy,
            &new_attributes.pass_through_hierarchy,
        )
        .context("field: TagAttributes.pass_through_hierarchy")?;
        let pass_through_lineage = merge_matched_field(
            &self.pass_through_lineage,
            &new_attributes.pass_through_lineage,
        )
        .context("field: TagAttributes.pass_through_lineage")?;

        let connectors = merge_set(&self.connectors, &new_attributes.connectors);
        Ok(TagAttributes {
            name,
            value,
            pass_through_hierarchy,
            pass_through_lineage,
            connectors,
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PolicyAttributes {
    name: String,
    privileges: HashSet<String>,
    pass_through_hierarchy: bool,
    pass_through_lineage: bool,
    connectors: HashSet<String>,
}

impl PolicyAttributes {
    fn merge_attributes(&self, new_attributes: &PolicyAttributes) -> Result<PolicyAttributes> {
        let name = merge_matched_field(&self.name, &new_attributes.name)
            .context("field: PolicyAttributes.name")?;
        let privileges = merge_matched_field(&self.privileges, &new_attributes.privileges)
            .context("field: PolicyAttributes.privileges")?;
        let pass_through_hierarchy = merge_matched_field(
            &self.pass_through_hierarchy,
            &new_attributes.pass_through_hierarchy,
        )
        .context("field: PolicyAttributes.pass_through_hierarchy")?;
        let pass_through_lineage = merge_matched_field(
            &self.pass_through_lineage,
            &new_attributes.pass_through_lineage,
        )
        .context("field: PolicyAttributes.pass_through_lineage")?;

        let connectors = merge_set(&self.connectors, &new_attributes.connectors);
        Ok(PolicyAttributes {
            name,
            privileges,
            pass_through_hierarchy,
            pass_through_lineage,
            connectors,
        })
    }
}

/// Enum of node types
#[derive(Debug, Clone)]
pub(crate) enum JettyNode {
    Group(GroupAttributes),
    User(UserAttributes),
    Asset(AssetAttributes),
    Tag(TagAttributes),
    Policy(PolicyAttributes),
}

impl JettyNode {
    fn merge_nodes(&self, new_node: &JettyNode) -> Result<JettyNode> {
        match (&self, new_node) {
            (JettyNode::Group(a1), JettyNode::Group(a2)) => {
                Ok(JettyNode::Group(a1.merge_attributes(a2)?))
            }
            (JettyNode::User(a1), JettyNode::User(a2)) => {
                Ok(JettyNode::User(a1.merge_attributes(a2)?))
            }
            (JettyNode::Asset(a1), JettyNode::Asset(a2)) => {
                Ok(JettyNode::Asset(a1.merge_attributes(a2)?))
            }
            (JettyNode::Tag(a1), JettyNode::Tag(a2)) => {
                Ok(JettyNode::Tag(a1.merge_attributes(a2)?))
            }
            (JettyNode::Policy(a1), JettyNode::Policy(a2)) => {
                Ok(JettyNode::Policy(a1.merge_attributes(a2)?))
            }
            (a, b) => Err(anyhow![
                "Unable to merge nodes of different types: {:?}, {:?}",
                a,
                b
            ]),
        }
    }

    /// Given a node, return the NodeName. This will return the name field
    /// wrapped in the appropriate enum.
    fn get_name(&self) -> NodeName {
        match &self {
            JettyNode::Asset(a) => NodeName::Asset(a.name.to_owned()),
            JettyNode::Group(a) => NodeName::Group(a.name.to_owned()),
            JettyNode::Policy(a) => NodeName::Policy(a.name.to_owned()),
            JettyNode::Tag(a) => NodeName::Tag(a.name.to_owned()),
            JettyNode::User(a) => NodeName::User(a.name.to_owned()),
        }
    }
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
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
enum NodeName {
    User(String),
    Group(String),
    Asset(String),
    Policy(String),
    Tag(String),
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
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
    last_modified: usize,
}

impl AccessGraph {
    pub(crate) fn build_graph(&mut self, data: ProcessedConnectorData) -> Result<()> {
        self.get_node_and_edges(&data.data.groups, &data.connector)?;
        self.get_node_and_edges(&data.data.users, &data.connector)?;
        self.get_node_and_edges(&data.data.assets, &data.connector)?;
        self.get_node_and_edges(&data.data.policies, &data.connector)?;
        self.get_node_and_edges(&data.data.tags, &data.connector)?;
        for edge in &self.edge_cache {
            self.graph.add_edge(edge.to_owned())?;
        }
        Ok(())
    }

    fn get_node_and_edges<T: NodeHelper>(
        &mut self,
        nodes: &Vec<T>,
        connector: &String,
    ) -> Result<()> {
        for n in nodes {
            let node = n.get_node(connector.to_owned());
            self.graph.add_node(&node)?;
            let edges = n.get_edges();
            self.edge_cache.extend(edges);
        }
        Ok(())
    }
}

fn merge_set(s1: &HashSet<String>, s2: &HashSet<String>) -> HashSet<String> {
    let mut s1 = s1.to_owned();
    let s2 = s2.to_owned();
    s1.extend(s2);
    s1
}
fn merge_matched_field<T>(s1: &T, s2: &T) -> Result<T>
where
    T: std::cmp::PartialEq + std::cmp::Eq + Debug + Clone,
{
    if s1 != s2 {
        return Err(anyhow![
            "unable to merge: fields don't match: {:?}, {:?}",
            s1,
            s2
        ]);
    }
    Ok(s1.to_owned())
}

fn merge_map<K, V>(m1: &HashMap<K, V>, m2: &HashMap<K, V>) -> Result<HashMap<K, V>>
where
    K: Debug + Clone + Hash + std::cmp::Eq,
    V: Debug + Clone + std::cmp::PartialEq,
{
    for (k, v) in m2 {
        if let Some(w) = m1.get(k) {
            if w != v {
                return Err(anyhow![
                    "unable to merge: conflicting data on key {:?}: {:?}, {:?}",
                    k,
                    w,
                    v
                ]);
            }
        }
    }

    let mut new_map = m1.to_owned();
    let new_m2 = m2.to_owned();

    new_map.extend(new_m2);

    Ok(new_map)
}
