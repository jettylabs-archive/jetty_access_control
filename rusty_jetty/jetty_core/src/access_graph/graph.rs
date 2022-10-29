//! Graph stuff
//!
pub mod typed_indices;

use anyhow::{anyhow, Context, Result};
use graphviz_rust as graphviz;
use graphviz_rust::cmd::CommandArg;
use graphviz_rust::cmd::Format;
use graphviz_rust::printer::PrinterContext;
use petgraph::stable_graph::NodeIndex;

use petgraph::{dot, stable_graph::StableDiGraph};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use self::typed_indices::AssetIndex;
use self::typed_indices::GroupIndex;
use self::typed_indices::PolicyIndex;
use self::typed_indices::TagIndex;
use self::typed_indices::UserIndex;

use super::{EdgeType, JettyNode, NodeName};
use crate::logging::warn;

/// The main graph wrapper
#[derive(Serialize, Deserialize)]
pub(crate) struct Graph {
    pub(crate) graph: StableDiGraph<JettyNode, EdgeType>,
    /// A map of node identifiers to indices
    pub(crate) nodes: NodeMap,
    /// A map of node hashes to indices
    pub(crate) node_ids: NodeIdMap,
}

/// A map of node names to typed indices
#[derive(Serialize, Deserialize, Default, Debug)]
pub(crate) struct NodeMap {
    assets: HashMap<NodeName, typed_indices::AssetIndex>,
    users: HashMap<NodeName, typed_indices::UserIndex>,
    groups: HashMap<NodeName, typed_indices::GroupIndex>,
    tags: HashMap<NodeName, typed_indices::TagIndex>,
    policies: HashMap<NodeName, typed_indices::PolicyIndex>,
}

/// The a map of UUIDs to typed indices
#[derive(Serialize, Deserialize, Default, Debug)]
pub(crate) struct NodeIdMap {
    assets: HashMap<uuid::Uuid, typed_indices::AssetIndex>,
    users: HashMap<uuid::Uuid, typed_indices::UserIndex>,
    groups: HashMap<uuid::Uuid, typed_indices::GroupIndex>,
    tags: HashMap<uuid::Uuid, typed_indices::TagIndex>,
    policies: HashMap<uuid::Uuid, typed_indices::PolicyIndex>,
}

impl Graph {
    /// Save a svg of the access graph to the specified filename
    pub fn visualize(&self, path: &str) -> Result<String> {
        let my_dot = dot::Dot::new(&self.graph);
        let g = graphviz::parse(&format!["{my_dot:?}"])
            .map_err(|s| anyhow!(s))
            .context("failed to parse")?;
        let draw = graphviz::exec(
            g,
            &mut PrinterContext::default(),
            vec![
                CommandArg::Format(Format::Svg),
                CommandArg::Output(path.to_owned()),
            ],
        )
        .context("failed to exec graphviz. do you need to install it?")?;
        Ok(draw)
    }

    /// Check whether a given NodeName exists in the graph, and, if so, return the NodeIndex.
    /// If it does not, returns None
    #[deprecated = "please transition to referencing nodes by their id rather than their name"]
    pub(crate) fn get_untyped_node_index(&self, node: &NodeName) -> Option<NodeIndex> {
        // I was hoping to do this with a trait object, but it turns out that
        // I couldn't easily return Option<&dyn ToNodeIndex> from the match -
        // apparently because of the Option (it worked fine without)
        match node {
            NodeName::User(_) => self
                .nodes
                .users
                .get(node)
                .map(|n| NodeIndex::from(n.to_owned())),
            NodeName::Group { .. } => self
                .nodes
                .groups
                .get(node)
                .map(|n| NodeIndex::from(n.to_owned())),
            NodeName::Asset { .. } => self
                .nodes
                .assets
                .get(node)
                .map(|n| NodeIndex::from(n.to_owned())),
            NodeName::Policy { .. } => self
                .nodes
                .policies
                .get(node)
                .map(|n| NodeIndex::from(n.to_owned())),
            NodeName::Tag(_) => self
                .nodes
                .tags
                .get(node)
                .map(|n| NodeIndex::from(n.to_owned())),
        }
    }

