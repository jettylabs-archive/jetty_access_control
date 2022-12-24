//! Utilities to return only part of a graph
//!

use std::collections::{HashMap, HashSet};

use petgraph::stable_graph::NodeIndex;

use crate::access_graph::{graph::typed_indices::TagIndex, AccessGraph, EdgeType, JettyNode};

use super::NodePath;

#[derive(Debug)]
/// The assets that are tagged by a tag, including those directly tagged, those with inherited tags, and those untagged
pub struct TaggedAssets {
    /// assets directly tagged with the tag
    pub directly_tagged: HashMap<NodeIndex, HashSet<NodePath>>,
    /// assets that inherit the tag via hierarchy
    pub via_hierarchy: HashMap<NodeIndex, HashSet<NodePath>>,
    /// assets that inherit the tag via lineage
    pub via_lineage: HashMap<NodeIndex, HashSet<NodePath>>,
    /// assets that are explicitly untagged
    pub untagged: HashSet<NodeIndex>,
}

impl AccessGraph {
    /// Return accessible assets
    pub fn asset_paths_for_tag(&self, tag: TagIndex) -> TaggedAssets {
        let jetty_node = &self[tag];

        let tag_node = match jetty_node {
            JettyNode::Tag(t) => t,
            _ => panic!("not a tag node"),
        };

        // The poison nodes are all the assets that the tag is directly removed from
        let binding = self.get_matching_descendants(
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
            let hierarchy_inheritors = self.all_matching_simple_paths_to_descendants(
                tag,
                |e| matches!(e, EdgeType::AppliedTo) || matches!(e, EdgeType::ParentOf),
                |n| matches!(n, JettyNode::Asset(_)),
                |n| matches!(n, JettyNode::Asset(_)),
                Some(2),
                None,
            );
            remove_poisoned_paths_from_collection(hierarchy_inheritors, &poison_nodes)
        } else {
            Default::default()
        };

        let node_paths_lineage = if tag_node.pass_through_lineage {
            // get paths of tags applied through lineage
            let lineage_inheritors = self.all_matching_simple_paths_to_descendants(
                tag,
                |e| matches!(e, EdgeType::AppliedTo) || matches!(e, EdgeType::DerivedTo),
                |n| matches!(n, JettyNode::Asset(_)),
                |n| matches!(n, JettyNode::Asset(_)),
                Some(2),
                None,
            );
            remove_poisoned_paths_from_collection(lineage_inheritors, &poison_nodes)
        } else {
            Default::default()
        };

        // get paths for tags applied only directly
        let directly_assigned = self.all_matching_simple_paths_to_descendants(
            tag,
            |e| matches!(e, EdgeType::AppliedTo),
            |_| false,
            |n| matches!(n, JettyNode::Asset(_)),
            None,
            Some(1),
        );
        let node_paths_direct =
            remove_poisoned_paths_from_collection(directly_assigned, &poison_nodes);

        TaggedAssets {
            directly_tagged: node_paths_direct,
            via_hierarchy: node_paths_hierarchy,
            via_lineage: node_paths_lineage,
            untagged: poison_nodes.into_iter().map(|n| n.to_owned()).collect(),
        }
    }
}

fn remove_poisoned_paths_from_collection(
    all_paths: HashMap<NodeIndex, HashSet<super::NodePath>>,
    poison_nodes: &HashSet<NodeIndex>,
) -> HashMap<NodeIndex, HashSet<super::NodePath>> {
    all_paths
        .iter()
        .map(|(n, p)| {
            // only keep paths that have no overlap with the poison nodes
            (
                n,
                p.iter()
                    .filter(|NodePath(vn)| {
                        poison_nodes
                            .intersection(&HashSet::from_iter(vn.iter().copied()))
                            .next()
                            .is_none()
                    })
                    .collect::<HashSet<_>>(),
            )
        })
        // now only keep the assets that still have a path;
        .filter(|(_, p)| !p.is_empty())
        .map(|(n, p)| (n.to_owned(), p.into_iter().map(|z| z.to_owned()).collect()))
        .collect()
}

#[cfg(test)]
mod tests {

