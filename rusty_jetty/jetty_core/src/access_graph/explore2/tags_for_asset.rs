//! Utilities to return only part of a graph
//!

use std::collections::{HashMap, HashSet};

use crate::access_graph::{AccessGraph, EdgeType, JettyNode, NodeName, TagAttributes};

use super::NodePath;

impl AccessGraph {
    /// Return accessible assets
    pub fn tags_for_asset(&self, asset: &NodeName) -> HashSet<JettyNode> {
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
        );

        // get direct tags that aren't applied through lineage or hierarchy:
        let single_asset_paths = self.get_paths_to_tags_via_inheritance(
            asset,
            |e| matches!(e, EdgeType::TaggedAs),
            |n| {
                matches!(
                    n,
                    JettyNode::Tag(TagAttributes {
                        pass_through_lineage: false,
                        pass_through_hierarchy: false,
                        ..
                    })
                )
            },
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
        );

        // for each poison path, get a map of the tag and a HashSet of the the assets that it has been removed from
        // if these poison nodes show up in any of the inheritance paths, that whole path is invalid
        let poison_nodes = poison_paths
            .iter()
            // Get the node the tag is removed from. The tag itself will be the last member of the path, so use the penultimate member
            .map(|(n, p)| {
                (
                    n,
                    p.iter()
                        .map(|NodePath(v)| &v[v.len() - 2])
                        .collect::<HashSet<_>>(),
                )
            })
            .collect::<HashMap<&JettyNode, HashSet<_>>>();

        let mut clean_paths = remove_poisoned_paths(hierarchy_paths, &poison_nodes);

        clean_paths.extend(remove_poisoned_paths(lineage_paths, &poison_nodes));
        clean_paths.extend(remove_poisoned_paths(single_asset_paths, &poison_nodes));

        clean_paths

        // Now get the hierarchy-based tags that don't have a poison tag in their path
    }

    fn get_paths_to_tags_via_inheritance(
        &self,
        from: &NodeName,
        edge_matcher: fn(&EdgeType) -> bool,
        target_matcher: fn(&JettyNode) -> bool,
    ) -> HashMap<JettyNode, Vec<super::NodePath>> {
        // go through inheritance to find all tags
        self.all_matching_simple_paths_to_children(
            from,
            edge_matcher,
            |n| matches!(n, JettyNode::Asset(_)),
            target_matcher,
            None,
            None,
        )
    }
}

fn remove_poisoned_paths<'a>(
    all_paths: HashMap<JettyNode, Vec<super::NodePath>>,
    poison_nodes: &HashMap<&JettyNode, HashSet<&JettyNode>>,
) -> HashSet<JettyNode> {
    all_paths
        .iter()
        .map(|(n, p)| {
            // only keep paths that have no overlap with the poison nodes
            (
                n,
                p.iter()
                    .filter(|NodePath(vn)| match poison_nodes.get(n) {
                        Some(z) => z
                            .intersection(&HashSet::from_iter(vn.iter()))
                            .next()
                            .is_none(),
                        None => true,
                    })
                    .collect::<Vec<_>>(),
            )
        })
        // now only keep the assets that still have a path;
        .filter(|(_n, p)| p.len() > 0)
        .map(|(n, _)| n.to_owned())
        .collect()
}

#[cfg(test)]
mod tests {

