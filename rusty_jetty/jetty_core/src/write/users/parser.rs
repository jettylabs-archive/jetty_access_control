//! Parse user config files

use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
};

use anyhow::{bail, Result};
use bimap::BiHashMap;
use colored::Colorize;
use glob::Paths;

use crate::{
    access_graph::NodeName,
    jetty::ConnectorNamespace,
    write::groups::{parser::get_all_group_names, GroupConfig},
    Jetty,
};

use super::{get_config_paths, UserYaml};

/// read a single config file into a UserYaml object
pub(crate) fn read_config_file(path: &PathBuf) -> Result<UserYaml> {
    let yaml = fs::read_to_string(path)?;
    let config_vec: Vec<UserYaml> = yaml_peg::serde::from_str(&yaml)?;
    if config_vec.is_empty() {
        bail!("unable to parse configuration")
    };
    let config = config_vec[0].to_owned();
    Ok(config)
}

/// read all config files into a map of file paths and user structs
pub(crate) fn read_config_files(paths: Paths) -> Result<HashMap<PathBuf, UserYaml>> {
    let mut res = HashMap::new();
    for path in paths {
        let path = path?;
        let config = read_config_file(&path)?;
        res.insert(path, config);
    }
    Ok(res)
}

/// Validate user configurations
fn validate_config(
    configs: &HashMap<PathBuf, UserYaml>,
    validated_group_config: &GroupConfig,
    jetty: &Jetty,
) -> Result<Vec<String>> {
    let ag = jetty.try_access_graph()?;
    let allowed_connectors: HashSet<ConnectorNamespace> =
        jetty.connectors.keys().cloned().collect();
    let mut allowed_local_names: HashSet<_> =
        ag.translator().get_all_local_users().into_keys().collect();
    let allowed_group_names = get_all_group_names(validated_group_config);
    let mut errors = Vec::new();
    let mut jetty_name_map = HashMap::new();
    let mut local_id_map = HashMap::new();
    for (path, config) in configs {
        // only one config per jetty name
        if let Some(old_path) = jetty_name_map.insert(config.name.to_owned(), path.to_owned()) {
            errors.push(format!(
                "duplicate jetty name found: {} exists in {} and {}",
                config.name.to_owned(),
                old_path.display(),
                path.display()
            ));
        }

        for (connector, local_name) in &config.identifiers {
            // make sure connectors are legal
            if !allowed_connectors.contains(connector) {
                errors.push(format!(
                    "invalid connector in {}: {connector} is doesn't exist in your project configuration", path.display()
                ));
            }

            // make sure that the connector-specific names are only used once and that they exist
            if let Some(old_path) = local_id_map.insert(
                (connector.to_owned(), local_name.to_owned()),
                path.to_owned(),
            ) {
                errors.push(format!(
                    "duplicate configuration found for {connector} user {local_name}: exists in {} and {}",
                    old_path.display(),
                    path.display()
                ));
            }
            // make sure local names are valid (but only if it's not a duplicate)
            else if !allowed_local_names.remove(&(connector.to_owned(), local_name.to_owned())) {
                errors.push(format!(
                    "invalid identifier in {}: {connector} doesn't have a user identified as \"{local_name}\"", path.display()
                ));
            }
        }

        for group in &config.member_of {
            if !allowed_group_names.contains(group) {
                errors.push(format!(
                    "invalid group name in {}: group config doesn't specify a group called \"{group}\"", path.display()
                ));
            }
        }
    }

    // if there are an remaining allowed_local_names, we create errors - all users must be accounted for
    for (connector, local_name) in &allowed_local_names {
        errors.push(format!(
            "missing configuration for {connector} user {local_name}"
        ));
    }

    // only one use of each identifier
    Ok(errors)
}

/// Get a map of nodenames to local ids for each connector
pub(crate) fn get_nodename_local_id_map(
    configs: &HashMap<PathBuf, UserYaml>,
) -> HashMap<ConnectorNamespace, BiHashMap<NodeName, String>> {
    let mut res = HashMap::new();
    for config in configs.values() {
        for (connector, local_name) in &config.identifiers {
            res.entry(connector.to_owned())
                .and_modify(|m: &mut BiHashMap<NodeName, String>| {
                    m.insert(
                        NodeName::User(config.name.to_owned()),
                        local_name.to_owned(),
                    );
                })
                .or_insert({
                    let mut m = BiHashMap::new();
                    m.insert(
                        NodeName::User(config.name.to_owned()),
                        local_name.to_owned(),
                    );
                    m
                });
        }
    }
    res
}

/// Read and validate user config
pub fn get_validated_file_config_map(
    jetty: &Jetty,
    validated_group_config: &GroupConfig,
) -> Result<HashMap<PathBuf, UserYaml>> {
    let paths = get_config_paths()?;
    let configs = read_config_files(paths)?;
    let errors = validate_config(&configs, validated_group_config, jetty)?;
    if !errors.is_empty() {
        bail!(
            "invalid user configuration: ({})\n{}",
            if errors.len() == 1 {
                "1 error".to_owned()
            } else {
                format!("{} errors", errors.len())
            },
            errors
                .into_iter()
                .map(|e| format!("{}", format!(" - {e}").as_str().red()))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    Ok(configs)
}

/// Given a map of paths->UserYaml, return all the user names from the config
pub(crate) fn get_all_user_names(config: &HashMap<PathBuf, UserYaml>) -> HashSet<String> {
    config
        .iter()
        .map(|(_, user)| user.name.to_owned())
        .collect()
}

pub(crate) fn get_validated_nodename_local_id_map(
    jetty: &Jetty,
    validated_group_config: &GroupConfig,
) -> Result<HashMap<ConnectorNamespace, BiHashMap<NodeName, String>>> {
    let configs = get_validated_file_config_map(jetty, validated_group_config)?;
    Ok(get_nodename_local_id_map(&configs))
}
