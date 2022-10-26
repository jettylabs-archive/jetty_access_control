use std::collections::HashMap;

use anyhow::Context;
use petgraph::stable_graph::NodeIndex;

use crate::{
    access_graph::{
        graph::typed_indices::{AssetIndex, TagIndex, UserIndex},
        AccessGraph, JettyNode,
    },
    connectors::UserIdentifier,
};

impl AccessGraph {
    /// get a map of tags and corresponding assets that are accessible by a user
    pub fn get_user_accessible_tags<'a>(
        &'a self,
        user: UserIndex,
    ) -> HashMap<TagIndex, Vec<AssetIndex>> {
        // get all the user_accessable assets
        let accessable_assets = self.get_user_accessible_assets(user);
        let tag_asset_map = accessable_assets
            .iter()
            .map(|(c, _)| (c, self.tags_for_asset(*c)))
            .map(|(c, i)| i.iter().map(|n| (n.clone(), c)).collect::<Vec<_>>())
            .flatten()
            .fold(
                HashMap::<TagIndex, Vec<AssetIndex>>::new(),
                |mut acc, (tag_node, asset_index)| {
                    acc.entry(TagIndex::new(tag_node))
                        .and_modify(|e| {
                            e.push(asset_index.to_owned());
                        })
                        .or_insert(vec![asset_index.to_owned()]);
                    acc
                },
            );
        tag_asset_map
    }
}
