//! Utilities to return only part of a graph
//!

use std::collections::{HashMap, HashSet};

use petgraph::stable_graph::NodeIndex;

use crate::access_graph::{AccessGraph, EdgeType, JettyNode, NodeName};

use super::NodePath;

#[derive(Debug)]
/// The assets that are tagged by a tag, including those directly tagged, those with inherited tags, and those untagged
pub struct TaggedAssets {
    /// assets directly tagged with the tag
    pub directly_tagged: HashMap<NodeIndex, Vec<NodePath>>,
    /// assets that inherit the tag via hierarchy
    pub via_hierarchy: HashMap<NodeIndex, Vec<NodePath>>,
    /// assets that inherit the tag via lineage
    pub via_lineage: HashMap<NodeIndex, Vec<NodePath>>,
    /// assets that are explicitly untagged
    pub untagged: HashSet<NodeIndex>,
}

impl AccessGraph {
    /// Return accessible assets
    pub fn asset_paths_for_tag(&self, tag: &NodeName) -> TaggedAssets {
        let jetty_node = self.get_node(tag).unwrap();

        let tag_node = match jetty_node {
            JettyNode::Tag(t) => t,
            _ => panic!("not a tag node"),
        };

        // The poison nodes are all the assets that the tag is directly removed from
        let binding = self.get_matching_children(
            tag,
            |n| matches!(n, EdgeType::RemovedFrom),
            |_| false,
            |n| matches!(n, JettyNode::Asset(_)),
            None,
            Some(1),
        );
        let poison_nodes = HashSet::from_iter(binding.into_iter());

        let node_paths_hierarchy = if tag_node.pass_through_hierarchy {
            // get paths of tags applied through hierarchy
            let hierarchy_inheritors = self.all_matching_simple_paths_to_children(
                tag,
                |e| matches!(e, EdgeType::AppliedTo) || matches!(e, EdgeType::ParentOf),
                |n| matches!(n, JettyNode::Asset(_)),
                |n| matches!(n, JettyNode::Asset(_)),
                Some(2),
                None,
            );
            remove_poisoned_paths(hierarchy_inheritors, &poison_nodes)
        } else {
            Default::default()
        };

        let node_paths_lineage = if tag_node.pass_through_lineage {
            // get paths of tags applied through lineage
            let lineage_inheritors = self.all_matching_simple_paths_to_children(
                tag,
                |e| matches!(e, EdgeType::AppliedTo) || matches!(e, EdgeType::DerivedTo),
                |n| matches!(n, JettyNode::Asset(_)),
                |n| matches!(n, JettyNode::Asset(_)),
                Some(2),
                None,
            );
            remove_poisoned_paths(lineage_inheritors, &poison_nodes)
        } else {
            Default::default()
        };

        // get paths for tags applied only directly
        let directly_assigned = self.all_matching_simple_paths_to_children(
            tag,
            |e| matches!(e, EdgeType::AppliedTo),
            |_| false,
            |n| matches!(n, JettyNode::Asset(_)),
            None,
            Some(1),
        );
        let node_paths_direct = remove_poisoned_paths(directly_assigned, &poison_nodes);

        TaggedAssets {
            directly_tagged: node_paths_direct,
            via_hierarchy: node_paths_hierarchy,
            via_lineage: node_paths_lineage,
            untagged: poison_nodes.into_iter().map(|n| n.to_owned()).collect(),
        }
    }
}

