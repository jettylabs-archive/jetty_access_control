//! Types and functions for processed nodes. Processed nodes are used after the translation layer - all
//! references to other nodes have been converted to NodeNames

use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
};

use uuid::Uuid;

use crate::{
    access_graph::{
        helpers::{insert_edge_pair, NodeHelper},
        AccessGraph, AssetAttributes, DefaultPolicyAttributes, EdgeType, GroupAttributes,
        JettyEdge, JettyNode, NodeName, PolicyAttributes, TagAttributes, UserAttributes,
    },
    jetty::ConnectorNamespace,
    logging::error,
};

use super::{
    nodes::{EffectivePermission, SparseMatrix},
    AssetType,
};

/// Container for all node data for a given connector
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct ProcessedConnectorData {
    /// All groups in the connector
    pub groups: Vec<ProcessedGroup>,
    /// All users in the connector
    pub users: Vec<ProcessedUser>,
    /// All assets in the connector
    pub assets: Vec<ProcessedAsset>,
    /// All tags in the connector
    pub tags: Vec<ProcessedTag>,
    /// All policies in the connector
    pub policies: Vec<ProcessedPolicy>,
    /// Default policies from the connector
    pub default_policies: Vec<ProcessedDefaultPolicy>,
    /// All references to un-owned assets. Only necessary
    pub asset_references: Vec<ProcessedAssetReference>,
    /// Mapping of all users to the assets they have permissions granted
    /// to.
    ///
    /// `effective_permissions["user_identifier"]["asset://cual"]` would contain the effective
    /// permissions for that user,asset combination, with one EffectivePermission
    /// per privilege containing possible explanations.
    pub effective_permissions: SparseMatrix<NodeName, NodeName, HashSet<EffectivePermission>>,
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
/// Group data provided by connectors
pub struct ProcessedGroup {
    /// Group name
    pub name: NodeName,
    /// K-V pairs of group-specific metadata. When sent to the graph
    /// the keys should be namespaced (e.g. `snow::key : value`)
    pub metadata: HashMap<String, String>,
    /// IDs of the groups this group is a member of
    pub member_of: HashSet<NodeName>,
    /// IDs of users that are members of this group
    pub includes_users: HashSet<NodeName>,
    /// IDs of groups that are members of this group
    pub includes_groups: HashSet<NodeName>,
    /// IDs of policies that are applied to this group
    pub granted_by: HashSet<NodeName>,
    /// Names of connector
    pub connector: ConnectorNamespace,
}

/// User data provided by connectors
#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct ProcessedUser {
    /// The name of the user. When coming from a connector, this
    /// should be the name the connector uses to refer to a person.
    /// When sent to the graph, it should be the Jetty identifier for
    /// the user (which may be different)
    pub name: NodeName,
    /// Additional user identifiers that are used to resolve users
    /// cross-platform
    pub identifiers: HashSet<super::UserIdentifier>,
    /// K-V pairs of user-specific metadata. When sent to the graph
    /// the keys should be namespaced (e.g. `snow::key : value`)
    pub metadata: HashMap<String, String>,
    /// IDs of the groups this user is a member of
    pub member_of: HashSet<NodeName>,
    /// IDs of policies that are applied to this user
    pub granted_by: HashSet<NodeName>,
    /// Names of connector
    pub connector: ConnectorNamespace,
}

/// Struct used to populate asset nodes and edges in the graph
#[derive(Default, PartialEq, Eq, Debug, Clone)]
pub struct ProcessedAsset {
    /// Connector Universal Asset Locator
    pub name: NodeName,
    /// Type of asset being modeled
    pub asset_type: super::AssetType,
    /// K-V pairs of asset-specific metadata. When sent to the graph
    /// the keys should be namespaced (e.g. `snow::key : value`)
    pub metadata: HashMap<String, String>,
    /// IDs of policies that govern this asset.
    /// Jetty will dedup these with Policy.governs_assets.
    pub governed_by: HashSet<NodeName>,
    /// IDs of hierarchical children of the asset
    pub child_of: HashSet<NodeName>,
    /// IDs of hierarchical parents of the asset
    pub parent_of: HashSet<NodeName>,
    /// IDs of assets this asset is derived from
    pub derived_from: HashSet<NodeName>,
    /// IDs of assets that are derived from this one
    pub derived_to: HashSet<NodeName>,
    /// IDs of tags associated with this asset
    pub tagged_as: HashSet<NodeName>,
    /// Names of connector
    pub connector: ConnectorNamespace,
}

