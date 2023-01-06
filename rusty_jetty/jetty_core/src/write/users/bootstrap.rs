//! Bootstrapping the user configuration

use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
};

use anyhow::{anyhow, Context, Result};

use crate::{
    access_graph::{
        graph::typed_indices::{TypedIndex, UserIndex},
        NodeName,
    },
    jetty::ConnectorNamespace,
    project,
    write::{groups::parse_and_validate_groups, utils::clean_string_for_path},
    Jetty,
};

use super::{parser::get_validated_file_config_map, UserYaml};

impl Jetty {
    /// Get all the users from the access graph and convert them into a map of path to file and yaml config
    pub fn generate_bootstrapped_user_yaml(&self) -> Result<HashMap<PathBuf, UserYaml>> {
        let ag = self.try_access_graph()?;
        let users = &ag.graph.nodes.users;

        let mut res = HashMap::new();

        for (name, &idx) in users {
            res.insert(
                get_filename_from_node_name(name),
                user_yaml_from_idx(self, idx)?,
            );
        }

        Ok(res)
    }
}

/// Write the output of generate_bootstrapped_users_yaml to the proper directories
pub fn write_bootstrapped_user_yaml(users: HashMap<PathBuf, UserYaml>) -> Result<()> {
    let parent_path = project::users_cfg_root_path_local();

    // make sure the parent directories exist
    fs::create_dir_all(&parent_path)?;

    for (path, policy_doc) in users {
        write_user_config_file(&parent_path.join(path), &policy_doc)?;
    }
    Ok(())
}

fn user_yaml_from_idx(jetty: &Jetty, idx: UserIndex) -> Result<UserYaml> {
    let ag = jetty.try_access_graph()?;
    let attributes = idx.get_attributes(jetty)?;
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
        member_of: idx
            .member_of_groups(jetty)?
            .into_iter()
            .map(|g| g.name(jetty).unwrap().to_string())
            .collect(),
    })
}

/// Add files for missing users and delete files for non-existent users.
pub fn update_user_files(jetty: &Jetty) -> Result<()> {
    let ag = jetty.try_access_graph()?;

    // get all the users in the config
    let validated_group_config = &parse_and_validate_groups(jetty)?;
    let configs = get_validated_file_config_map(jetty, validated_group_config)?;

    #[allow(clippy::unnecessary_to_owned)]
    let configs_local_name_map = configs
        .to_owned()
        .into_iter()
        .flat_map(|(path, user)| {
            user.identifiers
                .into_iter()
                .map(|(connector, id)| ((connector, id), path.to_owned()))
                .collect::<Vec<_>>()
        })
        .collect::<HashMap<(ConnectorNamespace, String), PathBuf>>();

    #[allow(clippy::unnecessary_to_owned)]
    let mut configs_node_name_map: HashMap<_, _> = configs
        .to_owned()
        .into_iter()
        .map(|(path, user)| (NodeName::User(user.name.to_owned()), (path, user)))
        .collect();

    // get all the users in the translator
    let translator_users = ag.translator().get_all_local_users();
    let translator_set: HashSet<_> = translator_users.keys().cloned().collect();

    // find users missing from the config
    let config_user_set = configs_local_name_map
        .keys()
        .cloned()
        .collect::<HashSet<_>>();
    let missing_users = translator_set
        .difference(&config_user_set)
        .collect::<HashSet<_>>();

    // if a user is missing, get it's node_name from the translator. If that name exists in the config, add this identifer.
    // If it doesn't, write a new file

    let mut write_list: HashSet<NodeName> = HashSet::new();
    let mut delete_list: HashSet<NodeName> = HashSet::new();
    for (conn, local_user) in missing_users {
        let node_name = translator_users[&(conn.to_owned(), local_user.to_owned())].to_owned();
        // if the node exists, update it
        if let Some((_path, user_yaml)) = configs_node_name_map.get_mut(&node_name) {
            user_yaml
                .identifiers
                .insert(conn.to_owned(), local_user.to_owned());
            write_list.insert(node_name.to_owned());
        }
        // if node doesn't exit, grab it from the graph and add it to the config list
        else {
            let idx = ag
                .get_user_index_from_name(&node_name)
                .ok_or_else(|| anyhow!("unable to find referenced user"))?;
            let user_yaml = user_yaml_from_idx(jetty, idx)?;
            configs_node_name_map.insert(
                node_name.to_owned(),
                (
                    project::users_cfg_root_path_local()
                        .join(get_filename_from_node_name(&node_name)),
                    user_yaml,
                ),
            );
            write_list.insert(node_name.to_owned());
        }
    }

    // find non-existent users in the config and remove from identities
    let extra_users = config_user_set
        .difference(&translator_set)
        .collect::<HashSet<_>>();

    for (conn, local_user) in extra_users {
        // a bit roundabout, but it works
        let file_name =
            configs_local_name_map[&(conn.to_owned(), local_user.to_owned())].to_owned();
        let node_name = NodeName::User(configs[&file_name].name.to_owned());

        if let Some((_path, user_yaml)) = configs_node_name_map.get_mut(&node_name) {
            user_yaml.identifiers.remove(conn);
            if user_yaml.identifiers.is_empty() {
                write_list.remove(&node_name);
                delete_list.insert(node_name.to_owned());
            } else {
                write_list.insert(node_name.to_owned());
            };
        }
    }

    // update files in the write list
    for node_name in write_list {
        let (path, user) = configs_node_name_map[&node_name].to_owned();
        let parent_path = project::users_cfg_root_path_local();
        // make sure the parent directories exist
        fs::create_dir_all(&parent_path)?;

        fs::write(path, yaml_peg::serde::to_string(&user)?)?;
    }

    // delete files in the delete list
    for node_name in delete_list {
        let (path, _user) = configs_node_name_map[&node_name].to_owned();

        fs::remove_file(path).context("removing nonexistent user from config")?;
    }

    // Delete any empty folders
    let user_directories = glob::glob(
        format!(
            "{}/**/",
            project::users_cfg_root_path_local().to_string_lossy()
        )
        .as_str(),
    )
    .context("trouble generating config directory paths")?;
    for dir in user_directories {
        fs::remove_dir(dir?).ok();
    }

    Ok(())
}

fn get_filename_from_node_name(name: &NodeName) -> PathBuf {
    format!("{}.yaml", clean_string_for_path(name.to_string())).into()
}

/// Write a UserYaml struct to a config file
pub(crate) fn write_user_config_file(path: &PathBuf, config: &UserYaml) -> Result<()> {
    let doc = yaml_peg::serde::to_string(config)?;
    fs::write(path, doc)?;
    Ok(())
}
