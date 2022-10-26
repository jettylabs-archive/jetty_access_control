//! Typed graph indices
//!
use serde::{Deserialize, Serialize};

use crate::access_graph::{
    AccessGraph, AssetAttributes, GroupAttributes, NodeIndex, PolicyAttributes, TagAttributes,
    UserAttributes,
};

/// Implements the ToNodeIndex trait for one or more types that have an `idx` field.
macro_rules! impl_to_node_index {
    (for $($t:ty),+) => {
        $(impl From<$t> for NodeIndex {
            fn from(idx: $t) -> Self {
                idx.idx
            }
        })*
    }
}

impl_to_node_index!(for AssetIndex, UserIndex, TagIndex, GroupIndex, PolicyIndex);

/// Index to an Asset node in the AccessGraph
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AssetIndex {
    idx: NodeIndex,
}

impl AssetIndex {
    /// get reference ot the AssetAttributes corresponding to the node index
    pub fn get_attributes<'a>(&self, ag: &'a AccessGraph) -> &'a AssetAttributes {
        let x = &ag[*self];
        match x {
            crate::access_graph::JettyNode::Asset(a) => a,
            _ => panic!("mismatch in node type; expected asset"),
        }
    }
    /// Create a new AssetIndex from a NodeIndex
    pub fn new(idx: NodeIndex) -> Self {
        AssetIndex { idx }
    }
}

/// Index to an User node in the AccessGraph
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UserIndex {
    idx: NodeIndex,
}

impl UserIndex {
    /// get reference ot the UserAttributes corresponding to the node index
    pub fn get_attributes<'a>(&self, ag: &'a AccessGraph) -> &'a UserAttributes {
        let x = &ag[*self];
        match x {
            crate::access_graph::JettyNode::User(a) => a,
            _ => panic!("mismatch in node type; expected user"),
        }
    }
    pub(crate) fn new(idx: NodeIndex) -> Self {
        UserIndex { idx }
    }
}

/// Index to an Group node in the AccessGraph
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GroupIndex {
    idx: NodeIndex,
}
impl GroupIndex {
    /// get reference ot the GroupAttributes corresponding to the node index
    pub fn get_attributes<'a>(&self, ag: &'a AccessGraph) -> &'a GroupAttributes {
        let x = &ag[*self];
        match x {
            crate::access_graph::JettyNode::Group(a) => a,
            _ => panic!("mismatch in node type; expected group"),
        }
    }
    pub(crate) fn new(idx: NodeIndex) -> Self {
        GroupIndex { idx }
    }
}

/// Index to an Tag node in the AccessGraph
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TagIndex {
    idx: NodeIndex,
}
impl TagIndex {
    /// get reference ot the TagAttributes corresponding to the node index
    pub fn get_attributes<'a>(&self, ag: &'a AccessGraph) -> &'a TagAttributes {
        let x = &ag[*self];
        match x {
            crate::access_graph::JettyNode::Tag(a) => a,
            _ => panic!("mismatch in node type; expected tag"),
        }
    }
    pub(crate) fn new(idx: NodeIndex) -> Self {
        TagIndex { idx }
    }
}

/// Index to an Policy node in the AccessGraph
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PolicyIndex {
    idx: NodeIndex,
}
impl PolicyIndex {
    /// get reference ot the PolicyAttributes corresponding to the node index
    pub fn get_attributes<'a>(&self, ag: &'a AccessGraph) -> &'a PolicyAttributes {
        let x = &ag[*self];
        match x {
            crate::access_graph::JettyNode::Policy(a) => a,
            _ => panic!("mismatch in node type; expected policy"),
        }
    }
    pub(crate) fn new(idx: NodeIndex) -> Self {
        PolicyIndex { idx }
    }
}
