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

use crate::{access_graph::NodeName, jetty::ConnectorNamespace, Jetty};

use super::{get_config_paths, UserYaml};

/// read all config files into a Vec of user structs
fn read_config_files(paths: Paths) -> Result<HashMap<PathBuf, UserYaml>> {
    let mut res = HashMap::new();
    for path in paths {
        let path = path?;
        let yaml = fs::read_to_string(&path)?;
        let config_vec: Vec<UserYaml> = yaml_peg::serde::from_str(&yaml)?;
        if config_vec.is_empty() {
            bail!("unable to parse configuration")
        };
        let config = config_vec[0].to_owned();
        res.insert(path, config);
    }
    Ok(res)
}

/// Validate user configurations
fn validate_config(configs: &HashMap<PathBuf, UserYaml>, jetty: &Jetty) -> Result<Vec<String>> {
    let allowed_connectors: HashSet<ConnectorNamespace> =
        jetty.connectors.keys().cloned().collect();
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
            if !allowed_connectors.contains(&connector) {
                errors.push(format!(
                    "invalid connector in {}: {connector} is doesn't exist in your project configuration", path.display()
                ));
            }

            // make sure that the connector-specific names are only used once
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
        }
    }

    // only one use of each identifier
    Ok(errors)
}

/// Get a map of nodenames to local ids for each connector
fn get_nodename_local_id_map(
    configs: &HashMap<PathBuf, UserYaml>,
) -> HashMap<ConnectorNamespace, BiHashMap<NodeName, String>> {
    let mut res = HashMap::new();
    for (_path, config) in configs {
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
    todo!()
}

pub(crate) fn get_validated_nodename_local_id_map(
    jetty: &Jetty,
) -> Result<HashMap<ConnectorNamespace, BiHashMap<NodeName, String>>> {
    let paths = get_config_paths()?;
    let configs = read_config_files(paths)?;
    let errors = validate_config(&configs, jetty)?;
    if !errors.is_empty() {
        bail!(
            "invalid user configuration:\n{}",
            errors
                .into_iter()
                .map(|e| format!("{}", format!(" -{e}").as_str().red()))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    Ok(get_nodename_local_id_map(&configs))
}
