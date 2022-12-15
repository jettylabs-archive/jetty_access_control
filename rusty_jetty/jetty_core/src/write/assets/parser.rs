//! Parse asset configuration files

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use anyhow::{anyhow, bail, Result};
use graphviz_rust::attributes::width;

use crate::{
    access_graph::{AssetAttributes, NodeName},
    connectors::AssetType,
    jetty::ConnectorNamespace,
    Jetty,
};

use super::{CombinedPolicyState, PolicyState, YamlAssetDoc};

pub(crate) fn parse_asset_config(
    val: &str,
    jetty: &Jetty,
    config_groups: &BTreeMap<String, BTreeMap<ConnectorNamespace, NodeName>>,
) -> Result<CombinedPolicyState> {
    let config_vec: Vec<YamlAssetDoc> = yaml_peg::serde::from_str(val)?;
    if config_vec.is_empty() {
        bail!("unable to parse configuration")
    };
    let mut res_policies = HashMap::new();
    let mut res_default_policies = HashMap::new();
    let config = config_vec[0].to_owned();
    let ag = jetty.try_access_graph()?;

    // make sure the asset exists
    let matching_assets = ag
        .graph
        .nodes
        .assets
        .keys()
        .filter(|n| match n {
            NodeName::Asset {
                connector,
                asset_type,
                path,
            } => {
                if connector == &config.identifier.connector
                    && asset_type == &config.identifier.asset_type
                    && &path.to_string() == &config.identifier.name
                {
                    true
                } else {
                    false
                }
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

    let asset_name = matching_assets[0].to_owned();

    // Iterate through the policies
    for policy in &config.policies {
        let policy_state = PolicyState {
            privileges: match &policy.privileges {
                Some(p) => p.to_owned().into_iter().collect(),
                None => Default::default(),
            },
            metadata: Default::default(),
        };

        // validate the wildcard path, if it exists
        if let Some(wildcard_path) = &policy.path {
            path_is_legal(wildcard_path)?
        }

        // validate groups
        if let Some(groups) = &policy.groups {
            for group in groups {
                // make sure the groups exist. Needs info from the group parsing. Use the resolved group name
                let group_name = config_groups
                    .get(group)
                    .ok_or(anyhow!(
                        "unable to find a group called {group} in the configuration"
                    ))?
                    .get(&config.identifier.connector.to_owned())
                    .unwrap();

                // Now add the matching group to the results map

                // Depending on whether its a default policy or not...
                match &policy.path {
                    Some(p) => {
                        res_default_policies.insert(
                            (
                                asset_name.to_owned(),
                                group_name.to_owned(),
                                p.to_owned(),
                                policy.types.to_owned(),
                            ),
                            policy_state.to_owned(),
                        );
                    }
                    None => {
                        res_policies.insert(
                            (asset_name.to_owned(), group_name.to_owned()),
                            policy_state.to_owned(),
                        );
                    }
                }
            }
        };

        // Make sure all the users exist
        if let Some(users) = &policy.users {
            for user in users {
                let matching_users = ag
                    .graph
                    .nodes
                    .users
                    .keys()
                    .filter(|n| match n {
                        NodeName::User(graph_user) => {
                            if graph_user == user {
                                true
                            } else {
                                false
                            }
                        }
                        _ => false,
                    })
                    .collect::<Vec<_>>();
                if matching_users.is_empty() {
                    bail!("unable to find user: {user}")
                }
                if matching_users.len() > 1 {
                    bail!("found too many matching users for {user} 😳")
                }
                // Now add the matching user to the results map
                // Depending on whether its a default policy or not...
                match &policy.path {
                    Some(p) => {
                        res_default_policies.insert(
                            (
                                asset_name.to_owned(),
                                NodeName::User(user.to_owned()),
                                p.to_owned(),
                                policy.types.to_owned(),
                            ),
                            policy_state.to_owned(),
                        );
                    }
                    None => {
                        res_policies.insert(
                            (asset_name.to_owned(), NodeName::User(user.to_owned())),
                            policy_state.to_owned(),
                        );
                    }
                }
            }
        };
        // Make sure the specified privileges are allowed/exist
        // if it's a default policy (path exists), then allow any platform privilege. If not, match based on type.
        let allowed_privilege_set = if let Some(_) = policy.path {
            let binding = &jetty.connector_manifests()[&config.identifier.connector];
            binding
                .asset_privileges
                .values()
                .flatten()
                .map(|v| v.to_owned())
                .collect::<HashSet<_>>()
        } else {
            let asset_attribs = AssetAttributes::try_from(ag.get_node(&asset_name)?.to_owned())?;

            jetty.connector_manifests()[&config.identifier.connector].asset_privileges
                [&asset_attribs.asset_type]
                .to_owned()
        };
        for privilege in &policy_state.privileges {
            if !allowed_privilege_set.contains(privilege) {
                bail!("unsupported privilege: {privilege}")
            }
        }

        // if types are specified, make sure they are applied, and that a path is supplied
        if let Some(types) = &policy.types {
            if let None = policy.path {
                bail!("types can only be specified for default policies: this policy doesn't have the `path` attribute")
            }
            let allowed_types = jetty.connector_manifests()[&config.identifier.connector]
                .asset_privileges
                .to_owned()
                .into_keys()
                .collect::<HashSet<_>>();
            for asset_type in types {
                if !allowed_types.contains(&asset_type) {
                    bail!(
                        "the type `{}` is not allowed for this connector",
                        asset_type.to_string()
                    )
                }
            }
        }

        // Now, if it is a default policy, expand the config to
    }

    Ok(CombinedPolicyState {
        policies: res_policies,
        default_policies: res_default_policies,
    })
}

fn path_is_legal(wildcard_path: &String) -> Result<()> {
    let segments = wildcard_path.split("/").collect::<Vec<_>>();
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