impl Ord for ProcessedAsset {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for ProcessedAsset {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Struct used to populate connections edges to/from asset nodes that are owned by another connector
#[derive(Default, PartialEq, Eq, Debug, Clone)]
pub struct ProcessedAssetReference {
    /// Connector Universal Asset Locator
    pub name: NodeName,
    /// Type of asset being modeled
    pub metadata: HashMap<String, String>,
    /// IDs of policies that govern this asset.
    /// Jetty will dedup these with Policy.governs_assets.
    pub governed_by: HashSet<NodeName>,
    /// IDs of hierarchical children of the asset
    pub child_of: HashSet<NodeName>,
    /// IDs of hierarchical parents of the asset
    pub parent_of: HashSet<NodeName>,
    /// IDs of assets this asset is derived from
    pub derived_from: HashSet<NodeName>,
    /// IDs of assets that are derived from this one
    pub derived_to: HashSet<NodeName>,
    /// IDs of tags associated with this asset
    pub tagged_as: HashSet<NodeName>,
}

impl Ord for ProcessedAssetReference {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for ProcessedAssetReference {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Struct used to populate tag nodes and edges in the graph
#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct ProcessedTag {
    /// context
    pub name: NodeName,
    /// Optional value for the tag (for the case of key-value tags)
    pub value: Option<String>,
    /// Optional description for the tag
    pub description: Option<String>,
    /// Whether the tag is to be passed through asset hierarchy (only to direct
    /// descendants of this node)
    pub pass_through_hierarchy: bool,
    /// Whether the tag is to be passed through asset lineage (only to direct
    /// descendants of this node)
    pub pass_through_lineage: bool,
    /// IDs of assets the tag is applied to
    pub applied_to: HashSet<NodeName>,
    /// IDs of assets the tag is applied to
    pub removed_from: HashSet<NodeName>,
    /// IDs of policies that are applied to this asset
    pub governed_by: HashSet<NodeName>,
    /// Names of connector
    pub connector: ConnectorNamespace,
}

/// Struct used to populate policy nodes and edges in the graph
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProcessedPolicy {
    /// ID of the Policy, namespaced for the relevant context
    pub name: NodeName,
    /// Privileges associated with the policy, scoped to
    /// relevant context
    pub privileges: HashSet<String>,
    /// IDs of assets governed by the policy
    pub governs_assets: HashSet<NodeName>,
    /// IDs of tags governed by the policy
    pub governs_tags: HashSet<NodeName>,
    /// IDs or goups the policy is applied to
    pub granted_to_groups: HashSet<NodeName>,
    /// IDs of users the policy is applied to
    pub granted_to_users: HashSet<NodeName>,
    /// Whether the policy also applies to child assets
    pub pass_through_hierarchy: bool,
    /// Whether the policy also applies to derived assets
    pub pass_through_lineage: bool,
    /// Names of connector
    pub connector: ConnectorNamespace,
}

impl NodeHelper for ProcessedGroup {
    fn get_node(&self) -> Option<JettyNode> {
        Some(JettyNode::Group(GroupAttributes {
            name: self.name.to_owned(),
            id: Uuid::new_v5(&Uuid::NAMESPACE_URL, self.name.to_string().as_bytes()),
            metadata: self.metadata.to_owned(),
            connectors: HashSet::from([self.connector.to_owned()]),
        }))
    }

    fn get_edges(&self) -> HashSet<JettyEdge> {
        let mut hs = HashSet::<JettyEdge>::new();
        for v in &self.member_of {
            insert_edge_pair(
                &mut hs,
                self.name.to_owned(),
                v.to_owned(),
                EdgeType::MemberOf,
            );
        }
        for v in &self.includes_users {
            insert_edge_pair(
                &mut hs,
                self.name.to_owned(),
                v.to_owned(),
                EdgeType::Includes,
            );
        }
        for v in &self.includes_groups {
            insert_edge_pair(
                &mut hs,
                self.name.to_owned(),
                v.to_owned(),
                EdgeType::Includes,
            );
        }
        for v in &self.granted_by {
            insert_edge_pair(
                &mut hs,
                self.name.to_owned(),
                v.to_owned(),
                EdgeType::GrantedFrom,
            );
        }
        hs
    }
}

/// Struct used to populate default policy nodes and edges in the graph
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProcessedDefaultPolicy {
    /// Tricky one...
    pub name: NodeName,
    /// Privileges associated with the policy, scoped to
    /// relevant context
    pub privileges: HashSet<String>,
    /// IDs of assets governed by the policy
    pub root_node: NodeName,
    /// Path to determine scope
    pub matching_path: String,
    /// The types of assets to apply this to
    pub target_type: AssetType,
    /// Who the privilege is granted to
    pub grantee: NodeName,
    /// The metadata associated with the policy
    pub metadata: HashMap<String, String>,
    /// Connector that the privilege exists in
    pub connector: ConnectorNamespace,
}

/// Object used to populate user nodes and edges in the graph

impl NodeHelper for ProcessedUser {
    fn get_node(&self) -> Option<JettyNode> {
        Some(JettyNode::User(UserAttributes::new(
            &self.name,
            &self.identifiers,
            &self.metadata,
            Some(&self.connector),
        )))
    }

