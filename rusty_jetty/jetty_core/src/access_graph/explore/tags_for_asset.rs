//! Utilities to return only part of a graph
//!

use std::collections::{HashMap, HashSet};

use petgraph::stable_graph::NodeIndex;
use serde::Serialize;

use crate::access_graph::{
    graph::typed_indices::AssetIndex, AccessGraph, EdgeType, JettyNode, TagAttributes,
};

use super::NodePath;

#[derive(Serialize, Debug)]
/// the tags that are applied to an asset
pub struct AssetTags {
    /// Tags applied directly
    pub direct: HashSet<NodeIndex>,
    /// Tags inherited via lineage
    pub via_lineage: HashSet<NodeIndex>,
    /// Tags inherited via hierarchy
    pub via_hierarchy: HashSet<NodeIndex>,
}

impl AccessGraph {
    /// Return tags for an asset, grouped by the tag source.
    pub fn tags_for_asset_by_source<T: Into<NodeIndex> + Copy>(&self, asset: T) -> AssetTags {
        // get paths of tags applied through hierarchy
        let hierarchy_paths = self.get_paths_to_tags_via_inheritance(
            asset,
            |e| matches!(e, EdgeType::ChildOf) || matches!(e, EdgeType::TaggedAs),
            |n| {
                matches!(
                    n,
                    JettyNode::Tag(TagAttributes {
                        pass_through_hierarchy: true,
                        ..
                    })
                )
            },
            // starting at depth of two to exclude directly tagged assets (handled later)
            2,
        );

        // get paths of tags applied through lineage
        let lineage_paths = self.get_paths_to_tags_via_inheritance(
            asset,
            |e| matches!(e, EdgeType::DerivedFrom) || matches!(e, EdgeType::TaggedAs),
            |n| {
                matches!(
                    n,
                    JettyNode::Tag(TagAttributes {
                        pass_through_lineage: true,
                        ..
                    })
                )
            },
            // starting at depth of two to exclude directly tagged assets (handled later)
            2,
        );

        // get direct tags that aren't applied through lineage or hierarchy:
        let direct_paths = self.get_paths_to_tags_via_inheritance(
            asset,
            // By only allowing TaggedAs nodes, we can only traverse from assets to tags.
            |e| matches!(e, EdgeType::TaggedAs),
            |n| matches!(n, JettyNode::Tag(_)),
            1,
        );

        // get the paths to nodes that have had the tag explicitly removed
        let poison_paths = self.get_paths_to_tags_via_inheritance(
            asset,
            |e| {
                matches!(e, EdgeType::ChildOf)
                    || matches!(e, EdgeType::DerivedFrom)
                    || matches!(e, EdgeType::UntaggedAs)
            },
            |n| matches!(n, JettyNode::Tag(_)),
            1,
        );

        // for each poison path, get a map of the tag and a HashSet of the the assets that it has been removed from
        // if these poison nodes show up in any of the inheritance paths, that whole path is invalid
        let poison_nodes = poison_paths
            .iter()
            // Get the node the tag is removed from. The tag itself will be the last member of the path, so use the penultimate member
            .map(|(n, p)| {
                (
                    *n,
                    p.iter()
                        .map(|NodePath(v)| v[v.len() - 2])
                        .collect::<HashSet<_>>(),
                )
            })
            .collect::<HashMap<NodeIndex, HashSet<_>>>();

        AssetTags {
            direct: remove_poisoned_paths(direct_paths, &poison_nodes),
            via_lineage: remove_poisoned_paths(lineage_paths, &poison_nodes),
            via_hierarchy: remove_poisoned_paths(hierarchy_paths, &poison_nodes),
        }
    }

    /// get all tags applied to an asset
    pub fn tags_for_asset(&self, asset: AssetIndex) -> HashSet<NodeIndex> {
        let asset_tags = self.tags_for_asset_by_source(asset);
        let mut return_tags = asset_tags.direct;
        return_tags.extend(asset_tags.via_lineage);
        return_tags.extend(asset_tags.via_hierarchy);

        return_tags
    }

    fn get_paths_to_tags_via_inheritance<T: Into<NodeIndex>>(
        &self,
        from: T,
        edge_matcher: fn(&EdgeType) -> bool,
        target_matcher: fn(&JettyNode) -> bool,
        min_depth: usize,
    ) -> HashMap<NodeIndex, HashSet<super::NodePath>> {
        // go through inheritance to find all tags
        self.all_matching_simple_paths_to_descendants(
            from,
            edge_matcher,
            |n| matches!(n, JettyNode::Asset(_)),
            target_matcher,
            Some(min_depth),
            None,
        )
    }
}

fn remove_poisoned_paths(
    all_paths: HashMap<NodeIndex, HashSet<super::NodePath>>,
    poison_nodes: &HashMap<NodeIndex, HashSet<NodeIndex>>,
) -> HashSet<NodeIndex> {
    all_paths
        .iter()
        .map(|(n, p)| {
            // only keep paths that have no overlap with the poison nodes
            (
                n,
                p.iter()
                    .filter(|NodePath(vn)| match poison_nodes.get(n) {
                        Some(z) => z
                            .intersection(&HashSet::from_iter(vn.iter().copied()))
                            .next()
                            .is_none(),
                        None => true,
                    })
                    .collect::<HashSet<_>>(),
            )
        })
        // now only keep the assets that still have a path;
        .filter(|(_n, p)| !p.is_empty())
        .map(|(n, _)| n.to_owned())
        .collect()
}

#[cfg(test)]
mod tests {

