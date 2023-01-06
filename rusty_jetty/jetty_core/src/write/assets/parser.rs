//! Parse asset configuration files

use std::{
    collections::{BTreeSet, HashMap, HashSet},
    path::PathBuf,
};

use anyhow::{anyhow, bail, Context, Result};

use crate::{
    access_graph::{AccessGraph, AssetAttributes, NodeName, UserAttributes},
    connectors::AssetType,
    jetty::ConnectorNamespace,
    Jetty,
};

use super::{
    get_config_paths, CombinedPolicyState, DefaultPolicyState, PolicyState, YamlAssetDoc,
    YamlAssetIdentifier, YamlDefaultPolicy, YamlPolicy,
};

/// Parse the configuration into a policy state struct
pub(crate) fn parse_asset_config(
    val: &str,
    jetty: &Jetty,
    config_groups: &HashMap<String, HashMap<ConnectorNamespace, NodeName>>,
) -> Result<(YamlAssetIdentifier, CombinedPolicyState)> {
    let ag = jetty.try_access_graph()?;

    let config = simple_parse(val)?;

    // make sure the asset exists
    let asset_name = get_asset_name(
        &config.identifier.name,
        &config.identifier.asset_type,
        &config.identifier.connector,
        ag,
    )?;

    // Iterate through the normal policies
    let res_policies = parse_policies(
        &asset_name,
        &config.policies,
        &config.identifier.connector,
        config_groups,
        jetty,
    )?;

    // Iterate through the normal policies
    let res_default_policies = parse_default_policies(
        &asset_name,
        &config.default_policies,
        &config.identifier.connector,
        config_groups,
        jetty,
    )?;

    Ok((
        config.identifier,
        CombinedPolicyState {
            policies: res_policies,
            default_policies: res_default_policies,
        },
    ))
}

/// parse all configs into a map with the file path and YamlAssetDoc
pub(crate) fn parse_to_file_map() -> Result<HashMap<PathBuf, YamlAssetDoc>> {
    let mut res = HashMap::new();
    let paths = get_config_paths()?;
    for path in paths {
        let path = path?;
        let doc = std::fs::read_to_string(&path)
            .context(format!("problem reading {}", path.display()))?;
        let parsed_doc =
            simple_parse(&doc).context(format!("problem parsing {}", path.display()))?;
        res.insert(path.to_owned(), parsed_doc);
    }
    Ok(res)
}

/// parse a yaml file into a YamlAssetDoc with only syntactic validation
pub(crate) fn simple_parse(val: &str) -> Result<YamlAssetDoc> {
    let config_vec: Vec<YamlAssetDoc> = yaml_peg::serde::from_str(val)?;
    if config_vec.is_empty() {
        bail!("unable to parse configuration")
    };
    let config = config_vec[0].to_owned();
    Ok(config)
}

fn parse_policies(
    asset_name: &NodeName,
    policies: &BTreeSet<YamlPolicy>,
    connector: &ConnectorNamespace,
    config_groups: &HashMap<String, HashMap<ConnectorNamespace, NodeName>>,
    jetty: &Jetty,
) -> Result<HashMap<(NodeName, NodeName), PolicyState>> {
    let ag = jetty.try_access_graph()?;
    let mut res_policies = HashMap::new();
    for policy in policies {
        let policy_state = PolicyState {
            privileges: match &policy.privileges {
                Some(p) => p.iter().cloned().collect(),
                None => Default::default(),
            },
            metadata: Default::default(),
        };

        // validate groups
        if let Some(groups) = &policy.groups {
            for group in groups {
                let group_name = get_group_name(group, connector, config_groups)?;
                // Now add the matching group to the results map
                res_policies.insert(
                    (asset_name.to_owned(), group_name.to_owned()),
                    policy_state.to_owned(),
                );
            }
        };

        // Make sure all the users exist
        if let Some(users) = &policy.users {
            for user in users {
                let _user_name = get_user_name_and_connectors(user, ag)?;
                // Now add the matching user to the results map
                // Depending on whether its a default policy or not...

                res_policies.insert(
                    (asset_name.to_owned(), NodeName::User(user.to_owned())),
                    policy_state.to_owned(),
                );
            }
        };

        // Make sure the specified privileges are allowed/exist
        if let Some(privileges) = &policy.privileges {
            privileges_are_legal(privileges, asset_name, jetty, connector, None)?;
        }
    }
    Ok(res_policies)
}

