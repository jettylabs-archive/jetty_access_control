//! Bootstrapping new group configurations

use std::{
    collections::{HashMap, HashSet},
    fs,
};

use anyhow::Result;

use crate::{
    access_graph::{graph::typed_indices::TypedIndex, NodeName},
    project, Jetty,
};

use super::{GroupConfig, GroupYaml};

/// Fetch the configuration of groups from the environment. This will always reflect connector-specific relationships
pub fn get_env_config(jetty: &Jetty) -> Result<GroupConfig> {
    let env_nodes = get_env_membership_nodes(jetty)?;

    Ok(env_nodes
        .into_iter()
        .map(|(group, members)| GroupYaml {
            name: group.to_string(),
            // when bootstrapping, identifiers are not necessary because we don't try to combine multiple groups into one
            identifiers: Default::default(),
            member_of: members.into_iter().map(|m| m.to_string()).collect(),
            description: None,
        })
        .collect())
}

/// Get a map of Groups and member_of for those groups
pub(crate) fn get_env_membership_nodes(
    jetty: &Jetty,
) -> Result<HashMap<NodeName, HashSet<NodeName>>> {
    let ag = jetty.try_access_graph()?;
    let all_groups = &ag.graph.nodes.groups;

    all_groups
        .iter()
        .map(
            |(node_name, idx)| -> Result<(NodeName, HashSet<NodeName>)> {
                Ok((
                    node_name.to_owned(),
                    idx.member_of(jetty)?
                        .into_iter()
                        .map(|g| g.name(jetty).unwrap())
                        .collect::<HashSet<_>>(),
                ))
            },
        )
        .collect()
}

/// Write the generated group config to a file
pub fn write_env_config(group_config: &GroupConfig) -> Result<()> {
    let doc = yaml_peg::serde::to_string(&group_config)?;

    fs::create_dir_all(project::groups_cfg_path_local().parent().unwrap()).unwrap(); // Create the parent dir, if needed
    fs::write(project::groups_cfg_path_local(), doc)?;
    Ok(())
}
