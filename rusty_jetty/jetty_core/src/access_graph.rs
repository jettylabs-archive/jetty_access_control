//! # Access Graph
//!
//! `access_graph` is a library for modeling data access permissions and metadata as a graph.

pub mod explore;
pub mod graph;
pub mod helpers;
#[cfg(test)]
pub mod test_util;
pub mod translate;

use crate::connectors::nodes::{ConnectorData, EffectivePermission, SparseMatrix};
use crate::connectors::processed_nodes::{ProcessedConnectorData, ProcessedDefaultPolicy};
#[cfg(test)]
use crate::cual::Cual;
use crate::log_runtime;
use crate::Jetty;

use crate::connectors::{AssetType, UserIdentifier};
use crate::jetty::ConnectorNamespace;
use crate::logging::debug;
use crate::write::tag_parser::{parse_tags, tags_to_jetty_node_helpers};

use self::graph::typed_indices::{AssetIndex, GroupIndex, PolicyIndex, TagIndex, UserIndex};
use self::helpers::NodeHelper;
use self::translate::Translator;

use super::connectors;
use core::hash::Hash;

use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::{Debug, Display};
use std::fs::{self, File};
use std::io::BufWriter;
use std::ops::{Index, IndexMut};
use std::path::PathBuf;

use anyhow::{anyhow, bail, Context, Result};
// reexporting for use in other packages
pub use petgraph::stable_graph::NodeIndex;
use serde::Deserialize;
use serde::Serialize;
use time::OffsetDateTime;

use uuid::Uuid;

use crate::permissions::matrix::InsertOrMerge;

/// Attributes associated with a User node
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct UserAttributes {
    /// User name
    pub name: NodeName,
    /// Node Id
    pub id: Uuid,
    /// Specific user identifiers
    pub identifiers: HashSet<connectors::UserIdentifier>,
    /// Misc user identifiers
    pub metadata: HashMap<String, String>,
    /// Connectors the user is present in
    pub connectors: HashSet<ConnectorNamespace>,
}

impl UserAttributes {
    fn merge_attributes(&self, new_attributes: &UserAttributes) -> Result<UserAttributes> {
        let name = merge_matched_field(&self.name, &new_attributes.name)
            .context("field: UserAttributes.name")?;
        let identifiers = merge_set(&self.identifiers, &new_attributes.identifiers);
        let metadata = merge_map(&self.metadata, &new_attributes.metadata)
            .context("field: UserAttributes.metadata")?;
        let connectors = merge_set(&self.connectors, &new_attributes.connectors);
        Ok(UserAttributes {
            name,
            id: self.id,
            identifiers,
            metadata,
            connectors,
        })
    }

    /// Generate new UserAttributes struct
    pub(crate) fn new(
        name: &NodeName,
        identifiers: &HashSet<UserIdentifier>,
        metadata: &HashMap<String, String>,
        connector: Option<&ConnectorNamespace>,
    ) -> Self {
        UserAttributes {
            name: name.to_owned(),
            id: Uuid::new_v5(&Uuid::NAMESPACE_URL, name.to_string().as_bytes()),
            identifiers: identifiers.to_owned(),
            metadata: metadata.to_owned(),
            connectors: connector
                .map(|c| HashSet::from([c.to_owned()]))
                .unwrap_or_default(),
        }
    }

    /// users can only have at most one connector, so this makes things a bit easier.
    /// While the graph is being created, it's possible that a user has 0 connectors, but that
    /// should never be the case when this might be called
    pub(crate) fn connectors(&self) -> &HashSet<ConnectorNamespace> {
        &self.connectors
    }

