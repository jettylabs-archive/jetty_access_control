//! Nodes to be recieved from connectors
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
};

use derivative::Derivative;
use serde::{Deserialize, Serialize};

use crate::cual::Cual;

use super::AssetType;

/// Alias for a sparse matrix addressable by matrix\[x\]\[y\], where each entry is of type T.
pub type SparseMatrix<X, Y, T> = HashMap<X, HashMap<Y, T>>;

/// Mode for a permission that's either "allow," "deny," "none," or something
/// else with a given explanation.
#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Serialize, Deserialize, PartialOrd, Ord)]
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
            other => PermissionMode::Other(other.to_owned()),
        }
    }
}
/// An effective permission
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialOrd, Ord, Eq)]
pub struct EffectivePermission {
    /// The privilege granted/denied for this permission.
    pub privilege: String,
    /// The mode for this permission.
    pub mode: PermissionMode,
    /// The human-readable reasons this permission was applied.
    pub reasons: Vec<String>,
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

/// `EffectivePermission`s are hashed solely on the privilege.
///
/// This means that
///
/// ```text
///  EffectivePermission{
///   privilege: "read",
///   mode: Allow,
///   reasons: []
/// }
/// ```
///
/// and
///
///
/// ```text
///  EffectivePermission{
///   privilege: "read",
///   mode: Deny,
///   reasons: ["some reasons"]
/// }
/// ```
///
/// are a hash collision and need to be merged to appropriately combine them.
impl std::hash::Hash for EffectivePermission {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.privilege.hash(state);
    }
}

/// `EffectivePermission`s are hashed solely on the privilege.
///
/// This means that
///
/// ```text
///  EffectivePermission{
///   privilege: "read",
///   mode: Allow,
///   reasons: []
/// }
/// ```
///
/// and
///
///
/// ```text
///  EffectivePermission{
///   privilege: "read",
///   mode: Deny,
///   reasons: ["some reasons"]
/// }
/// ```
///
/// are considered equal.
impl PartialEq for EffectivePermission {
    fn eq(&self, other: &Self) -> bool {
        self.privilege == other.privilege
    }
}

type UserName = String;
/// Container for all node data for a given connector
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ConnectorData {
    /// All groups in the connector
    pub groups: Vec<RawGroup>,
    /// All users in the connector
    pub users: Vec<RawUser>,
    /// All assets in the connector
    pub assets: Vec<RawAsset>,
    /// All tags in the connector
    pub tags: Vec<RawTag>,
    /// All policies in the connector
    pub policies: Vec<RawPolicy>,
    /// The default policies provided by the connector
    pub default_policies: Vec<RawDefaultPolicy>,
    /// References to assets that are owned by another connector
    pub asset_references: Vec<RawAssetReference>,
    /// Mapping of all users to the assets they have permissions granted
    /// to.
    ///
    /// `effective_permissions["user_identifier"]["asset://cual"]` would contain the effective
    /// permissions for that user,asset combination, with one EffectivePermission
    /// per privilege containing possible explanations.
    pub effective_permissions: SparseMatrix<UserName, Cual, HashSet<EffectivePermission>>,
    /// The globally unique cual prefix that can be used to match cuals to a namespace
    pub cual_prefix: Option<String>,
}

impl ConnectorData {
    /// Basic constructor
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        groups: Vec<RawGroup>,
        users: Vec<RawUser>,
        assets: Vec<RawAsset>,
        tags: Vec<RawTag>,
        policies: Vec<RawPolicy>,
        default_policies: Vec<RawDefaultPolicy>,
        asset_references: Vec<RawAssetReference>,
        effective_permissions: SparseMatrix<UserName, Cual, HashSet<EffectivePermission>>,
        cual_prefix: Option<String>,
    ) -> Self {
        Self {
            groups,
            users,
            assets,
            asset_references,
            tags,
            policies,
            default_policies,
            effective_permissions,
            cual_prefix,
        }
    }
}

#[derive(Default, Debug, PartialEq, Eq)]
/// Group data provided by connectors
pub struct RawGroup {
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

impl RawGroup {
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
pub struct RawUser {
    /// The name of the user. When coming from a connector, this
    /// should be the name the connector uses to refer to a person.
    /// When sent to the graph, it should be the Jetty identifier for
    /// the user (which may be different)
    pub name: String,
    /// Additional user identifiers that are used to resolve users
    /// cross-platform
    pub identifiers: HashSet<super::UserIdentifier>,
    /// K-V pairs of user-specific metadata. When sent to the graph
    /// the keys should be namespaced (e.g. `snow::key : value`)
    pub metadata: HashMap<String, String>,
    /// IDs of the groups this user is a member of
    pub member_of: HashSet<String>,
    /// IDs of policies that are applied to this user
    pub granted_by: HashSet<String>,
}

impl RawUser {
    /// Basic constructor.
    pub fn new(
        name: String,
        identifiers: HashSet<super::UserIdentifier>,
        metadata: HashMap<String, String>,
        member_of: HashSet<String>,
        granted_by: HashSet<String>,
    ) -> Self {
        Self {
            name,
            identifiers,
            metadata,
            member_of,
            granted_by,
        }
    }
}

/// Struct used to populate asset nodes and edges in the graph
#[derive(Default, PartialEq, Eq, Debug)]
pub struct RawAsset {
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

impl RawAsset {
    /// Basic constructor.
    #[allow(clippy::too_many_arguments)]
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

impl Ord for RawAsset {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cual.uri().cmp(&other.cual.uri())
    }
}

impl PartialOrd for RawAsset {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Struct used to populate asset nodes and edges in the graph
#[derive(Default, PartialEq, Eq, Debug)]
pub struct RawAssetReference {
    /// Connector Universal Asset Locator
    pub cual: Cual,
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

impl RawAssetReference {
    /// Basic constructor.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cual: Cual,
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

impl Ord for RawAssetReference {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cual.uri().cmp(&other.cual.uri())
    }
}

impl PartialOrd for RawAssetReference {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Struct used to populate tag nodes and edges in the graph
#[derive(Debug, PartialEq, Eq, Default)]
pub struct RawTag {
    /// context
    pub name: String,
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
    pub applied_to: HashSet<String>,
    /// IDs of assets the tag is applied to
    pub removed_from: HashSet<String>,
    /// IDs of policies that are applied to this asset
    pub governed_by: HashSet<String>,
}

/// Struct used to populate policy nodes and edges in the graph
#[derive(Debug, Derivative, Clone, PartialEq, Eq, Default)]
pub struct RawPolicy {
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
    pub pass_through_hierarchy: bool,
    /// Whether the policy also applies to derived assets
    pub pass_through_lineage: bool,
}

impl RawPolicy {
    /// Basic constructor.
    #[allow(clippy::too_many_arguments)]
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

/// Struct used to populate default policy nodes and edges in the graph. Must be returned
/// from the connector as a single policy that can be keyed off the asset_path and
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawDefaultPolicy {
    /// Privileges applied as part of this policy
    pub privileges: HashSet<String>,
    /// The cual of the asset that the policy originates from
    pub root_asset: Cual,
    /// The wildcard path to assets that will be affected by this policy (e.g. "*/**" )
    pub wildcard_path: String,
    /// The type that the policy should be applied to
    pub target_type: AssetType,
    /// policy grantee
    pub grantee: RawPolicyGrantee,
    /// metadata for the policy
    pub metadata: HashMap<String, String>,
}

/// Grantee of a policy
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RawPolicyGrantee {
    /// Grantee of a group
    Group(String),
    /// Grantee of a user
    User(String),
}
