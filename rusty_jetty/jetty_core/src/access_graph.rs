//! # Access Graph
//!
//! `access_graph` is a library for modeling data access permissions and metadata as a graph.

pub mod explore;
pub mod explore2;
pub mod graph;
mod helpers;
#[cfg(test)]
pub mod test_util;

use crate::connectors::nodes::{ConnectorData, EffectivePermission, SparseMatrix};
use crate::connectors::UserIdentifier;
use crate::logging::debug;
use crate::tag_parser::{parse_tags, tags_to_jetty_node_helpers};
use crate::{connectors::AssetType, cual::Cual};

use self::helpers::NodeHelper;
pub use self::helpers::ProcessedConnectorData;

use super::connectors;
use core::hash::Hash;

use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::fs::File;
use std::io::BufWriter;
use std::ops::{Index, IndexMut};

use anyhow::{anyhow, bail, Context, Result};
// reexporting for use in other packages
pub use petgraph::stable_graph::NodeIndex;

use serde::Deserialize;
use serde::Serialize;
use time::OffsetDateTime;

use crate::permissions::matrix::Merge;

const SAVED_GRAPH_PATH: &str = "jetty_graph";

/// Attributes associated with a User node
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserAttributes {
    /// User name
    pub name: String,
    /// Specific user identifiers
    pub identifiers: HashSet<connectors::UserIdentifier>,
    /// Misc user identifiers
    pub other_identifiers: HashSet<String>,
    /// K-V pairs of user-specific metadata
    pub metadata: HashMap<String, String>,
    /// Connectors the user is present in
    pub connectors: HashSet<String>,
}

impl UserAttributes {
    fn merge_attributes(&self, new_attributes: &UserAttributes) -> Result<UserAttributes> {
        let name = merge_matched_field(&self.name, &new_attributes.name)
            .context("field: UserAttributes.name")?;
        let identifiers = merge_set(&self.identifiers, &new_attributes.identifiers);
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

    /// Convenience constructor for testing
    #[cfg(test)]
    fn new(name: String) -> Self {
        Self {
            name,
            identifiers: Default::default(),
            other_identifiers: Default::default(),
            metadata: Default::default(),
            connectors: Default::default(),
        }
    }
}

impl TryFrom<JettyNode> for UserAttributes {
    type Error = anyhow::Error;

