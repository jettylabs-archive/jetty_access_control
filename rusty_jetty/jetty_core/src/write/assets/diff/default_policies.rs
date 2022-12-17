//! Module to diff default policies

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fmt::Display,
};

use colored::Colorize;

use crate::{
    access_graph::{AssetPath, NodeName},
    connectors::AssetType,
    jetty::ConnectorNamespace,
    write::assets::DefaultPolicyState,
};

#[derive(Debug, Clone)]
pub(crate) enum DefaultPolicyDiffDetails {
    AddDefaultPolicy {
        add: DefaultPolicyState,
    },
    RemoveDefaultPolicy,
    ModifyDefaultPolicy {
        add: DefaultPolicyState,
        remove: DefaultPolicyState,
    },
}

/// A diff of Default Policies
pub struct DefaultPolicyDiff {
    /// The name of the asset being changed
    pub(crate) root_asset: NodeName,
    /// The path to follow
    pub(crate) wildcard_path: String,
    /// The types of assets that the policy is getting configured for
    pub(crate) asset_types: BTreeSet<AssetType>,
    /// The changes that are being captured
    pub(crate) details: DefaultPolicyDiffDetails,
    pub(crate) connectors: HashSet<ConnectorNamespace>,
}
