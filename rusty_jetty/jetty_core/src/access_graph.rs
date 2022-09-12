//! # Access Graph
//!
//! `access_graph` is a library for modeling data access permissions and metadata as a graph.

pub mod graph;
mod helpers;

use crate::{connectors::AssetType, cual::Cual};

use self::helpers::NodeHelper;
pub use self::helpers::ProcessedConnectorData;

use super::connectors;
use core::hash::Hash;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;

use anyhow::{anyhow, Context, Result};

/// Attributes associated with a User node

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AssetAttributes {
    cual: Cual,
    name: String,
    asset_type: AssetType,
    metadata: HashMap<String, String>,
    connectors: HashSet<String>,
}

impl AssetAttributes {
    fn merge_attributes(&self, new_attributes: &AssetAttributes) -> Result<AssetAttributes> {
        let name = merge_matched_field(&self.name, &new_attributes.name)
            .context("field: AssetAttributes.name")?;
        let cual = merge_matched_field(&self.cual, &new_attributes.cual)
            .context("field: GroupAttributes.cual")?;
        let asset_type = merge_matched_field(&self.asset_type, &self.asset_type)
            .context("field: AssetAttributes.asset_type")?;
        let metadata = merge_map(&self.metadata, &new_attributes.metadata)
            .context("field: AssetAttributes.metadata")?;
        let connectors = merge_set(&self.connectors, &new_attributes.connectors);
        Ok(AssetAttributes {
            cual,
            name,
            asset_type,
            metadata,
            connectors,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum JettyNode {
    /// Group node
    Group(GroupAttributes),
    /// User node
    User(UserAttributes),
    /// Asset node
    Asset(AssetAttributes),
    /// Tag node
    Tag(TagAttributes),
    /// Policy node
    Policy(PolicyAttributes),
}

impl JettyNode {
    #[allow(dead_code)]
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
            JettyNode::Asset(a) => NodeName::Asset(a.cual.uri().to_owned()),
            JettyNode::Group(a) => NodeName::Group(a.name.to_owned()),
            JettyNode::Policy(a) => NodeName::Policy(a.name.to_owned()),
            JettyNode::Tag(a) => NodeName::Tag(a.name.to_owned()),
            JettyNode::User(a) => NodeName::User(a.name.to_owned()),
        }
    }
}

/// Enum of edge types
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub(crate) enum EdgeType {
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
pub enum NodeName {
    /// User node
    User(String),
    /// Group node
    Group(String),
    /// Asset node
    Asset(String),
    /// Policy node
    Policy(String),
    /// Tag node
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
    pub graph: graph::Graph,
    edge_cache: HashSet<JettyEdge>,
    #[allow(dead_code)]
    last_modified: usize,
}

impl AccessGraph {
    /// New graph
    pub fn new(data: Vec<ProcessedConnectorData>) -> Result<Self> {
        let mut ag = AccessGraph {
            graph: graph::Graph {
                graph: petgraph::stable_graph::StableDiGraph::new(),
                nodes: HashMap::new(),
            },
            edge_cache: HashSet::new(),
            last_modified: 0,
        };
        for connector_data in data {
            // ag.build_graph(connector_data)?;
            ag.add_nodes(&connector_data)?;
            ag.add_edges()?;
        }
        Ok(ag)
    }

    pub(crate) fn add_nodes(&mut self, data: &ProcessedConnectorData) -> Result<()> {
        self.register_nodes_and_edges(&data.data.groups, &data.connector)?;
        self.register_nodes_and_edges(&data.data.users, &data.connector)?;
        self.register_nodes_and_edges(&data.data.assets, &data.connector)?;
        self.register_nodes_and_edges(&data.data.policies, &data.connector)?;
        self.register_nodes_and_edges(&data.data.tags, &data.connector)?;
        Ok(())
    }

    pub(crate) fn add_edges(&mut self) -> Result<()> {
        for edge in &self.edge_cache {
            self.graph
                .add_edge(edge.to_owned())
                .context(format!("couldn't add edge {:?} to graph", edge))?;
        }
        Ok(())
    }

    fn register_nodes_and_edges<T: NodeHelper>(
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

#[allow(dead_code)]
fn merge_set(s1: &HashSet<String>, s2: &HashSet<String>) -> HashSet<String> {
    let mut s1 = s1.to_owned();
    let s2 = s2.to_owned();
    s1.extend(s2);
    s1
}
#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use anyhow::Result;

    use crate::connectors::nodes::{self, ConnectorData};

    use super::*;

    #[test]
    fn edges_generated_from_group() -> Result<()> {
        let input_group = vec![nodes::Group {
            name: "Group 1".to_string(),
            member_of: HashSet::from(["Group a".to_string(), "Group b".to_string()]),
            includes_users: HashSet::from(["User a".to_string()]),
            includes_groups: HashSet::from(["Group c".to_string()]),
            granted_by: HashSet::from(["Policy 1".to_string()]),
            ..Default::default()
        }];

        let data = ProcessedConnectorData {
            connector: "test".to_string(),
            data: ConnectorData {
                groups: vec![],
                users: vec![],
                assets: vec![],
                policies: vec![],
                tags: vec![],
            },
        };

        let mut ag = AccessGraph::new(vec![data])?;

        let output_edges = HashSet::from([
            JettyEdge {
                from: NodeName::Group("Group 1".to_string()),
                to: NodeName::Group("Group a".to_string()),
                edge_type: EdgeType::MemberOf,
            },
            JettyEdge {
                to: NodeName::Group("Group 1".to_string()),
                from: NodeName::Group("Group a".to_string()),
                edge_type: EdgeType::Includes,
            },
            JettyEdge {
                to: NodeName::Group("Group b".to_string()),
                from: NodeName::Group("Group 1".to_string()),
                edge_type: EdgeType::MemberOf,
            },
            JettyEdge {
                from: NodeName::Group("Group b".to_string()),
                to: NodeName::Group("Group 1".to_string()),
                edge_type: EdgeType::Includes,
            },
            JettyEdge {
                from: NodeName::Group("Group 1".to_string()),
                to: NodeName::User("User a".to_string()),
                edge_type: EdgeType::Includes,
            },
            JettyEdge {
                from: NodeName::User("User a".to_string()),
                to: NodeName::Group("Group 1".to_string()),
                edge_type: EdgeType::MemberOf,
            },
            JettyEdge {
                from: NodeName::Group("Group 1".to_string()),
                to: NodeName::Group("Group c".to_string()),
                edge_type: EdgeType::Includes,
            },
            JettyEdge {
                from: NodeName::Group("Group c".to_string()),
                to: NodeName::Group("Group 1".to_string()),
                edge_type: EdgeType::MemberOf,
            },
            JettyEdge {
                from: NodeName::Group("Group 1".to_string()),
                to: NodeName::Policy("Policy 1".to_string()),
                edge_type: EdgeType::GrantedBy,
            },
            JettyEdge {
                from: NodeName::Policy("Policy 1".to_string()),
                to: NodeName::Group("Group 1".to_string()),
                edge_type: EdgeType::GrantedTo,
            },
        ]);

        ag.register_nodes_and_edges(&input_group, &("test".to_string()))?;
        assert_eq!(ag.edge_cache, output_edges);
        Ok(())
    }
}
