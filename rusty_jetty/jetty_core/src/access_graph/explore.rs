//! Utilities for exploration of the graph.
//!

use anyhow::{bail, Result};

use super::{graph::Graph, JettyNode, NodeName};

#[inline(always)]
fn policy_matcher(p: &JettyNode) -> bool {
    matches!(p, JettyNode::Policy(_))
}

#[inline(always)]
fn user_matcher(u: &JettyNode) -> bool {
    matches!(u, JettyNode::User(_))
}

#[inline(always)]
fn asset_matcher(a: &JettyNode) -> bool {
    matches!(a, JettyNode::Asset(_))
}

impl Graph {
    fn get_assets_user_accesses(
        &self,
        user: &NodeName,
    ) -> Result<impl Iterator<Item = &JettyNode>> {
        if !matches!(user, NodeName::User(_)) {
            bail!("not a user");
        }

        // 1. traverse graph from user to their policies.
        Ok(self
            .get_neighbors_for_node(user, policy_matcher)?
            .map(|policy| {
                // 2. traverse graph from policies to their governed assets.
                self.get_neighbors_for_node(&policy.get_name(), asset_matcher)
                    .unwrap()
            })
            .flatten())
        // TODO: recursively get child assets as necessary here.
        // 3? ask connector for effective permissions
    }

    fn get_users_with_access_to(
        &self,
        asset: &NodeName,
    ) -> Result<impl Iterator<Item = &JettyNode>> {
        if !matches!(asset, NodeName::Asset(_)) {
            bail!("not a asset");
        }

        println!(
            "neighbors for {:?}: {:?}",
            asset,
            self.get_neighbors_for_node(asset, policy_matcher)?
                .collect::<Vec<_>>()
        );
        // 1. traverse graph from the asset to their policies.
        Ok(self
            .get_neighbors_for_node(asset, policy_matcher)?
            .map(|policy| {
                // 2. traverse graph from policies to their users.
                let r = self
                    .get_neighbors_for_node(&policy.get_name(), user_matcher)
                    .unwrap();
                r
            })
            .flatten())
        // 3? ask connector for effective permissions.
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
    fn get_assets_user_accesses_works() -> Result<()> {
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

        let test_asset = JettyNode::Asset(AssetAttributes {
            cual: Cual::new("my_cual".to_owned()),
            asset_type: AssetType::default(),
            metadata: HashMap::new(),
            connectors: HashSet::new(),
        });

        g.add_node(&test_asset)?;

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
        assert_eq!(a.collect::<Vec<_>>(), vec![&test_asset]);
        Ok(())
    }

    #[test]
    fn get_users_with_access_to_works() -> Result<()> {
        let mut g = new_graph();
        let test_user = JettyNode::User(UserAttributes {
            name: "user".to_owned(),
            identifiers: HashMap::new(),
            other_identifiers: HashSet::new(),
            metadata: HashMap::new(),
            connectors: HashSet::new(),
        });

        g.add_node(&test_user)?;

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

        // For this test we need the back edges so we can get back to users
        g.add_edge(JettyEdge {
            from: NodeName::Policy("policy".to_owned()),
            to: NodeName::User("user".to_owned()),
            edge_type: EdgeType::GrantedBy,
        })?;

        g.add_edge(JettyEdge {
            from: NodeName::Asset("my_cual".to_owned()),
            to: NodeName::Policy("policy".to_owned()),
            edge_type: EdgeType::GovernedBy,
        })?;

        let a = g.get_users_with_access_to(&NodeName::Asset("my_cual".to_owned()))?;
        assert_eq!(a.collect::<Vec<_>>(), vec![&test_user]);
        Ok(())
    }
}
