use std::collections::HashMap;

use anyhow::Context;
use petgraph::stable_graph::NodeIndex;

use crate::{
    access_graph::{AccessGraph, JettyNode, NodeName},
    connectors::UserIdentifier,
};

impl AccessGraph {
    /// get a map of tags and corresponding assets that are accessible by a user
    pub fn get_user_accessible_tags(
        &self,
        user: &UserIdentifier,
    ) -> HashMap<NodeIndex, Vec<JettyNode>> {
        // get all the user_accessable assets
        let accessable_assets = self.get_user_accessible_assets(user);
        let tag_asset_map = accessable_assets.keys().map(|c| (c, self.tags_for_asset(&NodeName::Asset(c.to_string()))))
            .flat_map(|(c, i)| i.iter().map(|n| (*n, c)).collect::<Vec<_>>())
            .fold(
                HashMap::<NodeIndex, Vec<JettyNode>>::new(),
                |mut acc, (tag_node, asset_cual)| {
                    acc.entry(tag_node)
                        .and_modify(|e| {
                            e.push(
                                self.get_node(&NodeName::Asset(asset_cual.to_string()))
                                    .context("nonexistent asset")
                                    .unwrap()
                                    .to_owned(),
                            );
                        })
                        .or_insert(vec![self
                            .get_node(&NodeName::Asset(asset_cual.to_string()))
                            .context("nonexistent asset")
                            .unwrap()
                            .to_owned()]);
                    acc
                },
            );
        tag_asset_map
    }
}
