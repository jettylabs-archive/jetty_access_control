//! Graph stuff
//!

use anyhow::bail;
use anyhow::{anyhow, Context, Result};
use graphviz_rust as graphviz;
use graphviz_rust::cmd::CommandArg;
use graphviz_rust::cmd::Format;
use graphviz_rust::printer::PrinterContext;
use petgraph::stable_graph::NodeIndex;

use petgraph::{dot, stable_graph::StableDiGraph};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{EdgeType, JettyNode, NodeName};

/// The main graph wrapper
#[derive(Serialize, Deserialize)]
pub struct Graph {
    pub(crate) graph: StableDiGraph<JettyNode, EdgeType>,
    /// A map of node identifiers to indicies
    pub(crate) nodes: HashMap<NodeName, NodeIndex>,
}

impl Graph {
    /// Save a svg of the access graph to the specified filename
    pub fn visualize(&self, path: &str) -> Result<String> {
        let my_dot = dot::Dot::new(&self.graph);
        let g = graphviz::parse(&format!["{:?}", my_dot])
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

    /// Check whether a given node already exists in the graph
    #[inline(always)]
    pub fn get_node(&self, node: &NodeName) -> Option<&NodeIndex> {
        self.nodes.get(node)
    }

    /// Adds a node to the graph and returns the index.
    pub(crate) fn add_node(&mut self, node: &JettyNode) -> Result<()> {
        let node_name = node.get_name();
        // Check for duplicate
        if let Some(&idx) = self.get_node(&node_name) {
            self.merge_nodes(idx, node)?;
        } else {
            let idx = self.graph.add_node(node.to_owned());
            self.nodes.insert(node_name, idx);
        };

        Ok(())
    }

    #[allow(dead_code)]
    fn get_paths(
        &self,
        from_node_name: &NodeName,
        to_node_name: &NodeName,
    ) -> Result<impl Iterator<Item = Vec<NodeIndex>> + '_> {
        if let (Some(from), Some(to)) = (self.get_node(from_node_name), self.get_node(to_node_name))
        {
            Ok(petgraph::algo::all_simple_paths::<Vec<_>, _>(
                &self.graph,
                *from,
                *to,
                0,
                None,
            ))
        } else {
            bail!(
                "node names {:?} -> {:?} not found.",
                from_node_name,
                to_node_name
            )
        }
    }

    /// Get the neighbors for the node with the given name.
    ///
    /// Get all neighbors for a node, filtered by thos that yield true when
    /// `matcher` is applied to them.
    pub(crate) fn get_neighbors_for_node(
        &self,
        node_name: &NodeName,
        matcher: fn(&JettyNode) -> bool,
    ) -> Result<impl Iterator<Item = &JettyNode>> {
        let node = self
            .get_node(node_name)
            .ok_or_else(|| anyhow!("node not found"))?;
        Ok(self.graph.neighbors(*node).filter_map(move |target_node| {
            let target = &self.graph[target_node];
            if matcher(target) {
                Some(target)
            } else {
                None
            }
        }))
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
            .context(format!["merging: {:?}, {:?}", node, new])?;
        Ok(node.to_owned())
    }

