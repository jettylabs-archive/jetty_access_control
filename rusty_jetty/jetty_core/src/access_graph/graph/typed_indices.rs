//! Typed graph indices
//!
use crate::access_graph::{
    AccessGraph, AssetAttributes, GroupAttributes, NodeIndex, TagAttributes, UserAttributes,
};

pub(crate) trait ToNodeIndex {
    fn get_index(&self) -> NodeIndex;
}

impl ToNodeIndex for NodeIndex {
    fn get_index(&self) -> NodeIndex {
        *self
    }
}

/// Implements the ToNodeIndex trait for one or more types that have an `idx` field.
macro_rules! impl_to_node_index {
    (for $($t:ty),+) => {
        $(impl ToNodeIndex for $t {
            fn get_index(&self) -> NodeIndex {
                self.idx
            }
        })*
    }
}

impl_to_node_index!(for AssetIndex, UserIndex, TagIndex, GroupIndex);

/// Index to an Asset node in the AccessGraph
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
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
}

/// Index to an User node in the AccessGraph
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
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
}

/// Index to an Group node in the AccessGraph
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct GroupIndex {
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
}

/// Index to an Tag node in the AccessGraph
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct TagIndex {
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
}
