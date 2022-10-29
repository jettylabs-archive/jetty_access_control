use std::collections::HashMap;

use crate::access_graph::{
    graph::typed_indices::{AssetIndex, TagIndex, UserIndex},
    AccessGraph,
};

impl AccessGraph {
    /// get a map of tags and corresponding assets that are accessible by a user
    pub fn get_user_accessible_tags(&self, user: UserIndex) -> HashMap<TagIndex, Vec<AssetIndex>> {
        // get all the user_accessable assets
        let accessable_assets = self.get_user_accessible_assets(user);
        let tag_asset_map = accessable_assets
            .keys()
            .map(|c| (c, self.tags_for_asset(*c)))
            .flat_map(|(c, i)| i.iter().map(|n| (*n, c)).collect::<Vec<_>>())
            .fold(
                HashMap::<TagIndex, Vec<AssetIndex>>::new(),
                |mut acc, (tag_node, asset_index)| {
                    acc.entry(TagIndex::new(tag_node))
                        .and_modify(|e| {
                            e.push(asset_index.to_owned());
                        })
                        .or_insert_with(|| vec![asset_index.to_owned()]);
                    acc
                },
            );
        tag_asset_map
    }
}