    #[deprecated = "please transition to referencing nodes by their id rather than their name"]
    /// Check whether a given node already exists in the graph, and, if so, return a typed index
    pub(crate) fn get_asset_node_index(&self, node: &NodeName) -> Option<AssetIndex> {
        match node {
            NodeName::Asset { .. } => self.nodes.assets.get(node).map(|i| i.to_owned()),
            _ => None,
        }
    }

    #[deprecated = "please transition to referencing nodes by their id rather than their name"]
    /// Check whether a given node already exists in the graph, and, if so, return a typed index
    pub(crate) fn get_user_node_index(&self, node: &NodeName) -> Option<UserIndex> {
        match node {
            NodeName::User(_) => self.nodes.users.get(node).map(|i| i.to_owned()),
            _ => None,
        }
    }

    #[deprecated = "please transition to referencing nodes by their id rather than their name"]
    /// Check whether a given node already exists in the graph, and, if so, return a typed index
    pub(crate) fn get_group_node_index(&self, node: &NodeName) -> Option<GroupIndex> {
        match node {
            NodeName::Group { .. } => self.nodes.groups.get(node).map(|i| i.to_owned()),
            _ => None,
        }
    }

    #[deprecated = "please transition to referencing nodes by their id rather than their name"]
    /// Check whether a given node already exists in the graph, and, if so, return a typed index
    pub(crate) fn get_tag_node_index(&self, node: &NodeName) -> Option<TagIndex> {
        match node {
            NodeName::Tag(_) => self.nodes.tags.get(node).map(|i| i.to_owned()),
            _ => None,
        }
    }

    #[deprecated = "please transition to referencing nodes by their id rather than their name"]
    /// Check whether a given node already exists in the graph, and, if so, return a typed index
    pub(crate) fn get_policy_node_index(&self, node: &NodeName) -> Option<PolicyIndex> {
        match node {
            NodeName::Policy { .. } => self.nodes.policies.get(node).map(|i| i.to_owned()),
            _ => None,
        }
    }

    /// Check whether a given node already exists in the graph, and, if so, return the NodeIndex.
    /// If if the NodeName does not exist in the graph, returns None
    pub(crate) fn get_untyped_node_index_from_id(&self, node: &Uuid) -> Option<NodeIndex> {
        // I was hoping to do this with a trait object, but it turns out that
        // I couldn't easily return Option<&dyn ToNodeIndex> from the match -
        // apparently because of the Option (it worked fine without)
        if let Some(idx) = self
            .node_ids
            .users
            .get(node)
            .map(|n| NodeIndex::from(n.to_owned()))
        {
            Some(idx)
        } else if let Some(idx) = self
            .node_ids
            .groups
            .get(node)
            .map(|n| NodeIndex::from(n.to_owned()))
        {
            Some(idx)
        } else if let Some(idx) = self
            .node_ids
            .assets
            .get(node)
            .map(|n| NodeIndex::from(n.to_owned()))
        {
            Some(idx)
        } else if let Some(idx) = self
            .node_ids
            .policies
            .get(node)
            .map(|n| NodeIndex::from(n.to_owned()))
        {
            Some(idx)
        } else { self
            .node_ids
            .tags
            .get(node)
            .map(|n| NodeIndex::from(n.to_owned())) }
    }

    /// Check whether a given node already exists in the graph, and, if so, return a typed index
    pub(crate) fn get_asset_node_index_from_id(&self, node: &Uuid) -> Option<AssetIndex> {
        self.node_ids.assets.get(node).map(|i| i.to_owned())
    }
    /// Check whether a given node already exists in the graph, and, if so, return a typed index
    pub(crate) fn get_user_node_index_from_id(&self, node: &Uuid) -> Option<UserIndex> {
        self.node_ids.users.get(node).map(|i| i.to_owned())
    }
    /// Check whether a given node already exists in the graph, and, if so, return a typed index
    pub(crate) fn get_group_node_index_from_id(&self, node: &Uuid) -> Option<GroupIndex> {
        self.node_ids.groups.get(node).map(|i| i.to_owned())
    }
    /// Check whether a given node already exists in the graph, and, if so, return a typed index
    pub(crate) fn get_tag_node_index_from_id(&self, node: &Uuid) -> Option<TagIndex> {
        self.node_ids.tags.get(node).map(|i| i.to_owned())
    }
    /// Check whether a given node already exists in the graph, and, if so, return a typed index
    pub(crate) fn get_policy_node_index_from_id(&self, node: &Uuid) -> Option<PolicyIndex> {
        self.node_ids.policies.get(node).map(|i| i.to_owned())
    }