    /// Add edges from cache. Return an error if to/from doesn't exist
    pub(crate) fn add_edge(&mut self, edge: super::JettyEdge) -> Result<()> {
        let to = self.get_node(&edge.to).ok_or_else(|| {
            anyhow![
                "Unable to find \"to\" node: {:?} for \"from\" {:?}",
                &edge.to,
                &edge.from
            ]
        })?;

        let from = self.get_node(&edge.from).ok_or_else(|| {
            anyhow![
                "Unable to find \"from\" node: {:?} for \"to\" {:?}",
                &edge.from,
                &edge.to
            ]
        })?;

        self.graph.add_edge(*from, *to, edge.edge_type);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::{anyhow, Context, Result};

    use crate::{
        access_graph::{test_util::new_graph, AssetAttributes, GroupAttributes, JettyEdge},
        connectors::AssetType,
        cual::Cual,
    };

    use super::*;

    use std::collections::{HashMap, HashSet};

    /// Test merge_nodes
    #[test]
    fn group_node_same_name_no_conflict() -> Result<()> {
        let mut g = new_graph();

        let original_node = JettyNode::Group(GroupAttributes {
            name: "Group 1".to_string(),
            metadata: HashMap::new(),
            connectors: HashSet::from(["test1".to_string()]),
        });

        // new_node introduces a new connector value
        let new_node = JettyNode::Group(GroupAttributes {
            name: "Group 1".to_string(),
            metadata: HashMap::new(),
            connectors: HashSet::from(["test2".to_string()]),
        });

        // desired output
        let combined_node = JettyNode::Group(GroupAttributes {
            name: "Group 1".to_string(),
            metadata: HashMap::new(),
            connectors: HashSet::from(["test2".to_string(), "test1".to_string()]),
        });

        g.add_node(&original_node)?;

        let &idx = g
            .get_node(&original_node.get_name())
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

        let original_node = JettyNode::Group(GroupAttributes {
            name: "Group 1".to_string(),
            metadata: HashMap::new(),
            connectors: HashSet::new(),
        });

        // new_node introduces a connector value
        let new_node = JettyNode::Group(GroupAttributes {
            name: "Group 2".to_string(),
            metadata: HashMap::new(),
            connectors: HashSet::new(),
        });

        g.add_node(&original_node)?;

        let &idx = g
            .get_node(&original_node.get_name())
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

        let original_node = JettyNode::Group(GroupAttributes {
            name: "Group 1".to_string(),
            metadata: HashMap::from([("test1".to_string(), "value2".to_string())]),
            connectors: HashSet::new(),
        });

        // new_node introduces a conflicting metadata value
        let new_node = JettyNode::Group(GroupAttributes {
            name: "Group 1".to_string(),
            metadata: HashMap::from([("test1".to_string(), "other_value".to_string())]),
            connectors: HashSet::new(),
        });

        g.add_node(&original_node)?;

        let &idx = g
            .get_node(&original_node.get_name())
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

        let original_node = JettyNode::Group(GroupAttributes {
            name: "Group 1".to_string(),
            metadata: HashMap::from([("test1".to_string(), "value2".to_string())]),
            connectors: HashSet::new(),
        });

        // new_node introduces a new metadata key
        let new_node = JettyNode::Group(GroupAttributes {
            name: "Group 1".to_string(),
            metadata: HashMap::from([("test2".to_string(), "value 3".to_string())]),
            connectors: HashSet::new(),
        });

        // when merged, the result should be:
        let combined_node = JettyNode::Group(GroupAttributes {
            name: "Group 1".to_string(),
            metadata: HashMap::from([
                ("test2".to_string(), "value 3".to_string()),
                ("test1".to_string(), "value2".to_string()),
            ]),
            connectors: HashSet::new(),
        });

        g.add_node(&original_node)?;

        let &idx = g
            .get_node(&original_node.get_name())
            .ok_or(anyhow!["Unable to find \"to\" node: {:?}", &original_node])?;

        let merged_node = g
            .merge_nodes(idx, &new_node)
            .context(anyhow!["merging nodes"])?;

        assert_eq!(merged_node, combined_node);

        Ok(())
    }

    #[test]
    fn get_paths_works() -> Result<()> {
        let mut g = new_graph();
        g.add_node(&JettyNode::Asset(AssetAttributes {
            cual: Cual::new("my_cual".to_owned()),
            asset_type: AssetType::default(),
            metadata: HashMap::new(),
            connectors: HashSet::new(),
        }))?;

        g.add_node(&JettyNode::Asset(AssetAttributes {
            cual: Cual::new("my_second_cual".to_owned()),
            asset_type: AssetType::default(),
            metadata: HashMap::new(),
            connectors: HashSet::new(),
        }))?;

        g.add_edge(JettyEdge {
            from: NodeName::Asset("my_cual".to_owned()),
            to: NodeName::Asset("my_second_cual".to_owned()),
            edge_type: EdgeType::ParentOf,
        })?;

        let paths = g.get_paths(
            &NodeName::Asset("my_cual".to_owned()),
            &NodeName::Asset("my_second_cual".to_owned()),
        )?;
        assert_eq!(
            paths.collect::<Vec<Vec<NodeIndex>>>(),
            vec![vec![NodeIndex::new(0), NodeIndex::new(1)]]
        );
        Ok(())
    }
}