fn remove_poisoned_paths<'a>(
    all_paths: HashMap<NodeIndex, Vec<super::NodePath>>,
    poison_nodes: &HashSet<NodeIndex>,
) -> HashMap<NodeIndex, Vec<super::NodePath>> {
    all_paths
        .iter()
        .map(|(n, p)| {
            // only keep paths that have no overlap with the poison nodes
            (
                n,
                p.iter()
                    .filter(|NodePath(vn)| {
                        poison_nodes
                            .intersection(&HashSet::from_iter(vn.iter().map(|i| *i)))
                            .next()
                            .is_none()
                    })
                    .collect::<Vec<_>>(),
            )
        })
        // now only keep the assets that still have a path;
        .filter(|(_, p)| !p.is_empty())
        .map(|(n, p)| {
            (
                n.to_owned(),
                p.into_iter().map(|z| z.to_owned()).collect::<Vec<_>>(),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {

    use crate::{
        access_graph::{AssetAttributes, TagAttributes},
        cual::Cual,
    };

    use anyhow::Result;

    use super::*;

    fn get_test_graph() -> AccessGraph {
        AccessGraph::new_dummy(
            &[
                &JettyNode::Tag(TagAttributes::new("tag1".to_owned(), true, false)),
                &JettyNode::Tag(TagAttributes::new("tag2".to_owned(), false, true)),
                &JettyNode::Tag(TagAttributes::new("tag3".to_owned(), true, true)),
                &JettyNode::Tag(TagAttributes::new("tag4".to_owned(), false, false)),
                &JettyNode::Asset(AssetAttributes::new(Cual::new("asset1://a"))),
                &JettyNode::Asset(AssetAttributes::new(Cual::new("asset3://a"))),
                &JettyNode::Asset(AssetAttributes::new(Cual::new("asset4://a"))),
                &JettyNode::Asset(AssetAttributes::new(Cual::new("asset5://a"))),
                &JettyNode::Asset(AssetAttributes::new(Cual::new("asset6://a"))),
                &JettyNode::Asset(AssetAttributes::new(Cual::new("asset7://a"))),
                &JettyNode::Asset(AssetAttributes::new(Cual::new("asset8://a"))),
            ],
            &[
                (
                    NodeName::Tag("tag1".to_owned()),
                    NodeName::Asset("asset1://a".to_owned()),
                    EdgeType::AppliedTo,
                ),
                (
                    NodeName::Tag("tag2".to_owned()),
                    NodeName::Asset("asset1://a".to_owned()),
                    EdgeType::AppliedTo,
                ),
                (
                    NodeName::Tag("tag3".to_owned()),
                    NodeName::Asset("asset1://a".to_owned()),
                    EdgeType::AppliedTo,
                ),
                (
                    NodeName::Tag("tag4".to_owned()),
                    NodeName::Asset("asset1://a".to_owned()),
                    EdgeType::AppliedTo,
                ),
                (
                    NodeName::Asset("asset1://a".to_owned()),
                    NodeName::Asset("asset4://a".to_owned()),
                    EdgeType::ParentOf,
                ),
                (
                    NodeName::Asset("asset4://a".to_owned()),
                    NodeName::Asset("asset6://a".to_owned()),
                    EdgeType::ParentOf,
                ),
                (
                    NodeName::Asset("asset6://a".to_owned()),
                    NodeName::Asset("asset8://a".to_owned()),
                    EdgeType::ParentOf,
                ),
                (
                    NodeName::Asset("asset1://a".to_owned()),
                    NodeName::Asset("asset3://a".to_owned()),
                    EdgeType::DerivedTo,
                ),
                (
                    NodeName::Asset("asset3://a".to_owned()),
                    NodeName::Asset("asset5://a".to_owned()),
                    EdgeType::DerivedTo,
                ),
                (
                    NodeName::Asset("asset5://a".to_owned()),
                    NodeName::Asset("asset7://a".to_owned()),
                    EdgeType::DerivedTo,
                ),
                (
                    NodeName::Tag("tag1".to_owned()),
                    NodeName::Asset("asset6://a".to_owned()),
                    EdgeType::RemovedFrom,
                ),
                (
                    NodeName::Tag("tag2".to_owned()),
                    NodeName::Asset("asset6://a".to_owned()),
                    EdgeType::RemovedFrom,
                ),
                (
                    NodeName::Tag("tag1".to_owned()),
                    NodeName::Asset("asset5://a".to_owned()),
                    EdgeType::RemovedFrom,
                ),
                (
                    NodeName::Tag("tag2".to_owned()),
                    NodeName::Asset("asset5://a".to_owned()),
                    EdgeType::RemovedFrom,
                ),
            ],
        )
    }

    #[test]
    fn no_inheritance_works() -> Result<()> {
        let ag = get_test_graph();
        let a = ag.asset_paths_for_tag(&NodeName::Tag("tag4".to_owned()));
        assert_eq!(a.directly_tagged.len(), 1);
        assert_eq!(a.via_hierarchy.len(), 0);
        assert_eq!(a.via_lineage.len(), 0);
        assert_eq!(a.untagged.len(), 0);
        Ok(())
    }

    #[test]
    fn lineage_inheritance_works() -> Result<()> {
        let ag = get_test_graph();
        let a = ag.asset_paths_for_tag(&NodeName::Tag("tag2".to_owned()));
        assert_eq!(a.directly_tagged.len(), 1);
        assert_eq!(a.via_hierarchy.len(), 0);
        assert_eq!(a.via_lineage.len(), 1);
        assert_eq!(a.untagged.len(), 2);
        Ok(())
    }

    #[test]
    fn hierarchy_inheritance_works() -> Result<()> {
        let ag = get_test_graph();
        let a = ag.asset_paths_for_tag(&NodeName::Tag("tag1".to_owned()));
        assert_eq!(a.directly_tagged.len(), 1);
        assert_eq!(a.via_hierarchy.len(), 1);
        assert_eq!(a.via_lineage.len(), 0);
        assert_eq!(a.untagged.len(), 2);
        Ok(())
    }

    #[test]
    fn both_inheritance_works() -> Result<()> {
        let ag = get_test_graph();
        let a = ag.asset_paths_for_tag(&NodeName::Tag("tag3".to_owned()));
        assert_eq!(a.directly_tagged.len(), 1);
        assert_eq!(a.via_hierarchy.len(), 3);
        assert_eq!(a.via_lineage.len(), 3);
        assert_eq!(a.untagged.len(), 0);
        Ok(())
    }
}
