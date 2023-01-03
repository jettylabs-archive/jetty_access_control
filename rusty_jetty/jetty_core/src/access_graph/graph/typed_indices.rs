//! Typed graph indices
//!
use std::collections::HashSet;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::{
    access_graph::{
        AssetAttributes, DefaultPolicyAttributes, EdgeType, GroupAttributes, JettyNode, NodeIndex,
        NodeName, PolicyAttributes, TagAttributes, UserAttributes,
    },
    Jetty,
};

pub(crate) trait TypedIndex {
    /// Return a result for whether the typed index is valid
    /// The index must exist and the node type must be correct
    fn is_valid(&self, jetty: &Jetty) -> Result<()>;
    /// Returns the nodes node index
    fn idx(&self) -> NodeIndex;
    /// Returns the node name
    fn name(&self, jetty: &Jetty) -> Result<NodeName> {
        let ag = jetty.try_access_graph()?;
        Ok(ag[self.idx()].get_node_name())
    }
}

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

impl_to_node_index!(for AssetIndex, UserIndex, TagIndex, GroupIndex, PolicyIndex, DefaultPolicyIndex);

/// Implements the TypedIndex trait for one or more types that have an `idx` field and a get_attributes method
macro_rules! impl_typed_index_trait{
    (for $($t:ty),+) => {
        $(impl TypedIndex for $t {
            fn idx(&self) -> NodeIndex {
                self.idx
            }

            fn is_valid(&self, jetty: &Jetty) -> Result<()> {
                self.get_attributes(jetty)?;
                Ok(())
            }
        })*
    }
}

impl_typed_index_trait!(for AssetIndex, UserIndex, TagIndex, GroupIndex, PolicyIndex, DefaultPolicyIndex);

/// Index to an Asset node in the AccessGraph
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AssetIndex {
    idx: NodeIndex,
}

impl AssetIndex {
    /// get reference ot the AssetAttributes corresponding to the node index
    pub fn get_attributes<'a>(&self, jetty: &'a Jetty) -> Result<&'a AssetAttributes> {
        let ag = jetty.try_access_graph()?;
        let x = &ag[*self];
        match x {
            crate::access_graph::JettyNode::Asset(a) => Ok(a),
            _ => bail!("mismatch in node type; expected asset"),
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
    pub fn get_attributes<'a>(&self, jetty: &'a Jetty) -> Result<&'a UserAttributes> {
        let ag = jetty.try_access_graph()?;
        let x = &ag[*self];
        match x {
            crate::access_graph::JettyNode::User(a) => Ok(a),
            _ => bail!("mismatch in node type; expected asset"),
        }
    }
    pub(crate) fn new(idx: NodeIndex) -> Self {
        UserIndex { idx }
    }

    /// Get the groups that this user is a direct member of
    pub fn member_of_groups(&self, jetty: &Jetty) -> Result<HashSet<GroupIndex>> {
        let ag = jetty.try_access_graph()?;
        Ok(ag
            .get_matching_children(
                self.idx,
                |e| matches!(e, EdgeType::MemberOf),
                |n| matches!(n, JettyNode::Group(_)),
            )
            .into_iter()
            .map(GroupIndex::new)
            .collect())
    }
}

/// Index to an Group node in the AccessGraph
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GroupIndex {
    idx: NodeIndex,
}
impl GroupIndex {
    /// get reference ot the GroupAttributes corresponding to the node index
    pub fn get_attributes<'a>(&self, jetty: &'a Jetty) -> Result<&'a GroupAttributes> {
        let ag = jetty.try_access_graph()?;
        let x = &ag[*self];
        match x {
            crate::access_graph::JettyNode::Group(a) => Ok(a),
            _ => bail!("mismatch in node type; expected asset"),
        }
    }
    pub(crate) fn new(idx: NodeIndex) -> Self {
        GroupIndex { idx }
    }

    /// Groups which are members of self
    pub fn member_of(&self, jetty: &Jetty) -> Result<HashSet<GroupIndex>> {
        let ag = jetty.try_access_graph()?;
        Ok(ag
            .get_matching_children(
                self.idx,
                |e| matches!(e, EdgeType::MemberOf),
                |n| matches!(n, JettyNode::Group(_)),
            )
            .into_iter()
            .map(GroupIndex::new)
            .collect())
    }
}

/// Index to an Tag node in the AccessGraph
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TagIndex {
    idx: NodeIndex,
}
impl TagIndex {
    /// get reference ot the TagAttributes corresponding to the node index
    pub fn get_attributes<'a>(&self, jetty: &'a Jetty) -> Result<&'a TagAttributes> {
        let ag = jetty.try_access_graph()?;
        let x = &ag[*self];
        match x {
            crate::access_graph::JettyNode::Tag(a) => Ok(a),
            _ => bail!("mismatch in node type; expected asset"),
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
    pub fn get_attributes<'a>(&self, jetty: &'a Jetty) -> Result<&'a PolicyAttributes> {
        let ag = jetty.try_access_graph()?;
        let x = &ag[*self];
        match x {
            crate::access_graph::JettyNode::Policy(a) => Ok(a),
            _ => bail!("mismatch in node type; expected asset"),
        }
    }
    pub(crate) fn new(idx: NodeIndex) -> Self {
        PolicyIndex { idx }
    }
}

/// Index to an Policy node in the AccessGraph
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DefaultPolicyIndex {
    idx: NodeIndex,
}
impl DefaultPolicyIndex {
    /// get reference ot the PolicyAttributes corresponding to the node index
    pub fn get_attributes<'a>(&self, jetty: &'a Jetty) -> Result<&'a DefaultPolicyAttributes> {
        let ag = jetty.try_access_graph()?;
        let x = &ag[*self];
        match x {
            crate::access_graph::JettyNode::DefaultPolicy(a) => Ok(a),
            _ => bail!("mismatch in node type; expected asset"),
        }
    }
    pub(crate) fn new(idx: NodeIndex) -> Self {
        DefaultPolicyIndex { idx }
    }
}