    #[cfg(test)]
    fn simple_new(name: String) -> Self {
        Self {
            name: NodeName::User(name.to_owned()),
            id: Uuid::new_v5(
                &Uuid::NAMESPACE_URL,
                NodeName::User(name).to_string().as_bytes(),
            ),
            identifiers: Default::default(),
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
    pub name: NodeName,
    /// Node Id
    pub id: Uuid,
    /// k-v pairs of group metadata
    pub metadata: HashMap<String, String>,
    /// All the connectors the group is present in
    pub connectors: HashSet<ConnectorNamespace>,
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
            id: self.id,
            metadata,
            connectors,
        })
    }
    /// Convenience constructor for testing
    #[cfg(test)]
    fn new(name: String) -> Self {
        Self {
            name: NodeName::Group {
                name,
                origin: ConnectorNamespace("".to_owned()),
            },
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
    /// Name of Asset
    pub name: NodeName,
    /// Node Id
    pub id: Uuid,
    /// Asset type
    pub asset_type: AssetType,
    /// Asset metadata
    pub metadata: HashMap<String, String>,
    /// Asset connectors
    pub connectors: HashSet<ConnectorNamespace>,
}

impl AssetAttributes {
    fn merge_attributes(&self, new_attributes: &AssetAttributes) -> Result<AssetAttributes> {
        let name = merge_matched_field(&self.name, &new_attributes.name)
            .context("field: GroupAttributes.name")?;
        let asset_type = merge_matched_field(&self.asset_type, &self.asset_type)
            .context("field: AssetAttributes.asset_type")?;
        let metadata = merge_map(&self.metadata, &new_attributes.metadata)
            .context("field: AssetAttributes.metadata")?;
        let connectors = merge_set(&self.connectors, &new_attributes.connectors);
        Ok(AssetAttributes {
            name,
            id: self.id,
            asset_type,
            metadata,
            connectors,
        })
    }

    pub(crate) fn name(&self) -> &NodeName {
        &self.name
    }

    pub(crate) fn asset_type(&self) -> &AssetType {
        &self.asset_type
    }

    /// Convenience constructor for testing
    #[cfg(test)]
    pub(crate) fn new(cual: Cual, connector: ConnectorNamespace) -> Self {
        let node_name = NodeName::Asset {
            connector,
            asset_type: cual.asset_type(),
            path: cual.asset_path(),
        };
        Self {
            name: node_name.to_owned(),
            id: Uuid::new_v5(&Uuid::NAMESPACE_URL, node_name.to_string().as_bytes()),
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
    pub name: NodeName,
    /// Node Id
    pub id: Uuid,
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
    pub connectors: HashSet<ConnectorNamespace>,
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
            id: self.id,
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
        let node_name = NodeName::Tag(name);
        Self {
            name: node_name.to_owned(),
            id: Uuid::new_v5(&Uuid::NAMESPACE_URL, node_name.to_string().as_bytes()),
            description: None,
            value: Default::default(),
            pass_through_hierarchy,
            pass_through_lineage,
            connectors: HashSet::from([ConnectorNamespace("Jetty".to_owned())]),
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
    /// Policy name
    pub name: NodeName,
    /// Node Id
    pub id: Uuid,
    /// Policy privileges
    pub privileges: HashSet<String>,
    /// Whether the policy is passed through hierarchy
    pub pass_through_hierarchy: bool,
    /// Whether the policy is passed through lineage
    pub pass_through_lineage: bool,
    /// Policy connectors
    pub connectors: HashSet<ConnectorNamespace>,
}

/// A struct describing the attributes of a policy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefaultPolicyAttributes {
    /// Policy name
    pub name: NodeName,
    /// Node Id
    pub id: Uuid,
    /// Policy privileges
    pub privileges: HashSet<String>,
    /// The path that this policy should be applied to
    pub matching_path: String,
    /// Metadata associated with the policy
    pub metadata: HashMap<String, String>,
    /// The types of assets that this should be applied to. If None, it's applied to all types
    pub target_type: AssetType,
    /// Connectors for the default policy. Policies include a single asset and single grantee, so
    /// this set should only ever include a single connector
    pub connectors: HashSet<ConnectorNamespace>,
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
            id: self.id,
            privileges,
            pass_through_hierarchy,
            pass_through_lineage,
            connectors,
        })
    }

    /// Convenience constructor for testing
    #[cfg(test)]
    fn new(name: String) -> Self {
        let node_name = NodeName::Policy {
            name,
            origin: Default::default(),
        };
        Self {
            name: node_name.to_owned(),
            id: Uuid::new_v5(&Uuid::NAMESPACE_URL, node_name.to_string().as_bytes()),
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

impl TryFrom<JettyNode> for DefaultPolicyAttributes {
    type Error = anyhow::Error;

    /// convert from a JettyNode to PolicyAttributes, if possible
    fn try_from(value: JettyNode) -> Result<Self, Self::Error> {
        match value {
            JettyNode::DefaultPolicy(a) => Ok(a),
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
    /// Default policy
    DefaultPolicy(DefaultPolicyAttributes),
}

impl JettyNode {
    /// Get the type (as a string) of the node.
    pub fn get_string_name(&self) -> String {
        match &self {
            JettyNode::Group(g) => g.name.to_string(),
            JettyNode::User(u) => u.name.to_string(),
            JettyNode::Asset(a) => a.name.to_string(),
            JettyNode::Tag(t) => t.name.to_string(),
            JettyNode::Policy(p) => p.name.to_string(),
            JettyNode::DefaultPolicy(p) => p.name.to_string(),
        }
    }

    /// Get a Vec of the connectors for a node
    pub fn get_node_connectors(&self) -> HashSet<ConnectorNamespace> {
        match &self {
            JettyNode::Group(g) => g.connectors.to_owned(),
            JettyNode::User(u) => u.connectors.to_owned(),
            JettyNode::Asset(a) => a.connectors.to_owned(),
            // Tags don't really have connectors at this point, so return an empty HashSet
            JettyNode::Tag(_t) => Default::default(),
            JettyNode::Policy(p) => p.connectors.to_owned(),
            JettyNode::DefaultPolicy(p) => p.connectors.to_owned(),
        }
    }

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
    pub(crate) fn get_node_name(&self) -> NodeName {
        match &self {
            JettyNode::Asset(a) => a.name.to_owned(),
            JettyNode::Group(a) => a.name.to_owned(),
            JettyNode::Policy(a) => a.name.to_owned(),
            JettyNode::Tag(a) => a.name.to_owned(),
            JettyNode::User(a) => a.name.to_owned(),
            JettyNode::DefaultPolicy(a) => a.name.to_owned(),
        }
    }

    /// Get id from a JettyNode
    pub fn id(&self) -> Uuid {
        match &self {
            JettyNode::Group(n) => n.id,
            JettyNode::User(n) => n.id,
            JettyNode::Asset(n) => n.id,
            JettyNode::Tag(n) => n.id,
            JettyNode::Policy(n) => n.id,
            JettyNode::DefaultPolicy(n) => n.id,
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
    GrantedFrom,
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
    /// default policy -> asset
    ProvidedDefaultForChildren,
    /// asset -> default policy
    ReceivedChildrensDefaultFrom,
    /// anything else
    #[default]
    Other,
}

fn get_edge_type_pair(edge_type: &EdgeType) -> EdgeType {
    match edge_type {
        EdgeType::MemberOf => EdgeType::Includes,
        EdgeType::Includes => EdgeType::MemberOf,
        EdgeType::GrantedFrom => EdgeType::GrantedTo,
        EdgeType::GrantedTo => EdgeType::GrantedFrom,
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
        EdgeType::ProvidedDefaultForChildren => EdgeType::ReceivedChildrensDefaultFrom,
        EdgeType::ReceivedChildrensDefaultFrom => EdgeType::ProvidedDefaultForChildren,
        EdgeType::Other => EdgeType::Other,
    }
}

/// A type representing the path of an asset
///
#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize, PartialOrd, Ord)]
pub struct AssetPath(Vec<String>);

impl Display for AssetPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.join("/"))
    }
}

impl AssetPath {
    /// Return a Vec<String> of the different path components
    pub fn components(&self) -> &Vec<String> {
        &self.0
    }

    /// Return the ancestors of a path
    pub fn ancestors(&self) -> Vec<&[String]> {
        self.0
            .iter()
            .enumerate()
            .map(|(i, _)| &self.0[..i + 1])
            .collect()
    }

    /// Build a new asset path
    pub(crate) fn new(path: Vec<String>) -> Self {
        Self(path)
    }
}

/// Mapping of node identifiers (like asset name) to their id in the graph
#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize, PartialOrd, Ord)]
pub enum NodeName {
    /// User node
    User(String),
    /// Group node
    Group {
        /// Group name
        name: String,
        /// Origin connector
        origin: ConnectorNamespace,
    },
    /// Asset node
    Asset {
        /// The assets connector
        connector: ConnectorNamespace,
        /// The AssetType if, and only if, it is included in the cual
        asset_type: Option<AssetType>,
        /// The path to the asset
        path: AssetPath,
    },
    /// Policy node
    Policy {
        /// Policy name
        name: String,
        /// Origin connector
        origin: ConnectorNamespace,
    },
    /// Tag node
    Tag(String),
    /// Default Policy Node
    DefaultPolicy {
        /// Root of the default policy path (before any wildcards)
        root_node: Box<NodeName>,
        /// The path of wildcards
        matching_path: String,
        /// The types of assets the policy should be applied to
        target_type: AssetType,
        /// The group/user the policy is granted to
        grantee: Box<NodeName>,
    },
}

impl Default for NodeName {
    fn default() -> Self {
        NodeName::User("".to_owned())
    }
}

impl Display for NodeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeName::User(n) => write!(f, "{n}"),
            NodeName::Group { name, origin } => write!(f, "{origin}::{name}"),
            NodeName::Asset {
                connector,
                asset_type,
                path,
            } => write!(
                f,
                "{}::{}{}",
                connector,
                path,
                match asset_type {
                    Some(t) => format!(" ({})", t.to_string()),
                    None => "".to_string(),
                }
            ),
            NodeName::Policy { name, origin } => write!(f, "{origin}::{name}"),
            NodeName::Tag(n) => write!(f, "{n}"),
            NodeName::DefaultPolicy {
                root_node,
                matching_path,
                target_type,
                grantee,
            } => {
                write!(
                    f,
                    "{} -> {}/{} ({})",
                    grantee,
                    root_node,
                    matching_path,
                    target_type.to_string(),
                )
            }
        }
    }
}

impl NodeName {
    /// This function generates a string intended to be used for matching in configuration files
    pub fn name_for_string_matching(&self) -> String {
        match self {
            NodeName::Asset {
                connector, path, ..
            } => {
                // for Assets, the matchable portion is the namespace + path.
                // The type must be matched separately.
                format!("{connector}::{path}")
            }
            _ => todo!(),
        }
    }

    /// This function gets the origin for a given group nodename, and fails if the nodename isn't for a group
    pub(crate) fn get_group_origin(&self) -> Result<&ConnectorNamespace> {
        match self {
            NodeName::Group { origin, .. } => Ok(origin),
            _ => bail!("expected a group nodename"),
        }
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub(crate) struct JettyEdge {
    from: NodeName,
    to: NodeName,
    edge_type: EdgeType,
}

impl JettyEdge {
    pub(crate) fn new(from: NodeName, to: NodeName, edge_type: EdgeType) -> Self {
        Self {
            from,
            to,
            edge_type,
        }
    }
}

/// Representation of data access state
#[derive(Serialize, Deserialize, Default)]
pub struct AccessGraph {
    /// The graph itself
    pub(crate) graph: graph::Graph,
    edge_cache: HashSet<JettyEdge>,
    /// Unix timestamp of when the graph was built
    last_modified: Option<OffsetDateTime>,
    /// The merged effective permissions from all connectors
    effective_permissions: SparseMatrix<UserIndex, AssetIndex, HashSet<EffectivePermission>>,
    /// The translator between local and global namespaces
    translator: Translator,
}

impl<T: Into<NodeIndex>> Index<T> for AccessGraph {
    type Output = JettyNode;

    fn index(&self, index: T) -> &Self::Output {
        let node_index: NodeIndex = index.into();
        self.graph.graph.index(node_index)
    }
}

impl<T: Into<NodeIndex>> IndexMut<T> for AccessGraph {
    fn index_mut(&mut self, index: T) -> &mut Self::Output {
        let node_index: NodeIndex = index.into();
        self.graph.graph.index_mut(node_index)
    }
}

impl AccessGraph {
    /// New graph
    pub fn new(
        connector_data: ProcessedConnectorData,
        translator: Option<Translator>,
    ) -> Result<Self> {
        let mut ag = AccessGraph {
            graph: graph::Graph {
                graph: petgraph::stable_graph::StableDiGraph::new(),
                nodes: Default::default(),
                node_ids: Default::default(),
                partial_match_mapping: Default::default(),
            },
            edge_cache: HashSet::new(),
            last_modified: Some(OffsetDateTime::now_utc()),
            effective_permissions: Default::default(),
            translator: if let Some(t) = translator {
                t
            } else {
                Default::default()
            },
        };
        // Create all nodes first, then create edges.
        log_runtime!("Add nodes", ag.add_nodes(&connector_data)?);
        log_runtime!("Add Edges", ag.add_edges()?);

        // Add default policies after the rest of the graph is created. This is necessary because
        // these policies depend on hierarchy, which isn't really established until this point.
        ag.register_default_policy_nodes_and_edges(&connector_data.default_policies)?;
        // Add the newly created edges
        ag.add_edges()?;

        // Merge effective permissions into the access graph
        ag.effective_permissions = ag.translate_effective_permissions_to_global_indices(
            connector_data.effective_permissions,
        );

        Ok(ag)
    }

    /// Create a new Access Graph from a Vec<(ConnectorData, ConnectorNamespace)>.
    /// This handles the translation from connectors local state to a global state
    pub fn new_from_connector_data(
        connector_data: Vec<(ConnectorData, ConnectorNamespace)>,
        jetty: &Jetty,
    ) -> Result<Self> {
        // Build the translator
        let tr = log_runtime!(
            "Initialize translator",
            Translator::new(&connector_data, jetty)?
        );
        // Process the connector data
        let pcd = log_runtime!(
            "Local to processed data",
            tr.local_to_processed_connector_data(connector_data)
        );
        let ag_res = log_runtime!(
            "Create access graph",
            AccessGraph::new(pcd.to_owned(), Some(tr))
        );

        log_runtime!(
            "translate effective permissions",
            ag_res.map(|mut ag| {
                ag.effective_permissions =
                    ag.translate_effective_permissions_to_global_indices(pcd.effective_permissions);
                ag
            })
        )
    }

    /// Return the translator
    pub fn translator(&self) -> &Translator {
        &self.translator
    }

    /// Return a mutable reference to the translator
    pub fn translator_mut(&mut self) -> &mut Translator {
        &mut self.translator
    }

    /// This translate effective permissions from using node names for indices to using
    /// node indices
    fn translate_effective_permissions_to_global_indices(
        &self,
        // This should match SparseMatrix<NodeName::User(), NodeName::Asset(), HashSet<_>>
        node_name_permissions: SparseMatrix<NodeName, NodeName, HashSet<EffectivePermission>>,
    ) -> SparseMatrix<UserIndex, AssetIndex, HashSet<EffectivePermission>> {
        let mut result = SparseMatrix::new();

        for (k1, v1) in &node_name_permissions {
            for (k2, v2) in v1 {
                result.insert_or_merge(
                    self.get_user_index_from_name(k1).unwrap(),
                    HashMap::from([(self.get_asset_index_from_name(k2).unwrap(), v2.to_owned())]),
                );
            }
        }
        result
    }

    // Get indices by id

    /// Get the untyped node index for a given NodeName.
    /// **Always prefer the get_untyped_index_from_id function when possible**
    pub fn get_untyped_index_from_name(&self, node_name: &NodeName) -> Option<NodeIndex> {
        self.graph.get_untyped_node_index(node_name)
    }

    /// Get the typed node index for a given NodeName.
    /// **Always prefer the get_asset_index_from_id function when possible**
    pub fn get_asset_index_from_name(&self, node_name: &NodeName) -> Option<AssetIndex> {
        self.graph.get_asset_node_index(node_name)
    }

    /// Get the untyped node index for a given NodeName.
    /// **Always prefer the get_user_index_from_id function when possible**
    pub fn get_user_index_from_name(&self, node_name: &NodeName) -> Option<UserIndex> {
        self.graph.get_user_node_index(node_name)
    }

    /// Get the untyped node index for a given NodeName.
    /// **Always prefer the get_tag_index_from_id function when possible**
    pub fn get_tag_index_from_name(&self, node_name: &NodeName) -> Option<TagIndex> {
        self.graph.get_tag_node_index(node_name)
    }

    /// Get the untyped node index for a given NodeName.
    /// **Always prefer the get_policy_index_from_id function when possible**
    pub fn get_policy_index_from_name(&self, node_name: &NodeName) -> Option<PolicyIndex> {
        self.graph.get_policy_node_index(node_name)
    }

    /// Get the untyped node index for a given NodeName.
    /// **Always prefer the get_group_index_from_id function when possible**
    pub fn get_group_index_from_name(&self, node_name: &NodeName) -> Option<GroupIndex> {
        self.graph.get_group_node_index(node_name)
    }

    // Get indices by id

    /// Get the untyped node index for a given Node ID
    pub fn get_untyped_index_from_id(&self, node_id: &Uuid) -> Option<NodeIndex> {
        self.graph.get_untyped_node_index_from_id(node_id)
    }

    /// Get the typed node index for a given Node Id
    pub fn get_asset_index_from_id(&self, node_id: &Uuid) -> Option<AssetIndex> {
        self.graph.get_asset_node_index_from_id(node_id)
    }
    /// Get the typed node index for a given Node Id
    pub fn get_user_index_from_id(&self, node_id: &Uuid) -> Option<UserIndex> {
        self.graph.get_user_node_index_from_id(node_id)
    }
    /// Get the typed node index for a given Node Id
    pub fn get_tag_index_from_id(&self, node_id: &Uuid) -> Option<TagIndex> {
        self.graph.get_tag_node_index_from_id(node_id)
    }
    /// Get the typed node index for a given Node Id
    pub fn get_policy_index_from_id(&self, node_id: &Uuid) -> Option<PolicyIndex> {
        self.graph.get_policy_node_index_from_id(node_id)
    }
    /// Get the typed node index for a given Node Id
    pub fn get_group_index_from_id(&self, node_id: &Uuid) -> Option<GroupIndex> {
        self.graph.get_group_node_index_from_id(node_id)
    }

    #[cfg(test)]
    /// New graph
    pub fn new_dummy(nodes: &[&JettyNode], edges: &[(NodeName, NodeName, EdgeType)]) -> Self {
        use self::test_util::new_graph_with;

        AccessGraph {
            graph: new_graph_with(nodes, edges).unwrap(),
            edge_cache: HashSet::new(),
            last_modified: Default::default(),
            effective_permissions: Default::default(),
            translator: Default::default(),
        }
    }

    /// Get last modified date for access graph
    pub fn get_last_modified(&self) -> Option<OffsetDateTime> {
        self.last_modified
    }

    /// Go through the connector data an add nodes and edges for most node types.
    /// **This intentionally excludes default policies. They must be handled separately
    /// after other nodes are added**
    pub(crate) fn add_nodes(&mut self, data: &ProcessedConnectorData) -> Result<()> {
        debug!("Number of groups being added: {}", &data.groups.len());
        debug!("Number of users being added: {}", &data.users.len());
        debug!("Number of assets being added: {}", &data.assets.len());
        debug!("Number of policies being added: {}", &data.policies.len());
        debug!("Number of tags being added: {}", &data.tags.len());
        debug!(
            "Number of asset_references being added: {}",
            &data.asset_references.len()
        );

        log_runtime!(
            "add nodes for groups",
            self.register_nodes_and_edges(&data.groups)?
        );
        log_runtime!(
            "add nodes for users",
            self.register_nodes_and_edges(&data.users)?
        );
        log_runtime!(
            "add nodes for assets",
            self.register_nodes_and_edges(&data.assets)?
        );
        log_runtime!(
            "add nodes for policies",
            self.register_nodes_and_edges(&data.policies)?
        );
        log_runtime!(
            "add nodes for tags",
            self.register_nodes_and_edges(&data.tags)?
        );
        log_runtime!(
            "add nodes for asset_references",
            self.register_nodes_and_edges(&data.asset_references)?
        );
        Ok(())
    }

    /// Adds all the edges from the edge cache, draining the cache as it goes.
    pub(crate) fn add_edges(&mut self) -> Result<()> {
        debug!("Edge cache size: {}", self.edge_cache.len());
        for edge in self.edge_cache.drain() {
            if !self.graph.add_edge(edge.to_owned()) {
                debug!("couldn't add edge {:?} to graph", edge);
            }
        }
        Ok(())
    }

    /// Add nodes to the graph and add edges to the edge cache
    fn register_nodes_and_edges<T: NodeHelper>(&mut self, nodes: &Vec<T>) -> Result<()> {
        for n in nodes {
            // Edges get added regardless of connector.
            let edges = n.get_edges();
            self.edge_cache.extend(edges);

            if let Some(node) = n.get_node() {
                self.graph.add_node(&node)?;
            }
        }
        Ok(())
    }

    fn register_default_policy_nodes_and_edges(
        &mut self,
        nodes: &Vec<ProcessedDefaultPolicy>,
    ) -> Result<()> {
        for n in nodes {
            let edges = n.get_edges(self);
            self.edge_cache.extend(edges);

            if let Some(node) = n.get_node() {
                self.graph.add_node(&node)?;
            } else {
                bail!("unable to add default policy node")
            }
        }
        Ok(())
    }

    /// Convenience fn to visualize the graph.
    pub fn visualize(&self, path: &str) -> Result<String> {
        self.graph.visualize(path)
    }

    /// Write the graph to disk
    pub fn serialize_graph(&self, graph_path: PathBuf) -> Result<()> {
        if let Some(p) = graph_path.parent() {
            fs::create_dir_all(p)?
        };
        let f = File::create(graph_path).context("creating file")?;
        let f = BufWriter::new(f);
        bincode::serialize_into(f, &self).context("serializing graph into file")?;
        Ok(())
    }

    /// Read the graph from disk
    pub fn deserialize_graph(graph_path: PathBuf) -> Result<Self> {
        let f = File::open(graph_path).context("opening graph file")?;
        let decoded = bincode::deserialize_from(f).context("deserializing graph from file")?;
        Ok(decoded)
    }

    /// Return a pointer to the petgraph - makes it easy to index and get node values
    pub fn graph(&self) -> &petgraph::stable_graph::StableGraph<JettyNode, EdgeType> {
        &self.graph.graph
    }
    /// add tags and appropriate edges from a configuration file to the graph
    pub fn add_tags(&mut self, config: &String) -> Result<()> {
        let parsed_tags = parse_tags(config).context("unable to parse tags")?;
        let tags = tags_to_jetty_node_helpers(parsed_tags, self, config)
            .context("unable to add tags to your environment")?;
        self.add_nodes(&ProcessedConnectorData {
            tags,
            ..Default::default()
        })?;

        // add edges from the cache
        self.add_edges()?;

        Ok(())
    }

    // Given a node index and a connector, add that connector to the specified asset in the graph
    pub(crate) fn add_connector_to_user<T>(
        &mut self,
        user_idx: T,
        connector: &ConnectorNamespace,
    ) -> Result<()>
    where
        T: Into<NodeIndex> + Copy,
    {
        match &mut self[user_idx.into()] {
            JettyNode::User(attributes) => attributes.connectors.insert(connector.to_owned()),
            _ => bail!("requires a node type"),
        };
        Ok(())
    }

    // Given a node index and a connector, remove that connector from the specified asset in the graph
    pub(crate) fn remove_connector_from_user<T>(
        &mut self,
        user_idx: T,
        connector: &ConnectorNamespace,
    ) -> Result<()>
    where
        T: Into<NodeIndex> + Copy,
    {
        match &mut self[user_idx.into()] {
            JettyNode::User(attributes) => attributes.connectors.remove(connector),
            _ => bail!("requires a node type"),
        };
        Ok(())
    }
}

fn merge_set<T>(s1: &HashSet<T>, s2: &HashSet<T>) -> HashSet<T>
where
    T: Eq + Hash + Clone,
{
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

pub(crate) fn merge_map<K, V>(m1: &HashMap<K, V>, m2: &HashMap<K, V>) -> Result<HashMap<K, V>>
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
pub fn cual_to_asset_name_test(cual: Cual, connector: ConnectorNamespace) -> NodeName {
    NodeName::Asset {
        connector,
        asset_type: cual.asset_type(),
        path: cual.asset_path(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use anyhow::Result;

    use crate::connectors::processed_nodes::ProcessedGroup;

    use super::*;

    #[test]
    fn edges_generated_from_group() -> Result<()> {
        let input_group = vec![ProcessedGroup {
            name: NodeName::Group {
                name: "Group 1".to_string(),
                origin: Default::default(),
            },
            member_of: HashSet::from([
                NodeName::Group {
                    name: "Group a".to_string(),
                    origin: Default::default(),
                },
                NodeName::Group {
                    name: "Group b".to_string(),
                    origin: Default::default(),
                },
            ]),
            includes_users: HashSet::from([NodeName::User("User a".to_string())]),
            includes_groups: HashSet::from([NodeName::Group {
                name: "Group c".to_string(),
                origin: Default::default(),
            }]),
            granted_by: HashSet::from([NodeName::Policy {
                name: "Policy 1".to_string(),
                origin: Default::default(),
            }]),
            ..Default::default()
        }];

        let mut ag = AccessGraph::new(Default::default(), None)?;

        let output_edges = HashSet::from([
            JettyEdge {
                from: NodeName::Group {
                    name: "Group 1".to_string(),
                    origin: Default::default(),
                },
                to: NodeName::Group {
                    name: "Group a".to_string(),
                    origin: Default::default(),
                },
                edge_type: EdgeType::MemberOf,
            },
            JettyEdge {
                to: NodeName::Group {
                    name: "Group 1".to_string(),
                    origin: Default::default(),
                },
                from: NodeName::Group {
                    name: "Group a".to_string(),
                    origin: Default::default(),
                },
                edge_type: EdgeType::Includes,
            },
            JettyEdge {
                to: NodeName::Group {
                    name: "Group b".to_string(),
                    origin: Default::default(),
                },
                from: NodeName::Group {
                    name: "Group 1".to_string(),
                    origin: Default::default(),
                },
                edge_type: EdgeType::MemberOf,
            },
            JettyEdge {
                from: NodeName::Group {
                    name: "Group b".to_string(),
                    origin: Default::default(),
                },
                to: NodeName::Group {
                    name: "Group 1".to_string(),
                    origin: Default::default(),
                },
                edge_type: EdgeType::Includes,
            },
            JettyEdge {
                from: NodeName::Group {
                    name: "Group 1".to_string(),
                    origin: Default::default(),
                },
                to: NodeName::User("User a".to_string()),
                edge_type: EdgeType::Includes,
            },
            JettyEdge {
                from: NodeName::User("User a".to_string()),
                to: NodeName::Group {
                    name: "Group 1".to_string(),
                    origin: Default::default(),
                },
                edge_type: EdgeType::MemberOf,
            },
            JettyEdge {
                from: NodeName::Group {
                    name: "Group 1".to_string(),
                    origin: Default::default(),
                },
                to: NodeName::Group {
                    name: "Group c".to_string(),
                    origin: Default::default(),
                },
                edge_type: EdgeType::Includes,
            },
            JettyEdge {
                from: NodeName::Group {
                    name: "Group c".to_string(),
                    origin: Default::default(),
                },
                to: NodeName::Group {
                    name: "Group 1".to_string(),
                    origin: Default::default(),
                },
                edge_type: EdgeType::MemberOf,
            },
            JettyEdge {
                from: NodeName::Group {
                    name: "Group 1".to_string(),
                    origin: Default::default(),
                },
                to: NodeName::Policy {
                    name: "Policy 1".to_string(),
                    origin: Default::default(),
                },
                edge_type: EdgeType::GrantedFrom,
            },
            JettyEdge {
                from: NodeName::Policy {
                    name: "Policy 1".to_string(),
                    origin: Default::default(),
                },
                to: NodeName::Group {
                    name: "Group 1".to_string(),
                    origin: Default::default(),
                },
                edge_type: EdgeType::GrantedTo,
            },
        ]);

        ag.register_nodes_and_edges(&input_group)?;
        assert_eq!(ag.edge_cache, output_edges);
        Ok(())
    }
}
