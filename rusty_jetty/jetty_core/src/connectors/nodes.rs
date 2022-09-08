//! Nodes to be recieved from connectors
use std::collections::{HashMap, HashSet};

use derivative::Derivative;

/// Container for all node data for a given connector
#[derive(Debug, Default, PartialEq)]
pub struct ConnectorData {
    /// All groups in the connector
    pub groups: Vec<Group>,
    /// All users in the connector
    pub users: Vec<User>,
    /// All assets in the connector
    pub assets: Vec<Asset>,
    /// All tags in the connector
    pub tags: Vec<Tag>,
    /// All policies in the connector
    pub policies: Vec<Policy>,
}

/// Group data provided by connectors
#[derive(Default, Debug, PartialEq, new)]
pub struct Group {
    /// Group name
    pub name: String,
    /// K-V pairs of group-specific metadata. When sent to the graph
    /// the keys should be namespaced (e.g. `snow::key : value`)
    pub metadata: HashMap<String, String>,
    /// IDs of the groups this group is a member of
    pub member_of: HashSet<String>,
    /// IDs of users that are members of this group
    pub includes_users: HashSet<String>,
    /// IDs of groups that are members of this group
    pub includes_groups: HashSet<String>,
    /// IDs of policies that are applied to this group
    pub granted_by: HashSet<String>,
}

/// User data provided by connectors
#[derive(Default, Debug, PartialEq, new)]
pub struct User {
    /// The name of the user. When coming from a connector, this
    /// should be the name the connector uses to refer to a person.
    /// When sent to the graph, it should be the Jetty identifier for
    /// the user (which may be different)
    pub name: String,
    /// Additional user identifiers that are used to resolve users
    /// cross-platform
    pub identifiers: HashMap<super::UserIdentifier, String>,
    /// Additional identifying strings that can be used for cross-
    /// platform entity resolution
    pub other_identifiers: HashSet<String>,
    /// K-V pairs of user-specific metadata. When sent to the graph
    /// the keys should be namespaced (e.g. `snow::key : value`)
    pub metadata: HashMap<String, String>,
    /// IDs of the groups this user is a member of
    pub member_of: HashSet<String>,
    /// IDs of policies that are applied to this user
    pub granted_by: HashSet<String>,
}

/// Struct used to populate asset nodes and edges in the graph
#[derive(Default, PartialEq, Debug, new)]
pub struct Asset {
    /// Name of asset, fully qualified for the scope of use (connector)
    /// or graph.
    pub name: String,
    /// Type of asset being modeled
    pub asset_type: super::AssetType,
    /// K-V pairs of asset-specific metadata. When sent to the graph
    /// the keys should be namespaced (e.g. `snow::key : value`)
    pub metadata: HashMap<String, String>,
    /// IDs of policies that govern this asset.
    /// Jetty will dedup these with Policy.governs_assets.
    pub governed_by: HashSet<String>,
    /// IDs of hierarchical children of the asset
    pub child_of: HashSet<String>,
    /// IDs of hierarchical parents of the asset
    pub parent_of: HashSet<String>,
    /// IDs of assets this asset is derived from
    pub derived_from: HashSet<String>,
    /// IDs of assets that are derived from this one
    pub derived_to: HashSet<String>,
    /// IDs of tags associated with this asset
    pub tagged_as: HashSet<String>,
}

/// Struct used to populate tag nodes and edges in the graph
#[derive(Debug, Derivative, PartialEq)]
#[derivative(Default)]
pub struct Tag {
    /// Name of the tag, appropriately namespaced for the
    /// context
    pub name: String,
    /// Optional value for the tag (for the case of key-value tags)
    pub value: Option<String>,
    /// Whether the tag is to be passed through asset hierarchy
    #[derivative(Default(value = "true"))]
    pub pass_through_hierarchy: bool,
    /// Whether the tag is to be passed through asset lineage
    pub pass_through_lineage: bool,
    /// IDs of assets the tag is applied to
    pub applied_to: HashSet<String>,
    /// IDs of policies that are applied to this asset
    pub governed_by: HashSet<String>,
}

/// Struct used to populate policy nodes and edges in the graph
#[derive(Debug, Derivative, Clone, PartialEq, new)]
#[derivative(Default)]
pub struct Policy {
    /// ID of the Policy, namespaced for the relevant context
    pub name: String,
    /// Privileges associated with the policy, scoped to
    /// relevant context
    pub privileges: HashSet<String>,
    /// IDs of assets governed by the policy
    pub governs_assets: HashSet<String>,
    /// IDs of tags governed by the policy
    pub governs_tags: HashSet<String>,
    /// IDs or goups the policy is applied to
    pub granted_to_groups: HashSet<String>,
    /// IDs of users the policy is applied to
    pub granted_to_users: HashSet<String>,
    /// Whether the policy also applies to child assets
    #[derivative(Default(value = "true"))]
    pub pass_through_hierarchy: bool,
    /// Whether the policy also applies to derived assets
    pub pass_through_lineage: bool,
}