    /// convert from a JettyNode to UserAttributes, if possible
    fn try_from(value: JettyNode) -> Result<Self, Self::Error> {
        match value {
            JettyNode::User(a) => Ok(a),
            _ => bail!("not a user node"),
        }
    }
}

/// Attributes associated with a Group node

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct GroupAttributes {
    /// Name of group
    pub name: String,
    /// k-v pairs of group metadata
    pub metadata: HashMap<String, String>,
    /// All the connectors the group is present in
    pub connectors: HashSet<String>,
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
    /// Convenience constructor for testing
    #[cfg(test)]
    fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

impl TryFrom<JettyNode> for GroupAttributes {
    type Error = anyhow::Error;

    /// convert from a JettyNode to GroupAttributes, if possible
    fn try_from(value: JettyNode) -> Result<Self, Self::Error> {
        match value {
            JettyNode::Group(a) => Ok(a),
            _ => bail!("not a user node"),
        }
    }
}

/// A struct defining the attributes of an asset
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetAttributes {
    cual: Cual,
    asset_type: AssetType,
    metadata: HashMap<String, String>,
    connectors: HashSet<String>,
}

impl AssetAttributes {
    fn merge_attributes(&self, new_attributes: &AssetAttributes) -> Result<AssetAttributes> {
        let cual = merge_matched_field(&self.cual, &new_attributes.cual)
            .context("field: GroupAttributes.cual")?;
        let asset_type = merge_matched_field(&self.asset_type, &self.asset_type)
            .context("field: AssetAttributes.asset_type")?;
        let metadata = merge_map(&self.metadata, &new_attributes.metadata)
            .context("field: AssetAttributes.metadata")?;
        let connectors = merge_set(&self.connectors, &new_attributes.connectors);
        Ok(AssetAttributes {
            cual,
            asset_type,
            metadata,
            connectors,
        })
    }

    pub(crate) fn cual(&self) -> &Cual {
        &self.cual
    }

    pub(crate) fn asset_type(&self) -> &AssetType {
        &self.asset_type
    }

    /// Convenience constructor for testing
    #[cfg(test)]
    pub(crate) fn new(cual: Cual) -> Self {
        Self {
            cual,
            asset_type: AssetType::default(),
            metadata: Default::default(),
            connectors: Default::default(),
        }
    }
}

impl TryFrom<JettyNode> for AssetAttributes {
    type Error = anyhow::Error;

    /// convert from a JettyNode to AssetAttributes, if possible
    fn try_from(value: JettyNode) -> Result<Self, Self::Error> {
        match value {
            JettyNode::Asset(a) => Ok(a),
            _ => bail!("not an asset node"),
        }
    }
}

/// A struct describing the attributes of a Tag
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagAttributes {
    /// Name of tag
    pub name: String,
    /// optional discription of the tag
    pub description: Option<String>,
    /// an optional value
    pub value: Option<String>,
    /// whether the tag is to be passed through hierarchy
    pub pass_through_hierarchy: bool,
    /// whether the tag is to be passed through lineage
    pub pass_through_lineage: bool,
    /// Connector the tag is from. This is not all the connectors that the tag may be applied to.
    /// We don't yet support specifying that.
    connectors: HashSet<String>,
}

impl TagAttributes {
    fn merge_attributes(&self, new_attributes: &TagAttributes) -> Result<TagAttributes> {
        let name = merge_matched_field(&self.name, &new_attributes.name)
            .context("field: TagAttributes.name")?;
        let description = merge_matched_field(&self.description, &new_attributes.description)
            .context("field: TagAttributes.description")?;
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
            description,
            pass_through_hierarchy,
            pass_through_lineage,
            connectors,
        })
    }

    /// Convenience constructor for testing
    #[cfg(test)]
    fn new(name: String, pass_through_hierarchy: bool, pass_through_lineage: bool) -> Self {
        Self {
            name,
            description: None,
            value: Default::default(),
            pass_through_hierarchy,
            pass_through_lineage,
            connectors: HashSet::from(["Jetty".to_owned()]),
        }
    }
}

impl TryFrom<JettyNode> for TagAttributes {
    type Error = anyhow::Error;

    /// convert from a JettyNode to TagAttributes, if possible
    fn try_from(value: JettyNode) -> Result<Self, Self::Error> {
        match value {
            JettyNode::Tag(a) => Ok(a),
            _ => bail!("not a tag node"),
        }
    }
}

/// A struct describing the attributes of a policy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyAttributes {
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

    /// Convenience constructor for testing
    #[cfg(test)]
    fn new(name: String) -> Self {
        Self {
            name,
            privileges: Default::default(),
            pass_through_hierarchy: Default::default(),
            pass_through_lineage: Default::default(),
            connectors: Default::default(),
        }
    }
}

impl TryFrom<JettyNode> for PolicyAttributes {
    type Error = anyhow::Error;

    /// convert from a JettyNode to PolicyAttributes, if possible
    fn try_from(value: JettyNode) -> Result<Self, Self::Error> {
        match value {
            JettyNode::Policy(a) => Ok(a),
            _ => bail!("not a policy node"),
        }
    }
}

