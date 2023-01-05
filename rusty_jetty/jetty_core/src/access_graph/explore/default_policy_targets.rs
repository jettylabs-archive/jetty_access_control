//! Find the potential targets for a default policy path.

use std::collections::BTreeSet;

use anyhow::{anyhow, Result};
use petgraph::stable_graph::NodeIndex;

use crate::access_graph::{AccessGraph, EdgeType, JettyNode, NodeName};

impl AccessGraph {
    /// Return a set of all the targets of a default (wildcard) policy
    pub(crate) fn default_policy_targets(
        &self,
        default_policy: &NodeName,
    ) -> Result<BTreeSet<NodeIndex>> {
        // Make sure that the node name is right
        let (root_node, matching_path, target_type) = if let NodeName::DefaultPolicy {
            root_node,
            matching_path,
            target_type,
            ..
        } = default_policy
        {
            (root_node, matching_path, target_type)
        } else {
            panic!("node_name must be a NodeName::DefaultPolicy: {default_policy}");
        };

        let wildcard_details = wildcard_parser(matching_path);
        let root_id = self.get_node(root_node)?.id();
        let root_idx = self
            .get_asset_index_from_id(&root_id)
            .ok_or_else(|| anyhow!("root node must exist in the graph"))?;

        let res = self
            .get_matching_descendants(
                root_idx,
                |e| matches!(e, EdgeType::ParentOf),
                |n| matches!(n, JettyNode::Asset(_)),
                |n| match n {
                    JettyNode::Asset(a) => target_type == a.asset_type(),
                    _ => false,
                },
                Some(wildcard_details.depth),
                if wildcard_details.open_ended {
                    None
                } else {
                    Some(wildcard_details.depth)
                },
            )
            .into_iter()
            .collect();

        Ok(res)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
struct WildcardDetails {
    // The depth at which the wildcard terminates
    depth: usize,
    // Whether the match is open-ended (**). If it is open_ended, that means that every matching asset
    // at the given level and below would be a match. If it's not open_ended (*), it limits the match to
    // that single level
    open_ended: bool,
}
fn wildcard_parser(wildcard_string: &String) -> WildcardDetails {
    let wildcard_string = match wildcard_string.strip_prefix('/') {
        Some(s) => s.to_owned(),
        None => wildcard_string.to_owned(),
    };
    let wildcard_string = match wildcard_string.strip_suffix('/') {
        Some(s) => s.to_owned(),
        None => wildcard_string.to_owned(),
    };

    let parts: Vec<_> = wildcard_string.split('/').collect();
    WildcardDetails {
        depth: parts.len(),
        open_ended: parts[parts.len() - 1] == "**",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wildcard_parser() -> Result<()> {
        let test_cases = [
            ("**", 1, true),
            ("/*", 1, false),
            ("*/**", 2, true),
            ("*/*", 2, false),
            ("*/*/*/*/*", 5, false),
        ];

        for (path, depth, open_ended) in test_cases {
            assert_eq!(
                wildcard_parser(&path.into()),
                WildcardDetails { depth, open_ended }
            );
        }
        Ok(())
    }
}
