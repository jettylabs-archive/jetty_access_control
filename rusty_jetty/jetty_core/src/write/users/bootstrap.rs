//! Bootstrapping the user configuration

use std::{collections::HashMap, fs, path::PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    access_graph::{graph::typed_indices::UserIndex, AccessGraph, NodeName, UserAttributes},
    jetty::ConnectorNamespace,
    project,
    write::utils::clean_string_for_path,
    Jetty,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct UserYaml {
    name: String,
    identifiers: HashMap<ConnectorNamespace, String>,
    id: Uuid,
}

impl Jetty {
    /// Get all the users from the access graph and convert them into a map of path to file and yaml config
    pub fn generate_bootstrapped_user_yaml(&self) -> Result<HashMap<PathBuf, String>> {
        let ag = self.try_access_graph()?;
        let users = &ag.graph.nodes.users;

        let mut res = HashMap::new();

        for (name, &idx) in users {
            res.insert(
                format!("{}.yaml", clean_string_for_path(name.to_string())).into(),
                yaml_peg::serde::to_string(&user_yaml_from_idx(self, idx)?)?,
            );
        }

        Ok(res)
    }
}

/// Write the output of generate_bootstrapped_users_yaml to the proper directories
pub fn write_bootstrapped_user_yaml(users: HashMap<PathBuf, String>) -> Result<()> {
    let parent_path = project::users_cfg_root_path_local();

    // make sure the parent directories exist
    fs::create_dir_all(&parent_path)?;

    for (path, policy_doc) in users {
        fs::write(parent_path.join(path), policy_doc)?;
    }
    Ok(())
}

fn user_yaml_from_idx(jetty: &Jetty, idx: UserIndex) -> Result<UserYaml> {
    let ag = jetty.try_access_graph()?;
    let attributes: UserAttributes = ag[idx].to_owned().try_into()?;
    let mut identifiers: HashMap<ConnectorNamespace, String> = HashMap::new();
    for connector in jetty.connectors.keys() {
        if let Ok(val) = ag
            .translator()
            .try_translate_node_name_to_local(&attributes.name, connector)
        {
            identifiers.insert(connector.to_owned(), val.to_owned());
        }
    }

    Ok(UserYaml {
        name: attributes.name.to_string(),
        identifiers,
        id: attributes.id.to_owned(),
    })
}