    /// Adds a node to the graph and returns the index.
    pub(crate) fn add_node(&mut self, node: &JettyNode) -> Result<()> {
        let node_name = node.get_node_name();
        let node_id = node.id();
        // Check for duplicate
        if let Some(idx) = self.get_untyped_node_index(&node_name) {
            self.merge_nodes(idx, node)?;
        } else {
            let idx = self.graph.add_node(node.to_owned());
            match node {
                JettyNode::Group(_) => {
                    self.nodes.groups.insert(node_name, GroupIndex::new(idx));
                    self.node_ids.groups.insert(node_id, GroupIndex::new(idx));
                }
                JettyNode::User(_) => {
                    self.nodes.users.insert(node_name, UserIndex::new(idx));
                    self.node_ids.users.insert(node_id, UserIndex::new(idx));
                }
                JettyNode::Asset(_) => {
                    self.nodes.assets.insert(node_name, AssetIndex::new(idx));
                    self.node_ids.assets.insert(node_id, AssetIndex::new(idx));
                }
                JettyNode::Tag(_) => {
                    self.nodes.tags.insert(node_name, TagIndex::new(idx));
                    self.node_ids.tags.insert(node_id, TagIndex::new(idx));
                }
                JettyNode::Policy(_) => {
                    self.nodes.policies.insert(node_name, PolicyIndex::new(idx));
                    self.node_ids
                        .policies
                        .insert(node_id, PolicyIndex::new(idx));
                }
            };
        };

        Ok(())
    }

    /// Updates a node. Should return the updated node. Returns an
    /// error if the nodes are incompatible (would require overwriting values).
    /// To be compatible, metadata from each
    #[allow(dead_code)]
    pub(crate) fn merge_nodes(&mut self, idx: NodeIndex, new: &JettyNode) -> Result<JettyNode> {
        // Fetch node from graph
        let node = &mut self.graph[idx];

        *node = node
            .merge_nodes(new)
            .context(format!["merging: {node:?}, {new:?}"])?;
        Ok(node.to_owned())
    }

