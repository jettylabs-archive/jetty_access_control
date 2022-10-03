//! Nodes to be recieved from connectors
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
};

use derivative::Derivative;

use crate::cual::Cual;

use super::UserIdentifier;

/// Alias for a sparse matrix addressable by matrix[x][y], where each entry is of type T.
pub type SparseMatrix<X, Y, T> = HashMap<X, HashMap<Y, T>>;

/// Mode for a permission that's either "allow," "deny," "none," or something
/// else with a given explanation.
#[derive(Debug, Default, Hash, PartialEq, Eq)]
pub enum PermissionMode {
    /// Allow this permission.
    Allow,
    /// Deny this permission.
    Deny,
    /// No permission set.
    #[default]
    None,
    /// Permission set to something else with a contained explanation.
    Other(String),
}

impl From<&str> for PermissionMode {
    fn from(val: &str) -> Self {
        match val.to_lowercase().as_str() {
            "allow" => PermissionMode::Allow,
            "deny" => PermissionMode::Deny,
            "none" => PermissionMode::None,
            other => PermissionMode::Other(other.to_owned()),
        }
    }
}
/// An effective permission
#[derive(Debug, Default, Hash, PartialEq, Eq)]
pub struct EffectivePermission {
    privilege: String,
    mode: PermissionMode,
    reasons: Vec<String>,
}

impl EffectivePermission {
    /// Basic constructor
    pub fn new(privilege: String, mode: PermissionMode, reasons: Vec<String>) -> Self {
        Self {
            privilege,
            mode,
            reasons,
        }
    }
}

/// Container for all node data for a given connector
#[derive(Debug, Default, PartialEq, Eq)]
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
    /// Mapping of all users to the assets they have permissions granted
    /// to.
    ///
    /// `effective_permissions["user_identifier"]["asset://cual"]` would contain the effective
    /// permissions for that user,asset combination, with one EffectivePermission
    /// per privilege containing possible explanations.
    pub effective_permissions: SparseMatrix<UserIdentifier, Cual, HashSet<EffectivePermission>>,
}

impl ConnectorData {
    /// Basic constructor
    pub fn new(
        groups: Vec<Group>,
        users: Vec<User>,
        assets: Vec<Asset>,
        tags: Vec<Tag>,
        policies: Vec<Policy>,
        effective_permissions: SparseMatrix<UserIdentifier, Cual, HashSet<EffectivePermission>>,
    ) -> Self {
        Self {
            groups,
            users,
            assets,
            tags,
            policies,
            effective_permissions,
        }
    }
}

#[derive(Default, Debug, PartialEq, Eq)]
/// Group data provided by connectors
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

impl Group {
    /// Basic constructor.
    pub fn new(
        name: String,
        metadata: HashMap<String, String>,
        member_of: HashSet<String>,
        includes_users: HashSet<String>,
        includes_groups: HashSet<String>,
        granted_by: HashSet<String>,
    ) -> Self {
        Self {
            name,
            metadata,
            member_of,
            includes_users,
            includes_groups,
            granted_by,
        }
    }
}

/// User data provided by connectors
#[derive(Default, Debug, PartialEq, Eq)]
pub struct User {
    /// The name of the user. When coming from a connector, this
    /// should be the name the connector uses to refer to a person.
    /// When sent to the graph, it should be the Jetty identifier for
    /// the user (which may be different)
    pub name: String,
    /// Additional user identifiers that are used to resolve users
    /// cross-platform
    pub identifiers: HashSet<super::UserIdentifier>,
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

impl User {
    /// Basic constructor.
    pub fn new(
        name: String,
        identifiers: HashSet<super::UserIdentifier>,
        other_identifiers: HashSet<String>,
        metadata: HashMap<String, String>,
        member_of: HashSet<String>,
        granted_by: HashSet<String>,
    ) -> Self {
        Self {
            name,
            identifiers,
            other_identifiers,
            metadata,
            member_of,
            granted_by,
        }
    }
}

/// Struct used to populate asset nodes and edges in the graph
#[derive(Default, PartialEq, Eq, Debug)]
pub struct Asset {
    /// Connector Universal Asset Locator
    pub cual: Cual,
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

impl Asset {
    /// Basic constructor.
    pub fn new(
        cual: Cual,
        name: String,
        asset_type: super::AssetType,
        metadata: HashMap<String, String>,
        governed_by: HashSet<String>,
        child_of: HashSet<String>,
        parent_of: HashSet<String>,
        derived_from: HashSet<String>,
        derived_to: HashSet<String>,
        tagged_as: HashSet<String>,
    ) -> Self {
        Self {
            cual,
            name,
            asset_type,
            metadata,
            governed_by,
            child_of,
            parent_of,
            derived_from,
            derived_to,
            tagged_as,
        }
    }
}

impl Ord for Asset {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cual.uri().cmp(&other.cual.uri())
    }
}

impl PartialOrd for Asset {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Struct used to populate tag nodes and edges in the graph
#[derive(Debug, Derivative, PartialEq, Eq)]
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
#[derive(Debug, Derivative, Clone, PartialEq, Eq)]
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

impl Policy {
    /// Basic constructor.
    pub fn new(
        name: String,
        privileges: HashSet<String>,
        governs_assets: HashSet<String>,
        governs_tags: HashSet<String>,
        granted_to_groups: HashSet<String>,
        granted_to_users: HashSet<String>,
        pass_through_hierarchy: bool,
        pass_through_lineage: bool,
    ) -> Self {
        Self {
            name,
            privileges,
            governs_assets,
            governs_tags,
            granted_to_groups,
            granted_to_users,
            pass_through_hierarchy,
            pass_through_lineage,
        }
    }
}