    use crate::access_graph::{cual_to_asset_name_test, AssetAttributes, NodeName};
    use crate::cual::Cual;

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
                    cual_to_asset_name_test(Cual::new("asset1://a/1"), Default::default()),
                    NodeName::Tag("tag1".to_owned()),
                    EdgeType::TaggedAs,
                ),
                (
                    cual_to_asset_name_test(Cual::new("asset1://a/1"), Default::default()),
                    NodeName::Tag("tag2".to_owned()),
                    EdgeType::TaggedAs,
                ),
                (
                    cual_to_asset_name_test(Cual::new("asset1://a/1"), Default::default()),
                    NodeName::Tag("tag3".to_owned()),
                    EdgeType::TaggedAs,
                ),
                (
                    cual_to_asset_name_test(Cual::new("asset1://a/1"), Default::default()),
                    NodeName::Tag("tag4".to_owned()),
                    EdgeType::TaggedAs,
                ),
                (
                    cual_to_asset_name_test(Cual::new("asset4://a/4"), Default::default()),
                    cual_to_asset_name_test(Cual::new("asset1://a/1"), Default::default()),
                    EdgeType::ChildOf,
                ),
                (
                    cual_to_asset_name_test(Cual::new("asset6://a/6"), Default::default()),
                    cual_to_asset_name_test(Cual::new("asset4://a/4"), Default::default()),
                    EdgeType::ChildOf,
                ),
                (
                    cual_to_asset_name_test(Cual::new("asset8://a/8"), Default::default()),
                    cual_to_asset_name_test(Cual::new("asset6://a/6"), Default::default()),
                    EdgeType::ChildOf,
                ),
                (
                    cual_to_asset_name_test(Cual::new("asset3://a/3"), Default::default()),
                    cual_to_asset_name_test(Cual::new("asset1://a/1"), Default::default()),
                    EdgeType::DerivedFrom,
                ),
                (
                    cual_to_asset_name_test(Cual::new("asset5://a/5"), Default::default()),
                    cual_to_asset_name_test(Cual::new("asset3://a/3"), Default::default()),
                    EdgeType::DerivedFrom,
                ),
                (
                    cual_to_asset_name_test(Cual::new("asset7://a/7"), Default::default()),
                    cual_to_asset_name_test(Cual::new("asset5://a/5"), Default::default()),
                    EdgeType::DerivedFrom,
                ),
                (
                    cual_to_asset_name_test(Cual::new("asset6://a/6"), Default::default()),
                    NodeName::Tag("tag1".to_owned()),
                    EdgeType::UntaggedAs,
                ),
                (
                    cual_to_asset_name_test(Cual::new("asset6://a/6"), Default::default()),
                    NodeName::Tag("tag2".to_owned()),
                    EdgeType::UntaggedAs,
                ),
                (
                    cual_to_asset_name_test(Cual::new("asset5://a/5"), Default::default()),
                    NodeName::Tag("tag1".to_owned()),
                    EdgeType::UntaggedAs,
                ),
                (
                    cual_to_asset_name_test(Cual::new("asset5://a/5"), Default::default()),
                    NodeName::Tag("tag2".to_owned()),
                    EdgeType::UntaggedAs,
                ),
            ],
        )
    }

    #[test]
    fn nodes_for_asset_lineage_works() -> Result<()> {
        let ag = get_test_graph();
        let a = ag.tags_for_asset(
            ag.get_asset_index_from_name(&cual_to_asset_name_test(
                Cual::new("asset3://a/3"),
                Default::default(),
            ))
            .unwrap(),
        );
        assert_eq!(a.len(), 2);
        Ok(())
    }

    #[test]
    fn nodes_for_asset_hierarchy_works() -> Result<()> {
        let ag = get_test_graph();
        let a = ag.tags_for_asset(
            ag.get_asset_index_from_name(&cual_to_asset_name_test(
                Cual::new("asset4://a/4"),
                Default::default(),
            ))
            .unwrap(),
        );
        assert_eq!(a.len(), 2);

        Ok(())
    }

    #[test]
    fn directly_applied_tags_works() -> Result<()> {
        let ag = get_test_graph();
        let a = ag.tags_for_asset(
            ag.get_asset_index_from_name(&cual_to_asset_name_test(
                Cual::new("asset1://a/1"),
                Default::default(),
            ))
            .unwrap(),
        );

        assert_eq!(a.len(), 4);

        Ok(())
    }

    #[test]
    fn tag_removal_works() -> Result<()> {
        let ag = get_test_graph();

        let a = ag.tags_for_asset(
            ag.get_asset_index_from_name(&cual_to_asset_name_test(
                Cual::new("asset6://a/6"),
                Default::default(),
            ))
            .unwrap(),
        );
        assert_eq!(a.len(), 1);

        let a = ag.tags_for_asset(
            ag.get_asset_index_from_name(&cual_to_asset_name_test(
                Cual::new("asset8://a/8"),
                Default::default(),
            ))
            .unwrap(),
        );
        assert_eq!(a.len(), 1);

        let a = ag.tags_for_asset(
            ag.get_asset_index_from_name(&cual_to_asset_name_test(
                Cual::new("asset5://a/5"),
                Default::default(),
            ))
            .unwrap(),
        );
        assert_eq!(a.len(), 1);

        let a = ag.tags_for_asset(
            ag.get_asset_index_from_name(&cual_to_asset_name_test(
                Cual::new("asset7://a/7"),
                Default::default(),
            ))
            .unwrap(),
        );
        assert_eq!(a.len(), 1);

        Ok(())
    }
}
