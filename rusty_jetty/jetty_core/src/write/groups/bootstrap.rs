//! Bootstrap from the generated graph into a yaml file

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{anyhow, bail, Result};
use serde::Serialize;

use crate::{
    access_graph::{AccessGraph, EdgeType, GroupAttributes, JettyEdge, JettyNode, NodeName},
    Jetty,
};

#[derive(Serialize, Debug)]
struct YamlGroup {
    names: Option<BTreeMap<String, String>>,
    members: YamlGroupMembers,
}

#[derive(Serialize, Debug)]
pub(crate) struct YamlGroupMembers {
    groups: Option<BTreeSet<String>>,
    users: Option<BTreeSet<String>>,
}

impl Jetty {
    fn build_bootstrapped_group_config(&self) -> Result<BTreeMap<String, YamlGroup>> {
        let mut res = BTreeMap::new();

        let ag = self.access_graph.as_ref().ok_or(anyhow!(
            "unable to bootstrap group configuration - no access graph exists"
        ))?;
        let ag_groups = &ag.graph.nodes.groups;
        for (name, idx) in ag_groups {
            if !matches!(name, NodeName::Group { .. }) {
                bail!("group index doesn't point to a group")
            }

            let members = &ag.get_matching_children(
                *idx,
                |e| matches!(e, EdgeType::Includes),
                |_| false,
                |n| matches!(n, JettyNode::Group(_)) || matches!(n, JettyNode::User(_)),
                None,
                Some(1),
            );

            let mut member_groups = BTreeSet::new();
            let mut member_users = BTreeSet::new();
            for member in members {
                match &ag[*member] {
                    JettyNode::Group(g) => {
                        member_groups.insert(g.name.to_string());
                    }
                    JettyNode::User(u) => {
                        member_users.insert(u.name.clone().to_string());
                    }
                    _ => bail!("improper child node returned when building graph config"),
                }
            }

            res.insert(
                name.to_string(),
                YamlGroup {
                    names: None,
                    members: YamlGroupMembers {
                        groups: if member_groups.is_empty() {
                            None
                        } else {
                            Some(member_groups)
                        },
                        users: if member_users.is_empty() {
                            None
                        } else {
                            Some(member_users)
                        },
                    },
                },
            );
        }

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_bootstrapped_group_config_works() -> Result<()> {
        let jetty = crate::write::groups::tests::get_jetty();
        let cfg = jetty.build_bootstrapped_group_config()?;
        dbg!(&cfg);
        let yaml_output = yaml_peg::serde::to_string(&cfg)?;
        println!("{}", &yaml_output);
        Ok(())
    }
}