/// Enum of node types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JettyNode {
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
    /// Get the type (as a string) of the node.
    pub fn get_string_name(&self) -> String {
        match &self {
            JettyNode::Group(g) => g.name.to_owned(),
            JettyNode::User(u) => u.name.to_owned(),
            JettyNode::Asset(a) => a.cual.uri(),
            JettyNode::Tag(t) => t.name.to_owned(),
            JettyNode::Policy(p) => p.name.to_owned(),
        }
    }

    /// Get a Vec of the connectors for a node
    pub fn get_node_connectors(&self) -> HashSet<String> {
        match &self {
            JettyNode::Group(g) => g.connectors.to_owned(),
            JettyNode::User(u) => u.connectors.to_owned(),
            JettyNode::Asset(a) => a.connectors.to_owned(),
            // Tags don't really have connectors at this point, so return an empty HashSet
            JettyNode::Tag(_t) => Default::default(),
            JettyNode::Policy(p) => p.connectors.to_owned(),
        }
    }

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
    fn get_node_name(&self) -> NodeName {
        match &self {
            JettyNode::Asset(a) => NodeName::Asset(a.cual.uri()),
            JettyNode::Group(a) => NodeName::Group(a.name.to_owned()),
            JettyNode::Policy(a) => NodeName::Policy(a.name.to_owned()),
            JettyNode::Tag(a) => NodeName::Tag(a.name.to_owned()),
            JettyNode::User(a) => NodeName::User(a.name.to_owned()),
        }
    }
}

/// Enum of edge types
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub enum EdgeType {
    /// user|group -> is a member of -> group
    MemberOf,
    /// group -> includes, as members -> user|group
    Includes,
    /// group|user -> has permission granted by -> policy
    GrantedBy,
    /// asset -> hierarchical child of -> asset
    ChildOf,
    /// asset -> hierarchical parent of -> asset
    ParentOf,
    /// asset -> derived via lineage from -> asset
    DerivedFrom,
    /// asset -> parent, via lineage, of -> asset
    DerivedTo,
    /// asset -> tagged with -> tag
    TaggedAs,
    /// asset -> governed by -> policy
    GovernedBy,
    /// tag -> applied to -> asset
    AppliedTo,
    /// policy -> governs -> asset
    Governs,
    /// policy -> granted_to -> user|group
    GrantedTo,
    /// tag -> removed from -> asset
    RemovedFrom,
    /// asset -> had removed -> tag
    UntaggedAs,
    /// anything else
    #[default]
    Other,
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
        EdgeType::RemovedFrom => EdgeType::UntaggedAs,
        EdgeType::UntaggedAs => EdgeType::RemovedFrom,
        EdgeType::Other => EdgeType::Other,
    }
}

/// Mapping of node identifiers (like asset name) to their id in the graph
#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize)]
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

#[derive(Hash, Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub(crate) struct JettyEdge {
    from: NodeName,
    to: NodeName,
    edge_type: EdgeType,
}

impl JettyEdge {
    #[allow(dead_code)]
    pub(crate) fn new(from: NodeName, to: NodeName, edge_type: EdgeType) -> Self {
        Self {
            from,
            to,
            edge_type,
        }
    }
}

/// Representation of data access state
#[derive(Serialize, Deserialize)]
pub struct AccessGraph {
    /// The graph itself
    pub(crate) graph: graph::Graph,
    edge_cache: HashSet<JettyEdge>,
    /// Unix timestamp of when the graph was built
    last_modified: OffsetDateTime,
    /// The merged effective permissions from all connectors
    effective_permissions: SparseMatrix<UserIdentifier, Cual, HashSet<EffectivePermission>>,
}

impl Index<NodeIndex> for AccessGraph {
    type Output = JettyNode;

    fn index(&self, index: NodeIndex) -> &Self::Output {
        self.graph.graph.index(index)
    }
}