#[allow(clippy::type_complexity)]
fn parse_default_policies(
    asset_name: &NodeName,
    default_policies: &BTreeSet<YamlDefaultPolicy>,
    connector: &ConnectorNamespace,
    config_groups: &HashMap<String, HashMap<ConnectorNamespace, NodeName>>,
    jetty: &Jetty,
) -> Result<HashMap<(NodeName, String, AssetType, NodeName), DefaultPolicyState>> {
    let ag = jetty.try_access_graph()?;
    let mut res_policies = HashMap::new();
    for policy in default_policies {
        // validate groups
        let groups: Option<BTreeSet<NodeName>> = if let Some(groups) = &policy.groups {
            Some(
                groups
                    .iter()
                    .map(|g| get_group_name(g, connector, config_groups))
                    .collect::<Result<BTreeSet<NodeName>>>()?,
            )
        } else {
            None
        };

        // validate users
        let users: Option<BTreeSet<NodeName>> = if let Some(users) = &policy.users {
            Some(
                users
                    .iter()
                    .map(|u| match get_user_name_and_connectors(u, ag) {
                        Ok((name, conns)) => {
                            if conns.contains(connector) {
                                Ok(name)
                            } else {
                                Err(anyhow!(
                                    "cannot set a {connector}-specific policy for non-{connector} user {u}"
                                ))
                            }
                        }

                        Err(e) => Err(anyhow!("{e}")),
                    })
                    .collect::<Result<BTreeSet<NodeName>>>()?,
            )
        } else {
            None
        };

        // make sure that types are specified and that they are all legal
        let allowed_types = jetty.connector_manifests()[connector]
            .asset_privileges
            .to_owned()
            .into_keys()
            .collect::<HashSet<_>>();
        if !allowed_types.contains(&policy.target_type) {
            bail!(
                "the type `{}` is not allowed for this connector",
                &policy.target_type.to_string()
            )
        }

        // Make sure the specified privileges are allowed/exist
        if let Some(privileges) = &policy.privileges {
            privileges_are_legal(
                privileges,
                asset_name,
                jetty,
                connector,
                Some(policy.target_type.to_owned()),
            )?;
        }

        // make sure the a path is legal
        path_is_legal(&policy.path)?;

        // now insert a policy for each user/group
        let mut agents = Vec::new();
        if let Some(some_users) = &users {
            agents.extend(some_users);
        }
        if let Some(some_groups) = &groups {
            agents.extend(some_groups);
        }

        for agent in agents {
            res_policies.insert(
                (
                    asset_name.to_owned(),
                    policy.path.to_owned(),
                    policy.target_type.to_owned(),
                    agent.to_owned(),
                ),
                DefaultPolicyState {
                    privileges: policy.privileges.to_owned().unwrap_or_default(),
                    metadata: policy
                        .metadata
                        .to_owned()
                        .unwrap_or_default()
                        .into_iter()
                        .collect(),
                    connector_managed: policy.connector_managed,
                },
            );
        }
    }

    Ok(res_policies)
}

/// Get a nodename for the given group. This checks the config to make sure that the group exists/will exist, and gets the appropriate name for the connector.
/// Returns an error if the group name isn't legal
fn get_group_name(
    group: &String,
    connector: &ConnectorNamespace,
    config_groups: &HashMap<String, HashMap<ConnectorNamespace, NodeName>>,
) -> Result<NodeName> {
    // make sure the groups exist. Needs info from the group parsing. Use the resolved group name
    let group_name = config_groups
        .get(group)
        .ok_or_else(|| anyhow!("unable to find a group called {group} in the configuration"))?
        .get(connector)
        .unwrap();
    Ok(group_name.to_owned())
}

/// Validate that a user exists and get their nodename
fn get_user_name_and_connectors(
    user: &String,
    ag: &AccessGraph,
) -> Result<(NodeName, HashSet<ConnectorNamespace>)> {
    let user_name = NodeName::User(user.to_owned());

    let user: UserAttributes = ag
        .get_node(&user_name)
        .context(format!(
            "looking for user \"{user}\": user does not appear to exist"
        ))?
        .to_owned()
        .try_into()?;
    Ok((user_name, user.connectors().to_owned()))
}

/// Validate that an asset exists and get its NodeName
fn get_asset_name(
    name: &String,
    asset_type: &Option<AssetType>,
    connector: &ConnectorNamespace,
    ag: &AccessGraph,
) -> Result<NodeName> {
    let matching_assets = ag
        .graph
        .nodes
        .assets
        .keys()
        .filter(|n| match n {
            NodeName::Asset {
                connector: ag_connector,
                asset_type: ag_asset_type,
                path,
            } => {
                connector == ag_connector
                    && asset_type == ag_asset_type
                    && &path.to_string() == name
            }
            _ => false,
        })
        .collect::<Vec<_>>();
    if matching_assets.is_empty() {
        bail!("unable to find asset referenced")
    }
    if matching_assets.len() > 1 {
        bail!("found too many matching assets 😳")
    }

    Ok(matching_assets[0].to_owned())
}

/// determine whether a set of privileges are legal for a policy
fn privileges_are_legal(
    privileges: &BTreeSet<String>,
    asset_name: &NodeName,
    jetty: &Jetty,
    connector: &ConnectorNamespace,
    target_type: Option<AssetType>,
) -> Result<()> {
    let ag = jetty.try_access_graph()?;
    let connector_privileges = &jetty.connector_manifests()[connector].asset_privileges;
    // if types were passed, it's a default policy
    let allowed_privilege_set = if let Some(t) = target_type {
        connector_privileges[&t].to_owned()
    }
    // Otherwise it's a normal policy, so get the allowed privileges for that type
    else {
        let asset_attribs = AssetAttributes::try_from(ag.get_node(asset_name)?.to_owned())?;

        connector_privileges[&asset_attribs.asset_type].to_owned()
    };
    for privilege in privileges {
        if !allowed_privilege_set.contains(privilege) {
            bail!("unsupported privilege: {privilege}")
        }
    }
    Ok(())
}

/// Validate that the wildcard path specified is allowed
fn path_is_legal(wildcard_path: &String) -> Result<()> {
    let segments = wildcard_path.split('/').collect::<Vec<_>>();
    let last_element_index = segments.len() - 1;
    for (idx, segment) in segments.into_iter().enumerate() {
        if segment.is_empty() {
            continue;
        }
        // we only allow wildcards
        if segment != "*" && segment != "**" {
            bail!("illegal wildcard path: {wildcard_path}; path elements must be `*` or `**`");
        }
        // "**" can only be used at the end
        if segment == "**" && idx != last_element_index {
            bail!("illegal wildcard path: {wildcard_path}; `**` can only be used at the end of a path");
        }
    }
    Ok(())
}