    /// Add edges from cache. Return false if to/from doesn't exist
    pub(crate) fn add_edge(&mut self, edge: super::JettyEdge) -> bool {
        let to = self.get_untyped_node_index(&edge.to).or_else(|| {
            warn![
                "Unable to find \"to\" node: {:?} for \"from\" {:?}",
                &edge.to, &edge.from
            ];
            None
        });

        let from = self.get_untyped_node_index(&edge.from).or_else(|| {
            warn![
                "Unable to find \"from\" node: {:?} for \"to\" {:?}",
                &edge.from, &edge.to
            ];
            None
        });

        if let (Some(to), Some(from)) = (to, from) {
            self.graph.add_edge(from, to, edge.edge_type);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::{anyhow, Context, Result};

    use crate::{
        access_graph::{test_util::new_graph, GroupAttributes},
        jetty::ConnectorNamespace,
    };

    use super::*;

    use std::collections::{HashMap, HashSet};

    /// Test merge_nodes
    #[test]
    fn group_node_same_name_no_conflict() -> Result<()> {
        let mut g = new_graph();

        let name = NodeName::Group {
            name: "Group 1".to_string(),
            origin: Default::default(),
        };
        let id = Uuid::new_v5(&Uuid::NAMESPACE_URL, name.to_string().as_bytes());

        let original_node = JettyNode::Group(GroupAttributes {
            name: name.to_owned(),
            id,
            metadata: HashMap::new(),
            connectors: HashSet::from([ConnectorNamespace("test1".to_string())]),
        });

        // new_node introduces a new connector value
        let new_node = JettyNode::Group(GroupAttributes {
            name: name.to_owned(),
            id,
            metadata: HashMap::new(),
            connectors: HashSet::from([ConnectorNamespace("test2".to_string())]),
        });

        // desired output
        let combined_node = JettyNode::Group(GroupAttributes {
            name,
            id,
            metadata: HashMap::new(),
            connectors: HashSet::from([
                ConnectorNamespace("test2".to_string()),
                ConnectorNamespace("test1".to_string()),
            ]),
        });

        g.add_node(&original_node)?;

        let idx = g
            .get_untyped_node_index(&original_node.get_node_name())
            .ok_or(anyhow!["Unable to find \"to\" node: {:?}", &original_node])?;

        let merged_node = g
            .merge_nodes(idx, &new_node)
            .context(anyhow!["merging nodes"])?;

        assert_eq!(combined_node, merged_node);

        Ok(())
    }

    #[test]
    fn group_node_name_conflict() -> Result<()> {
        let mut g = new_graph();

        let name = NodeName::Group {
            name: "Group 1".to_string(),
            origin: Default::default(),
        };
        let id = Uuid::new_v5(&Uuid::NAMESPACE_URL, name.to_string().as_bytes());

        let original_node = JettyNode::Group(GroupAttributes {
            name,
            id,
            metadata: HashMap::new(),
            connectors: HashSet::new(),
        });

        let name2 = NodeName::Group {
            name: "Group 2".to_string(),
            origin: Default::default(),
        };
        let id2 = Uuid::new_v5(&Uuid::NAMESPACE_URL, name2.to_string().as_bytes());
        // new_node introduces a connector value
        let new_node = JettyNode::Group(GroupAttributes {
            name: name2,
            id: id2,
            metadata: HashMap::new(),
            connectors: HashSet::new(),
        });

        g.add_node(&original_node)?;

        let idx = g
            .get_untyped_node_index(&original_node.get_node_name())
            .ok_or(anyhow!["Unable to find \"to\" node: {:?}", &original_node])?;

        let merged_node = g
            .merge_nodes(idx, &new_node)
            .context(anyhow!["merging nodes"]);

        assert!(merged_node.is_err());

        Ok(())
    }

    #[test]
    fn group_node_hashmap_conflict() -> Result<()> {
        let mut g = new_graph();

        let name = NodeName::Group {
            name: "Group 1".to_string(),
            origin: Default::default(),
        };
        let id = Uuid::new_v5(&Uuid::NAMESPACE_URL, name.to_string().as_bytes());

        let original_node = JettyNode::Group(GroupAttributes {
            name: name.to_owned(),
            id,
            metadata: HashMap::from([("test1".to_string(), "value2".to_string())]),
            connectors: HashSet::new(),
        });

        // new_node introduces a conflicting metadata value
        let new_node = JettyNode::Group(GroupAttributes {
            name,
            id,
            metadata: HashMap::from([("test1".to_string(), "other_value".to_string())]),
            connectors: HashSet::new(),
        });

        g.add_node(&original_node)?;

        let idx = g
            .get_untyped_node_index(&original_node.get_node_name())
            .ok_or(anyhow!["Unable to find \"to\" node: {:?}", &original_node])?;

        let merged_node = g
            .merge_nodes(idx, &new_node)
            .context(anyhow!["merging nodes"]);

        assert!(merged_node.is_err());

        Ok(())
    }

    #[test]
    fn group_node_hashmap_expand() -> Result<()> {
        let mut g = new_graph();

        let name = NodeName::Group {
            name: "Group 1".to_string(),
            origin: Default::default(),
        };
        let id = Uuid::new_v5(&Uuid::NAMESPACE_URL, name.to_string().as_bytes());

        let original_node = JettyNode::Group(GroupAttributes {
            name: name.to_owned(),
            id,
            metadata: HashMap::from([("test1".to_string(), "value2".to_string())]),
            connectors: HashSet::new(),
        });

        // new_node introduces a new metadata key
        let new_node = JettyNode::Group(GroupAttributes {
            name: name.to_owned(),
            id,
            metadata: HashMap::from([("test2".to_string(), "value 3".to_string())]),
            connectors: HashSet::new(),
        });

        // when merged, the result should be:
        let combined_node = JettyNode::Group(GroupAttributes {
            name,
            id,
            metadata: HashMap::from([
                ("test2".to_string(), "value 3".to_string()),
                ("test1".to_string(), "value2".to_string()),
            ]),
            connectors: HashSet::new(),
        });

        g.add_node(&original_node)?;

        let idx = g
            .get_untyped_node_index(&original_node.get_node_name())
            .ok_or(anyhow!["Unable to find \"to\" node: {:?}", &original_node])?;

        let merged_node = g
            .merge_nodes(idx, &new_node)
            .context(anyhow!["merging nodes"])?;

        assert_eq!(merged_node, combined_node);

        Ok(())
    }
}
