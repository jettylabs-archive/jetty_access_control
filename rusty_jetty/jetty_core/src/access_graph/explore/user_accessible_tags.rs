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
        let tag_asset_map = accessable_assets
            .iter()
            .map(|(c, _)| {
                (
                    c,
                    self.tags_for_asset(
                        self.get_untyped_index_from_name(&NodeName::Asset(c.to_owned()))
                            .context("find index from name")
                            .unwrap(),
                    ),
                )
            })
            .map(|(c, i)| i.iter().map(|n| (n.clone(), c)).collect::<Vec<_>>())
            .flatten()
            .fold(
                HashMap::<NodeIndex, Vec<JettyNode>>::new(),
                |mut acc, (tag_node, asset_cual)| {
                    acc.entry(tag_node)
                        .and_modify(|e| {
                            e.push(
                                self.get_node(&NodeName::Asset(asset_cual.to_owned()))
                                    .context("nonexistent asset")
                                    .unwrap()
                                    .to_owned(),
                            );
                        })
                        .or_insert(vec![self
                            .get_node(&NodeName::Asset(asset_cual.to_owned()))
                            .context("nonexistent asset")
                            .unwrap()
                            .to_owned()]);
                    acc
                },
            );
        tag_asset_map
    }
}