    fn get_edges(&self) -> HashSet<JettyEdge> {
        let mut hs = HashSet::<JettyEdge>::new();
        for v in &self.member_of {
            insert_edge_pair(
                &mut hs,
                self.name.to_owned(),
                v.to_owned(),
                EdgeType::MemberOf,
            );
        }
        for v in &self.granted_by {
            insert_edge_pair(
                &mut hs,
                self.name.to_owned(),
                v.to_owned(),
                EdgeType::GrantedFrom,
            );
        }
        hs
    }
}

impl NodeHelper for ProcessedAsset {
    fn get_node(&self) -> Option<JettyNode> {
        Some(JettyNode::Asset(AssetAttributes {
            name: self.name.to_owned(),
            id: Uuid::new_v5(&Uuid::NAMESPACE_URL, self.name.to_string().as_bytes()),
            asset_type: self.asset_type.to_owned(),
            metadata: self.metadata.to_owned(),
            connectors: HashSet::from([self.connector.to_owned()]),
        }))
    }

    fn get_edges(&self) -> HashSet<JettyEdge> {
        let mut hs = HashSet::<JettyEdge>::new();
        for v in &self.governed_by {
            insert_edge_pair(
                &mut hs,
                self.name.to_owned(),
                v.to_owned(),
                EdgeType::GovernedBy,
            );
        }
        for v in &self.child_of {
            insert_edge_pair(
                &mut hs,
                self.name.to_owned(),
                v.to_owned(),
                EdgeType::ChildOf,
            );
        }
        for v in &self.parent_of {
            insert_edge_pair(
                &mut hs,
                self.name.to_owned(),
                v.to_owned(),
                EdgeType::ParentOf,
            );
        }
        for v in &self.derived_from {
            insert_edge_pair(
                &mut hs,
                self.name.to_owned(),
                v.to_owned(),
                EdgeType::DerivedFrom,
            );
        }
        for v in &self.derived_to {
            insert_edge_pair(
                &mut hs,
                self.name.to_owned(),
                v.to_owned(),
                EdgeType::DerivedTo,
            );
        }
        for v in &self.tagged_as {
            insert_edge_pair(
                &mut hs,
                self.name.to_owned(),
                v.to_owned(),
                EdgeType::TaggedAs,
            );
        }
        hs
    }
}

impl NodeHelper for ProcessedAssetReference {
    fn get_node(&self) -> Option<JettyNode> {
        None
    }

    fn get_edges(&self) -> HashSet<JettyEdge> {
        let mut hs = HashSet::<JettyEdge>::new();
        for v in &self.governed_by {
            insert_edge_pair(
                &mut hs,
                self.name.to_owned(),
                v.to_owned(),
                EdgeType::GovernedBy,
            );
        }
        for v in &self.child_of {
            insert_edge_pair(
                &mut hs,
                self.name.to_owned(),
                v.to_owned(),
                EdgeType::ChildOf,
            );
        }
        for v in &self.parent_of {
            insert_edge_pair(
                &mut hs,
                self.name.to_owned(),
                v.to_owned(),
                EdgeType::ParentOf,
            );
        }
        for v in &self.derived_from {
            insert_edge_pair(
                &mut hs,
                self.name.to_owned(),
                v.to_owned(),
                EdgeType::DerivedFrom,
            );
        }
        for v in &self.derived_to {
            insert_edge_pair(
                &mut hs,
                self.name.to_owned(),
                v.to_owned(),
                EdgeType::DerivedTo,
            );
        }
        for v in &self.tagged_as {
            insert_edge_pair(
                &mut hs,
                self.name.to_owned(),
                v.to_owned(),
                EdgeType::TaggedAs,
            );
        }
        hs
    }
}

impl NodeHelper for ProcessedTag {
    fn get_node(&self) -> Option<JettyNode> {
        Some(JettyNode::Tag(TagAttributes {
            name: self.name.to_owned(),
            id: Uuid::new_v5(&Uuid::NAMESPACE_URL, self.name.to_string().as_bytes()),
            value: self.value.to_owned(),
            description: self.description.to_owned(),
            pass_through_hierarchy: self.pass_through_hierarchy,
            pass_through_lineage: self.pass_through_lineage,
            connectors: HashSet::from([self.connector.to_owned()]),
        }))
    }

