//! Utilities for exploration of the graph.
//!

use anyhow::{bail, Result};

use super::{graph::Graph, JettyNode, NodeName};

impl Graph {
    fn get_assets_user_accesses(
        &self,
        user: &NodeName,
    ) -> Result<impl Iterator<Item = &JettyNode>> {
        match user {
            NodeName::User(_) => (),
            _ => bail!("not a user"),
        };
        // 1. traverse graph from user to their policies.
        Ok(self
            .get_neighbors_for_node(user, |p| matches!(p, JettyNode::Policy(_)))?
            .map(|policy| {
                // 2. traverse graph from policies to their governed assets.
                self.get_neighbors_for_node(&policy.get_name(), |a| {
                    matches!(a, JettyNode::Asset(_))
                })
                .unwrap()
            })
            .flatten())
        // TODO: recursively get child assets here
        // 3? ask connector for effective permissions
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use crate::{
        access_graph::{
            test_util::new_graph, AssetAttributes, EdgeType, JettyEdge, PolicyAttributes,
            UserAttributes,
        },
        connectors::AssetType,
        cual::Cual,
    };

    use super::*;

    #[test]
    fn get_neighbors_for_node_works() -> Result<()> {
        let mut g = new_graph();
        g.add_node(&JettyNode::User(UserAttributes {
            name: "user".to_owned(),
            identifiers: HashMap::new(),
            other_identifiers: HashSet::new(),
            metadata: HashMap::new(),
            connectors: HashSet::new(),
        }))?;

        g.add_node(&JettyNode::Policy(PolicyAttributes {
            connectors: HashSet::new(),
            name: "policy".to_owned(),
            privileges: HashSet::new(),
            pass_through_hierarchy: false,
            pass_through_lineage: false,
        }))?;

        g.add_node(&JettyNode::Asset(AssetAttributes {
            cual: Cual::new("my_cual".to_owned()),
            asset_type: AssetType::default(),
            metadata: HashMap::new(),
            connectors: HashSet::new(),
        }))?;

        g.add_edge(JettyEdge {
            from: NodeName::User("user".to_owned()),
            to: NodeName::Policy("policy".to_owned()),
            edge_type: EdgeType::GrantedBy,
        })?;

        g.add_edge(JettyEdge {
            from: NodeName::Policy("policy".to_owned()),
            to: NodeName::Asset("my_cual".to_owned()),
            edge_type: EdgeType::Governs,
        })?;

        let a = g.get_assets_user_accesses(&NodeName::User("user".to_owned()))?;
        assert_eq!(
            a.collect::<Vec<_>>(),
            vec![&JettyNode::Asset(AssetAttributes {
                cual: Cual::new("my_cual".to_owned()),
                asset_type: AssetType::Other,
                metadata: HashMap::new(),
                connectors: HashSet::new(),
            })]
        );
        Ok(())
    }
}
