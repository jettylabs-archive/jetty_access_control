//! Parsing group configuration

use std::{
    collections::{HashMap, HashSet},
    fs,
};

use anyhow::{bail, Result};

use crate::{
    access_graph::NodeName, jetty::ConnectorNamespace, project, write::utils::error_vec_to_string,
    Jetty,
};

use super::GroupConfig;

/// read a non-validated config file from the default location. Return a GroupConfig struct
pub(crate) fn read_config_file() -> Result<GroupConfig> {
    let val = fs::read_to_string(project::groups_cfg_path_local())?;
    let res: Vec<GroupConfig> = yaml_peg::serde::from_str(&val)?;
    if res.is_empty() {
        bail!("unable to parse group configuration file")
    };
    Ok(res[0].to_owned())
}

fn validate_config(config: &GroupConfig, jetty: &Jetty) -> Vec<String> {
    let mut errors = Vec::new();
    let mut mapped_connectors = HashSet::new();

    let all_groups = get_all_group_names(config);

    for group in config {
        let (prefix, suffix) = split_group_name(&group.name);

        // if there's a prefix, make sure it's allowed
        if let Some(conn) = &prefix {
            if !jetty.has_connector(conn) {
                errors.push(format!("configuration specifies a group `{suffix}` with the prefix `{conn}` but there is no connector `{conn}` in the project"));
            }
        }

        // check the groups referenced in member_of to make sure they exist
        for g in &group.member_of {
            if !all_groups.contains(g) {
                errors.push(format!("configuration refers to group `{g}`, but there is no group with that name in the configuration"));
            }
            // if a group is connector-specific, only groups from Jetty or from that connector can be members. If it's a jetty group,
            // any group can be a member
            let (member_prefix, _) = split_group_name(g);
            if prefix.is_some() && member_prefix.is_some() && prefix != member_prefix {
                errors.push(format!(
                    "{}, a connector-specific group, cannot have group members from other connectors ({g})",
                    group.name
                ));
            }
        }

        for (conn, local_name) in &group.identifiers {
            // Check that the connectors exist
            if !jetty.has_connector(conn) {
                errors.push(format!("configuration refers to a connector called `{conn}`, but there is no connector with that name in the project"));
            }
            // make sure that there aren't any double assignments
            match mapped_connectors.insert((conn, local_name)) {
                true => (),
                false => {
                    errors.push(format!("the {conn}-specific group name, `{local_name}` was assigned to more than one group"));
                }
            };
        }
    }
    errors
}

fn split_group_name(name: &String) -> (Option<ConnectorNamespace>, String) {
    match name.split_once("::") {
        Some((prefix, suffix)) => (
            Some(ConnectorNamespace(prefix.to_owned())),
            suffix.to_owned(),
        ),
        None => (None, name.to_owned()),
    }
}

pub(crate) fn get_all_group_names(config: &GroupConfig) -> HashSet<String> {
    config.iter().map(|g| g.name.to_owned()).collect()
}

/// Parse and validate group configuration return a BTreeSet of configs
pub fn parse_and_validate_groups(jetty: &Jetty) -> Result<GroupConfig> {
    let config = read_config_file()?;
    let errors = validate_config(&config, jetty);
    if !errors.is_empty() {
        bail!(
            "configuration is invalid:\n{}",
            error_vec_to_string(&errors)
        );
    };
    Ok(config)
}

/// get the map of jetty group names to the connector-specific group names
pub fn get_group_to_nodename_map(
    validated_config: &GroupConfig,
    connectors: &HashSet<ConnectorNamespace>,
) -> HashMap<String, HashMap<ConnectorNamespace, NodeName>> {
    validated_config
        .iter()
        .map(|group| {
            (
                group.name.to_owned(),
                // branch on whether it's a connector-specific group
                if let (Some(connector), local_name) = split_group_name(&group.name) {
                    [(
                        connector.to_owned(),
                        NodeName::Group {
                            name: local_name,
                            origin: connector,
                        },
                    )]
                    .into()
                } else {
                    connectors
                        .iter()
                        .map(|connector| {
                            (connector.to_owned(), {
                                match group.identifiers.get(connector) {
                                    Some(name) => NodeName::Group {
                                        name: name.to_owned(),
                                        origin: connector.to_owned(),
                                    },
                                    None => NodeName::Group {
                                        name: group.name.to_owned(),
                                        origin: connector.to_owned(),
                                    },
                                }
                            })
                        })
                        .collect()
                },
            )
        })
        .collect()
}

/// Get the map of group -> member_of strings
pub(crate) fn get_group_membership_map(
    validated_config: &GroupConfig,
) -> HashMap<String, HashSet<String>> {
    validated_config
        .iter()
        .map(|g| (g.name.to_owned(), g.member_of.iter().cloned().collect()))
        .collect()
}