    fn get_edges(&self) -> HashSet<JettyEdge> {
        let mut hs = HashSet::<JettyEdge>::new();
        for v in &self.applied_to {
            insert_edge_pair(
                &mut hs,
                self.name.to_owned(),
                v.to_owned(),
                EdgeType::AppliedTo,
            );
        }
        for v in &self.governed_by {
            insert_edge_pair(
                &mut hs,
                self.name.to_owned(),
                v.to_owned(),
                EdgeType::GovernedBy,
            );
        }
        for v in &self.removed_from {
            insert_edge_pair(
                &mut hs,
                self.name.to_owned(),
                v.to_owned(),
                EdgeType::RemovedFrom,
            );
        }
        hs
    }
}

impl NodeHelper for ProcessedPolicy {
    fn get_node(&self) -> Option<JettyNode> {
        Some(JettyNode::Policy(PolicyAttributes {
            name: self.name.to_owned(),
            id: Uuid::new_v5(&Uuid::NAMESPACE_URL, self.name.to_string().as_bytes()),
            privileges: self.privileges.to_owned(),
            pass_through_hierarchy: self.pass_through_hierarchy,
            pass_through_lineage: self.pass_through_lineage,
            connectors: HashSet::from([self.connector.to_owned()]),
        }))
    }

    fn get_edges(&self) -> HashSet<JettyEdge> {
        let mut hs = HashSet::<JettyEdge>::new();
        for v in &self.governs_assets {
            insert_edge_pair(
                &mut hs,
                self.name.to_owned(),
                v.to_owned(),
                EdgeType::Governs,
            );
        }
        for v in &self.governs_tags {
            insert_edge_pair(
                &mut hs,
                self.name.to_owned(),
                v.to_owned(),
                EdgeType::Governs,
            );
        }
        for v in &self.granted_to_groups {
            insert_edge_pair(
                &mut hs,
                self.name.to_owned(),
                v.to_owned(),
                EdgeType::GrantedTo,
            );
        }
        for v in &self.granted_to_users {
            insert_edge_pair(
                &mut hs,
                self.name.to_owned(),
                v.to_owned(),
                EdgeType::GrantedTo,
            );
        }
        hs
    }
}

impl ProcessedDefaultPolicy {
    /// Gets a jetty node, based on a ProcessedDefaultPolicy
    pub(crate) fn get_node(&self) -> Option<JettyNode> {
        Some(JettyNode::DefaultPolicy(DefaultPolicyAttributes {
            name: NodeName::DefaultPolicy {
                root_node: Box::new(self.root_node.to_owned()),
                matching_path: self.matching_path.to_owned(),
                target_type: self.target_type.to_owned(),
                grantee: Box::new(self.grantee.to_owned()),
            },
            id: Uuid::new_v5(&Uuid::NAMESPACE_URL, self.name.to_string().as_bytes()),
            privileges: self.privileges.to_owned(),
            matching_path: self.matching_path.to_owned(),
            target_type: self.target_type.to_owned(),
            metadata: self.metadata.to_owned(),
            connectors: [self.connector.to_owned()].into(),
        }))
    }

    /// Gets all the edges needed to add a default policy to the graph. This requires graph traversal
    /// because we add an edge between the DefaultPolicy and every policy that it could impact.
    pub(crate) fn get_edges(&self, ag: &AccessGraph) -> HashSet<JettyEdge> {
        let mut edges = HashSet::new();

        // Insert an edge to the root node
        insert_edge_pair(
            &mut edges,
            self.name.to_owned(),
            self.root_node.to_owned(),
            EdgeType::ProvidedDefaultForChildren,
        );

        // Insert edges to all the nodes governed by the policy
        insert_edge_pair(
            &mut edges,
            self.name.to_owned(),
            self.grantee.to_owned(),
            EdgeType::GrantedTo,
        );

        // expand the path to get all relevant nodes
        let target_node_indices = match ag.default_policy_targets(&self.name) {
            Ok(i) => i,
            Err(e) => {
                error!("unable to properly build edges from default policies: {e}");
                Default::default()
            }
        };
        // Add edges to all target nodes
        edges.extend(target_node_indices.into_iter().map(|t| {
            JettyEdge::new(
                self.name.to_owned(),
                ag[t].get_node_name(),
                EdgeType::Governs,
            )
        }));

        edges
    }
}