impl IndexMut<NodeIndex> for AccessGraph {
    fn index_mut(&mut self, index: NodeIndex) -> &mut Self::Output {
        self.graph.graph.index_mut(index)
    }
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
            last_modified: OffsetDateTime::now_utc(),
            effective_permissions: Default::default(),
        };
        for connector_data in data {
            // Create all nodes first, then create edges.
            ag.add_nodes(&connector_data)?;
            // Merge effective permissions into the access graph
            ag.effective_permissions
                .merge(connector_data.data.effective_permissions)
                .context("merging effective permissions")
                .unwrap();
        }
        ag.add_edges()?;
        Ok(ag)
    }

    #[cfg(test)]
    /// New graph
    pub fn new_dummy(nodes: &[&JettyNode], edges: &[(NodeName, NodeName, EdgeType)]) -> Self {
        use self::test_util::new_graph_with;

        AccessGraph {
            graph: new_graph_with(nodes, edges).unwrap(),
            edge_cache: HashSet::new(),
            last_modified: OffsetDateTime::now_utc(),
            effective_permissions: Default::default(),
        }
    }

    /// Get last modified date for access graph
    pub fn get_last_modified(&self) -> OffsetDateTime {
        self.last_modified
    }

    pub(crate) fn add_nodes(&mut self, data: &ProcessedConnectorData) -> Result<()> {
        self.register_nodes_and_edges(&data.data.groups, &data.connector, None)?;
        self.register_nodes_and_edges(&data.data.users, &data.connector, None)?;
        self.register_nodes_and_edges(
            &data.data.assets,
            &data.connector,
            Some(|node, connector| node.cual.scheme() != connector.trim()),
        )?;
        self.register_nodes_and_edges(&data.data.policies, &data.connector, None)?;
        self.register_nodes_and_edges(&data.data.tags, &data.connector, None)?;
        Ok(())
    }

    /// Adds all the edges from the edge cache, draining the cache as it goes.
    pub(crate) fn add_edges(&mut self) -> Result<()> {
        for edge in self.edge_cache.drain() {
            if !self.graph.add_edge(edge.to_owned()) {
                debug!("couldn't add edge {:?} to graph", edge);
            }
        }
        Ok(())
    }

    /// Add nodes to the graph and add edges to the edge cache
    fn register_nodes_and_edges<T: NodeHelper>(
        &mut self,
        nodes: &Vec<T>,
        connector: &String,
        filter: Option<fn(&T, &str) -> bool>,
    ) -> Result<()> {
        for n in nodes {
            if let Some(should_filter) = filter {
                if should_filter(n, connector) {
                    debug!(
                        "Filtering node {:?}",
                        n.get_node(connector.to_owned()).get_string_name()
                    );
                    continue;
                }
            }
            let node = n.get_node(connector.to_owned());
            self.graph.add_node(&node)?;
            let edges = n.get_edges();
            self.edge_cache.extend(edges);
        }
        Ok(())
    }

    /// Convenience fn to visualize the graph.
    pub fn visualize(&self, path: &str) -> Result<String> {
        self.graph.visualize(path)
    }

    /// Write the graph to disk
    pub fn serialize_graph(&self) -> Result<()> {
        let f = File::create(SAVED_GRAPH_PATH).context("creating file")?;
        let f = BufWriter::new(f);
        bincode::serialize_into(f, &self).context("serializing graph into file")?;
        Ok(())
    }

    /// Read the graph from disk
    pub fn deserialize_graph() -> Result<Self> {
        let f = File::open(SAVED_GRAPH_PATH).context("opening graph file")?;
        let decoded = bincode::deserialize_from(f).context("deserializing graph from file")?;
        Ok(decoded)
    }

    /// Return a pointer to the petgraph - makes it easy to index and get node values
    pub fn graph(&self) -> &petgraph::stable_graph::StableGraph<JettyNode, EdgeType> {
        &self.graph.graph
    }
    /// add tags and appropriate edges from a configuration file to the graph
    pub fn add_tags(&mut self, config: &String) -> Result<()> {
        let parsed_tags = parse_tags(config)?;
        let tags = tags_to_jetty_node_helpers(parsed_tags, self, config)?;
        self.add_nodes(&ProcessedConnectorData {
            connector: "Jetty".to_owned(),
            data: ConnectorData {
                tags,
                ..Default::default()
            },
        })?;

        // add edges from the cache
        self.add_edges()?;

        Ok(())
    }
}

#[allow(dead_code)]
fn merge_set<T>(s1: &HashSet<T>, s2: &HashSet<T>) -> HashSet<T>
where
    T: Eq + Hash + Clone,
{
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
                effective_permissions: HashMap::new(),
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

        ag.register_nodes_and_edges(&input_group, &("test".to_string()), None)?;
        assert_eq!(ag.edge_cache, output_edges);
        Ok(())
    }
}