    use crate::access_graph::{AssetAttributes, GroupAttributes, UserAttributes};
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
                &JettyNode::Asset(AssetAttributes::new(Cual::new("asset1".to_owned()))),
                &JettyNode::Asset(AssetAttributes::new(Cual::new("asset3".to_owned()))),
                &JettyNode::Asset(AssetAttributes::new(Cual::new("asset4".to_owned()))),
                &JettyNode::Asset(AssetAttributes::new(Cual::new("asset5".to_owned()))),
                &JettyNode::Asset(AssetAttributes::new(Cual::new("asset6".to_owned()))),
                &JettyNode::Asset(AssetAttributes::new(Cual::new("asset7".to_owned()))),
                &JettyNode::Asset(AssetAttributes::new(Cual::new("asset8".to_owned()))),
            ],
            &[
                (
                    NodeName::Asset("asset1".to_owned()),
                    NodeName::Tag("tag1".to_owned()),
                    EdgeType::TaggedAs,
                ),
                (
                    NodeName::Asset("asset1".to_owned()),
                    NodeName::Tag("tag2".to_owned()),
                    EdgeType::TaggedAs,
                ),
                (
                    NodeName::Asset("asset1".to_owned()),
                    NodeName::Tag("tag3".to_owned()),
                    EdgeType::TaggedAs,
                ),
                (
                    NodeName::Asset("asset1".to_owned()),
                    NodeName::Tag("tag4".to_owned()),
                    EdgeType::TaggedAs,
                ),
                (
                    NodeName::Asset("asset4".to_owned()),
                    NodeName::Asset("asset1".to_owned()),
                    EdgeType::ChildOf,
                ),
                (
                    NodeName::Asset("asset6".to_owned()),
                    NodeName::Asset("asset4".to_owned()),
                    EdgeType::ChildOf,
                ),
                (
                    NodeName::Asset("asset8".to_owned()),
                    NodeName::Asset("asset6".to_owned()),
                    EdgeType::ChildOf,
                ),
                (
                    NodeName::Asset("asset3".to_owned()),
                    NodeName::Asset("asset1".to_owned()),
                    EdgeType::DerivedFrom,
                ),
                (
                    NodeName::Asset("asset5".to_owned()),
                    NodeName::Asset("asset3".to_owned()),
                    EdgeType::DerivedFrom,
                ),
                (
                    NodeName::Asset("asset7".to_owned()),
                    NodeName::Asset("asset5".to_owned()),
                    EdgeType::DerivedFrom,
                ),
                (
                    NodeName::Asset("asset6".to_owned()),
                    NodeName::Tag("tag1".to_owned()),
                    EdgeType::UntaggedAs,
                ),
                (
                    NodeName::Asset("asset6".to_owned()),
                    NodeName::Tag("tag2".to_owned()),
                    EdgeType::UntaggedAs,
                ),
                (
                    NodeName::Asset("asset5".to_owned()),
                    NodeName::Tag("tag1".to_owned()),
                    EdgeType::UntaggedAs,
                ),
                (
                    NodeName::Asset("asset5".to_owned()),
                    NodeName::Tag("tag2".to_owned()),
                    EdgeType::UntaggedAs,
                ),
            ],
        )
    }

    #[test]
    fn nodes_for_asset_lineage_works() -> Result<()> {
        let ag = get_test_graph();
        let a = ag.tags_for_asset(&NodeName::Asset("asset3".to_owned()));
        assert_eq!(a.len(), 2);
        Ok(())
    }

    #[test]
    fn nodes_for_asset_hierarchy_works() -> Result<()> {
        let ag = get_test_graph();
        let a = ag.tags_for_asset(&NodeName::Asset("asset4".to_owned()));
        assert_eq!(a.len(), 2);

        Ok(())
    }

    #[test]
    fn directly_applied_tags_works() -> Result<()> {
        let ag = get_test_graph();
        let a = ag.tags_for_asset(&NodeName::Asset("asset1".to_owned()));

        assert_eq!(a.len(), 4);

        Ok(())
    }

    #[test]
    fn tag_removal_works() -> Result<()> {
        let ag = get_test_graph();

        let a = ag.tags_for_asset(&NodeName::Asset("asset6".to_owned()));
        assert_eq!(a.len(), 1);

        let a = ag.tags_for_asset(&NodeName::Asset("asset8".to_owned()));
        assert_eq!(a.len(), 1);

        let a = ag.tags_for_asset(&NodeName::Asset("asset5".to_owned()));
        assert_eq!(a.len(), 1);

        let a = ag.tags_for_asset(&NodeName::Asset("asset7".to_owned()));
        assert_eq!(a.len(), 1);

        Ok(())
    }
}