    use crate::{
        access_graph::{cual_to_asset_name_test, AssetAttributes, NodeName, TagAttributes},
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
                &JettyNode::Asset(AssetAttributes::new(
                    Cual::new("asset1://a/1"),
                    Default::default(),
                )),
                &JettyNode::Asset(AssetAttributes::new(
                    Cual::new("asset3://a/3"),
                    Default::default(),
                )),
                &JettyNode::Asset(AssetAttributes::new(
                    Cual::new("asset4://a/4"),
                    Default::default(),
                )),
                &JettyNode::Asset(AssetAttributes::new(
                    Cual::new("asset5://a/5"),
                    Default::default(),
                )),
                &JettyNode::Asset(AssetAttributes::new(
                    Cual::new("asset6://a/6"),
                    Default::default(),
                )),
                &JettyNode::Asset(AssetAttributes::new(
                    Cual::new("asset7://a/7"),
                    Default::default(),
                )),
                &JettyNode::Asset(AssetAttributes::new(
                    Cual::new("asset8://a/8"),
                    Default::default(),
                )),
            ],
            &[
                (
                    NodeName::Tag("tag1".to_owned()),
                    cual_to_asset_name_test(Cual::new("asset1://a/1"), Default::default()),
                    EdgeType::AppliedTo,
                ),
                (
                    NodeName::Tag("tag2".to_owned()),
                    cual_to_asset_name_test(Cual::new("asset1://a/1"), Default::default()),
                    EdgeType::AppliedTo,
                ),
                (
                    NodeName::Tag("tag3".to_owned()),
                    cual_to_asset_name_test(Cual::new("asset1://a/1"), Default::default()),
                    EdgeType::AppliedTo,
                ),
                (
                    NodeName::Tag("tag4".to_owned()),
                    cual_to_asset_name_test(Cual::new("asset1://a/1"), Default::default()),
                    EdgeType::AppliedTo,
                ),
                (
                    cual_to_asset_name_test(Cual::new("asset1://a/1"), Default::default()),
                    cual_to_asset_name_test(Cual::new("asset4://a/4"), Default::default()),
                    EdgeType::ParentOf,
                ),
                (
                    cual_to_asset_name_test(Cual::new("asset4://a/4"), Default::default()),
                    cual_to_asset_name_test(Cual::new("asset6://a/6"), Default::default()),
                    EdgeType::ParentOf,
                ),
                (
                    cual_to_asset_name_test(Cual::new("asset6://a/6"), Default::default()),
                    cual_to_asset_name_test(Cual::new("asset8://a/8"), Default::default()),
                    EdgeType::ParentOf,
                ),
                (
                    cual_to_asset_name_test(Cual::new("asset1://a/1"), Default::default()),
                    cual_to_asset_name_test(Cual::new("asset3://a/3"), Default::default()),
                    EdgeType::DerivedTo,
                ),
                (
                    cual_to_asset_name_test(Cual::new("asset3://a/3"), Default::default()),
                    cual_to_asset_name_test(Cual::new("asset5://a/5"), Default::default()),
                    EdgeType::DerivedTo,
                ),
                (
                    cual_to_asset_name_test(Cual::new("asset5://a/5"), Default::default()),
                    cual_to_asset_name_test(Cual::new("asset7://a/7"), Default::default()),
                    EdgeType::DerivedTo,
                ),
                (
                    NodeName::Tag("tag1".to_owned()),
                    cual_to_asset_name_test(Cual::new("asset6://a/6"), Default::default()),
                    EdgeType::RemovedFrom,
                ),
                (
                    NodeName::Tag("tag2".to_owned()),
                    cual_to_asset_name_test(Cual::new("asset6://a/6"), Default::default()),
                    EdgeType::RemovedFrom,
                ),
                (
                    NodeName::Tag("tag1".to_owned()),
                    cual_to_asset_name_test(Cual::new("asset5://a/5"), Default::default()),
                    EdgeType::RemovedFrom,
                ),
                (
                    NodeName::Tag("tag2".to_owned()),
                    cual_to_asset_name_test(Cual::new("asset5://a/5"), Default::default()),
                    EdgeType::RemovedFrom,
                ),
            ],
        )
    }

    #[test]
    fn no_inheritance_works() -> Result<()> {
        let ag = get_test_graph();
        let a = ag.asset_paths_for_tag(
            ag.get_tag_index_from_name(&NodeName::Tag("tag4".to_owned()))
                .unwrap(),
        );
        assert_eq!(a.directly_tagged.len(), 1);
        assert_eq!(a.via_hierarchy.len(), 0);
        assert_eq!(a.via_lineage.len(), 0);
        assert_eq!(a.untagged.len(), 0);
        Ok(())
    }

    #[test]
    fn lineage_inheritance_works() -> Result<()> {
        let ag = get_test_graph();
        let a = ag.asset_paths_for_tag(
            ag.get_tag_index_from_name(&NodeName::Tag("tag2".to_owned()))
                .unwrap(),
        );
        assert_eq!(a.directly_tagged.len(), 1);
        assert_eq!(a.via_hierarchy.len(), 0);
        assert_eq!(a.via_lineage.len(), 1);
        assert_eq!(a.untagged.len(), 2);
        Ok(())
    }

    #[test]
    fn hierarchy_inheritance_works() -> Result<()> {
        let ag = get_test_graph();
        let a = ag.asset_paths_for_tag(
            ag.get_tag_index_from_name(&NodeName::Tag("tag1".to_owned()))
                .unwrap(),
        );
        assert_eq!(a.directly_tagged.len(), 1);
        assert_eq!(a.via_hierarchy.len(), 1);
        assert_eq!(a.via_lineage.len(), 0);
        assert_eq!(a.untagged.len(), 2);
        Ok(())
    }

    #[test]
    fn both_inheritance_works() -> Result<()> {
        let ag = get_test_graph();
        let a = ag.asset_paths_for_tag(
            ag.get_tag_index_from_name(&NodeName::Tag("tag3".to_owned()))
                .unwrap(),
        );
        assert_eq!(a.directly_tagged.len(), 1);
        assert_eq!(a.via_hierarchy.len(), 3);
        assert_eq!(a.via_lineage.len(), 3);
        assert_eq!(a.untagged.len(), 0);
        Ok(())
    }
}
