//! Types and functionality to translate between connectors' local representation
//! and Jetty's global representation

pub mod diffs;

use std::collections::{HashMap, HashSet};

use super::NodeName;
use crate::{
    connectors::{
        nodes::{
            ConnectorData, EffectivePermission, RawAsset, RawAssetReference, RawDefaultPolicy,
            RawGroup, RawPolicy, RawPolicyGrantee, RawTag, RawUser, SparseMatrix,
        },
        processed_nodes::{
            ProcessedAsset, ProcessedAssetReference, ProcessedConnectorData,
            ProcessedDefaultPolicy, ProcessedGroup, ProcessedPolicy, ProcessedTag, ProcessedUser,
        },
        UserIdentifier,
    },
    cual::Cual,
    jetty::ConnectorNamespace,
    log_runtime,
    logging::debug,
    permissions::matrix::{DoubleInsert, InsertOrMerge},
    write::{groups::parse_and_validate_groups, users},
    Jetty,
};

use anyhow::{anyhow, bail, Context, Result};
use bimap;
use serde::{Deserialize, Serialize};
use url::Url;

/// Struct to translate local data to global data and back again
/// Eventually, this will need to be persisted with the graph to enable the write path
#[derive(Default, Serialize, Deserialize)]
pub struct Translator {
    global_to_local: GlobalToLocalIdentifiers,
    local_to_global: LocalToGlobalIdentifiers,
    cual_prefix_to_namespace: bimap::BiHashMap<Option<String>, ConnectorNamespace>,
}

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct GlobalToLocalIdentifiers {
    users: SparseMatrix<ConnectorNamespace, NodeName, String>,
    groups: SparseMatrix<ConnectorNamespace, NodeName, String>,
    policies: SparseMatrix<ConnectorNamespace, NodeName, String>,
}

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct LocalToGlobalIdentifiers {
    users: SparseMatrix<ConnectorNamespace, String, NodeName>,
    groups: SparseMatrix<ConnectorNamespace, String, NodeName>,
    policies: SparseMatrix<ConnectorNamespace, String, NodeName>,
}

impl Translator {
    /// Use the ConnectorData from all connectors to populate the mappings
    pub fn new(data: &[(ConnectorData, ConnectorNamespace)], jetty: &Jetty) -> Result<Self> {
        let mut t = Translator::default();

        // build the namespace mapping
        t.build_cual_namespace_map(data);

        // Start by pulling out all the user nodes and resolving them to single identities
        t.resolve_users(data, jetty)?;
        t.resolve_groups(data);
        t.resolve_policies(data);

        Ok(t)
    }

    fn build_cual_namespace_map(&mut self, data: &[(ConnectorData, ConnectorNamespace)]) {
        for (ConnectorData { cual_prefix, .. }, namespace) in data {
            self.cual_prefix_to_namespace
                .insert(cual_prefix.to_owned(), namespace.to_owned());
        }
    }

    // This is entity resolution for users. Right now it is very simple, but it can be built out as needed
    fn resolve_users(
        &mut self,
        data: &[(ConnectorData, ConnectorNamespace)],
        jetty: &Jetty,
    ) -> Result<()> {
        let user_data: Vec<_> = data.iter().map(|(c, n)| (&c.users, n)).collect();
        // get all the users in the config
        // FUTURE: We end up parsing the group config too many times. Try to centralize this, perhaps as part of the Jetty struct
        let user_config_id_map =
            if let Ok(validated_group_config) = &parse_and_validate_groups(jetty) {
                users::parser::get_validated_nodename_local_id_map(jetty, validated_group_config)?
            } else {
                Default::default()
            };

        // for each connector, look over all the users.
        for (users, namespace) in user_data {
            for user in users {
                // if a user exists in the config, just use that mapping
                let node_name = if let Some(name) = user_config_id_map
                    .get(namespace)
                    .and_then(|m| m.get_by_right(&user.name))
                {
                    name.to_owned()
                }
                // if no user exists in the config, use their email if possible, or just the connector_specific id
                else {
                    let mut node_name = NodeName::User(user.name.to_owned());
                    for id in &user.identifiers {
                        //If they have an Email address, make that the identifier.
                        if let UserIdentifier::Email(email) = id {
                            node_name = NodeName::User(email.to_owned())
                        }
                    }
                    node_name
                };

                self.local_to_global.users.double_insert(
                    namespace.to_owned(),
                    user.name.to_owned(),
                    node_name.to_owned(),
                );
                self.global_to_local.users.double_insert(
                    namespace.to_owned(),
                    node_name,
                    user.name.to_owned(),
                );
            }
        }
        Ok(())
    }

    /// This resolves groups. When we start allowing cross-platform Jetty groups, this will need an update.
    /// This takes the name of a group and creates a NodeName::Group from it
    fn resolve_groups(&mut self, data: &[(ConnectorData, ConnectorNamespace)]) {
        let group_data: Vec<_> = data.iter().map(|(c, n)| (&c.groups, n)).collect();
        // for each connector, look over all the users.
        for (groups, namespace) in group_data {
            for group in groups {
                self.local_to_global.groups.double_insert(
                    namespace.to_owned(),
                    group.name.to_owned(),
                    NodeName::Group {
                        name: group.name.to_owned(),
                        origin: namespace.to_owned(),
                    },
                );
                self.global_to_local.groups.double_insert(
                    namespace.to_owned(),
                    NodeName::Group {
                        name: group.name.to_owned(),
                        origin: namespace.to_owned(),
                    },
                    group.name.to_owned(),
                );
            }
        }
    }

    /// This resolves policies. When we start allowing cross-platform Jetty policies, this will need an update.
    /// This takes the name of a policy and creates a NodeName::Policy from it
    fn resolve_policies(&mut self, data: &[(ConnectorData, ConnectorNamespace)]) {
        let policy_data: Vec<_> = data.iter().map(|(c, n)| (&c.policies, n)).collect();
        // for each connector, look over all the policies.
        for (policies, namespace) in policy_data {
            for policy in policies {
                self.local_to_global.policies.double_insert(
                    namespace.to_owned(),
                    policy.name.to_owned(),
                    NodeName::Policy {
                        name: policy.name.to_owned(),
                        origin: namespace.to_owned(),
                    },
                );
                self.global_to_local.policies.double_insert(
                    namespace.to_owned(),
                    NodeName::Policy {
                        name: policy.name.to_owned(),
                        origin: namespace.to_owned(),
                    },
                    policy.name.to_owned(),
                );
            }
        }
    }

    /// Translate locally scoped connector data to globally scoped processed connector data
    pub fn local_to_processed_connector_data(
        &self,
        data: Vec<(ConnectorData, ConnectorNamespace)>,
    ) -> ProcessedConnectorData {
        let mut result = log_runtime!(
            "translate ep axes",
            ProcessedConnectorData {
                effective_permissions: self
                    .translate_effective_permissions_axes_to_global_node_names(&data),
                ..Default::default()
            }
        );

        for (cd, namespace) in data {
            // convert the users
            log_runtime!(
                "translate users",
                result.users.extend(
                    cd.users
                        .into_iter()
                        .map(|u| self.translate_user_to_global(u, namespace.to_owned()))
                        .collect::<Vec<ProcessedUser>>(),
                )
            );
            // convert the groups
            log_runtime!(
                "translate groups",
                result.groups.extend(
                    cd.groups
                        .into_iter()
                        .map(|g| self.translate_group_to_global(g, namespace.to_owned()))
                        .collect::<Vec<ProcessedGroup>>(),
                )
            );
            // convert the assets
            log_runtime!(
                "translate assets",
                result.assets.extend(
                    cd.assets
                        .into_iter()
                        .map(|a| self.translate_asset_to_global(a, namespace.to_owned()))
                        .collect::<Vec<ProcessedAsset>>(),
                )
            );
            // convert the tags
            log_runtime!(
                "translate tags",
                result.tags.extend(
                    cd.tags
                        .into_iter()
                        .map(|t| self.translate_tag_to_global(t, namespace.to_owned()))
                        .collect::<Vec<ProcessedTag>>(),
                )
            );
            // convert the policies
            log_runtime!(
                "translate policies",
                result.policies.extend(
                    cd.policies
                        .into_iter()
                        .map(|p| self.translate_policy_to_global(p, namespace.to_owned()))
                        .collect::<Vec<ProcessedPolicy>>(),
                )
            );
            // convert the assets
            log_runtime!(
                "translate assets",
                result.asset_references.extend(
                    cd.asset_references
                        .into_iter()
                        .filter_map(|a| {
                            self.translate_asset_reference_to_global(a, namespace.to_owned())
                        })
                        .collect::<Vec<ProcessedAssetReference>>(),
                )
            );
            // convert the default policies
            log_runtime!(
                "translate default policies",
                result.default_policies.extend(
                    cd.default_policies
                        .into_iter()
                        .map(|a| self.translate_default_policy_to_global(a, namespace.to_owned()))
                        .collect::<Vec<ProcessedDefaultPolicy>>(),
                )
            );
        }

        result
    }

    /// Convert node from connector into ProcessedNode by converting all references to global NodeNames
    fn translate_user_to_global(
        &self,
        user: RawUser,
        connector: ConnectorNamespace,
    ) -> ProcessedUser {
        ProcessedUser {
            name: self.local_to_global.users[&connector][&user.name].to_owned(),
            identifiers: user.identifiers,
            metadata: user.metadata,
            member_of: user
                .member_of
                .iter()
                .map(|g| self.local_to_global.groups[&connector][g].to_owned())
                .collect(),
            granted_by: user
                .granted_by
                .iter()
                .map(|g| self.local_to_global.policies[&connector][g].to_owned())
                .collect(),
            connector,
        }
    }

    /// Convert node from connector into ProcessedNode by converting all references to global NodeNames
    fn translate_group_to_global(
        &self,
        group: RawGroup,
        connector: ConnectorNamespace,
    ) -> ProcessedGroup {
        ProcessedGroup {
            name: self.local_to_global.groups[&connector][&group.name].to_owned(),
            metadata: group.metadata,
            member_of: group
                .member_of
                .iter()
                .map(|g| self.local_to_global.groups[&connector][g].to_owned())
                .collect(),
            includes_users: group
                .includes_users
                .iter()
                .map(|u| self.local_to_global.users[&connector][u].to_owned())
                .collect(),
            includes_groups: group
                .includes_groups
                .iter()
                .map(|g| self.local_to_global.groups[&connector][g].to_owned())
                .collect(),
            granted_by: group
                .granted_by
                .iter()
                .map(|p| self.local_to_global.policies[&connector][p].to_owned())
                .collect(),
            connector,
        }
    }

    /// Convert node from connector into ProcessedNode by converting all references to global NodeNames
    fn translate_asset_to_global(
        &self,
        asset: RawAsset,
        connector: ConnectorNamespace,
    ) -> ProcessedAsset {
        ProcessedAsset {
            name: self.cual_to_asset_name(asset.cual).unwrap(),
            asset_type: asset.asset_type,
            metadata: asset.metadata,
            governed_by: asset
                .governed_by
                .iter()
                .map(|g| self.local_to_global.policies[&connector][g].to_owned())
                .collect(),
            child_of: asset
                .child_of
                .into_iter()
                .map(|g| self.cual_to_asset_name(Cual::new(g.as_str())).unwrap())
                .collect(),
            parent_of: asset
                .parent_of
                .into_iter()
                .map(|g| self.cual_to_asset_name(Cual::new(g.as_str())).unwrap())
                .collect(),
            derived_from: asset
                .derived_from
                .into_iter()
                .filter_map(|g| self.cual_to_asset_name(Cual::new(g.as_str())).ok())
                .collect(),
            derived_to: asset
                .derived_to
                .into_iter()
                .filter_map(|g| self.cual_to_asset_name(Cual::new(g.as_str())).ok())
                .collect(),
            tagged_as: asset.tagged_as.into_iter().map(NodeName::Tag).collect(),
            connector,
        }
    }

    /// Convert node from connector into ProcessedNode by converting all references to global NodeNames
    fn translate_asset_reference_to_global(
        &self,
        asset: RawAssetReference,
        connector: ConnectorNamespace,
    ) -> Option<ProcessedAssetReference> {
        let name = self.cual_to_asset_name(asset.cual).ok()?;
        Some(ProcessedAssetReference {
            name,
            metadata: asset.metadata,
            governed_by: asset
                .governed_by
                .iter()
                .map(|g| self.local_to_global.policies[&connector][g].to_owned())
                .collect(),
            child_of: asset
                .child_of
                .into_iter()
                .filter_map(|g| self.cual_to_asset_name(Cual::new(g.as_str())).ok())
                .collect(),
            parent_of: asset
                .parent_of
                .into_iter()
                .filter_map(|g| self.cual_to_asset_name(Cual::new(g.as_str())).ok())
                .collect(),
            derived_from: asset
                .derived_from
                .into_iter()
                .filter_map(|g| self.cual_to_asset_name(Cual::new(g.as_str())).ok())
                .collect(),
            derived_to: asset
                .derived_to
                .into_iter()
                .filter_map(|g| self.cual_to_asset_name(Cual::new(g.as_str())).ok())
                .collect(),
            tagged_as: asset.tagged_as.into_iter().map(NodeName::Tag).collect(),
        })
    }

    /// Convert node from connector into ProcessedNode by converting all references to global NodeNames
    fn translate_tag_to_global(&self, tag: RawTag, connector: ConnectorNamespace) -> ProcessedTag {
        ProcessedTag {
            name: NodeName::Tag(tag.name),
            value: tag.value,
            description: tag.description,
            pass_through_hierarchy: tag.pass_through_hierarchy,
            pass_through_lineage: tag.pass_through_lineage,
            applied_to: tag
                .applied_to
                .into_iter()
                .map(|t| self.cual_to_asset_name(Cual::new(t.as_str())).unwrap())
                .collect(),
            removed_from: tag
                .removed_from
                .into_iter()
                .map(|t| self.cual_to_asset_name(Cual::new(t.as_str())).unwrap())
                .collect(),
            governed_by: tag
                .governed_by
                .iter()
                .map(|p| self.local_to_global.policies[&connector][p].to_owned())
                .collect(),
            connector,
        }
    }

    /// Convert node from connector into ProcessedNode by converting all references to global NodeNames
    fn translate_policy_to_global(
        &self,
        policy: RawPolicy,
        connector: ConnectorNamespace,
    ) -> ProcessedPolicy {
        ProcessedPolicy {
            name: self.local_to_global.policies[&connector][&policy.name].to_owned(),
            privileges: policy.privileges,
            governs_assets: policy
                .governs_assets
                .into_iter()
                .map(|a| self.cual_to_asset_name(Cual::new(a.as_str())).unwrap())
                .collect(),
            governs_tags: policy.governs_tags.into_iter().map(NodeName::Tag).collect(),
            granted_to_groups: policy
                .granted_to_groups
                .iter()
                .map(|g| self.local_to_global.groups[&connector][g].to_owned())
                .collect(),
            granted_to_users: policy
                .granted_to_users
                .iter()
                .map(|u| self.local_to_global.users[&connector][u].to_owned())
                .collect(),
            pass_through_hierarchy: policy.pass_through_hierarchy,
            pass_through_lineage: policy.pass_through_lineage,
            connector,
        }
    }

    /// Convert node from raw to processed default policy
    fn translate_default_policy_to_global(
        &self,
        policy: RawDefaultPolicy,
        connector: ConnectorNamespace,
    ) -> ProcessedDefaultPolicy {
        let root_node = self.cual_to_asset_name(policy.root_asset).unwrap();
        let grantee = match policy.grantee {
            RawPolicyGrantee::Group(g) => self.local_to_global.groups[&connector][&g].to_owned(),
            RawPolicyGrantee::User(u) => {
                match self.local_to_global.users.get(&connector) {
                    Some(m) => match m.get(&u) {
                        Some(_) => (),
                        None => println!("Unable to find user {u} in map: {m:?}"),
                    },
                    None => println!("Unable to find connector {connector}"),
                }
                self.local_to_global.users[&connector][&u].to_owned()
            }
        };
        ProcessedDefaultPolicy {
            name: NodeName::DefaultPolicy {
                root_node: Box::new(root_node.to_owned()),
                matching_path: policy.wildcard_path.to_owned(),
                target_type: policy.target_type.to_owned(),
                grantee: Box::new(grantee.to_owned()),
            },
            privileges: policy.privileges,
            root_node,
            matching_path: policy.wildcard_path,
            target_type: policy.target_type,
            grantee,
            connector,
            metadata: policy.metadata.to_owned(),
        }
    }

    /// Take the permissions from a connector and convert them to a single matrix using
    /// NodeNames. After this conversion there is still one more step - they need to be converted
    /// into a NodeIndex-axis matrix.
    fn translate_effective_permissions_axes_to_global_node_names(
        &self,
        data: &Vec<(ConnectorData, ConnectorNamespace)>,
    ) -> SparseMatrix<NodeName, NodeName, HashSet<EffectivePermission>> {
        let mut result: SparseMatrix<NodeName, NodeName, HashSet<EffectivePermission>> =
            SparseMatrix::new();

        for (c_data, namespace) in data {
            for (k1, v1) in &c_data.effective_permissions {
                for (k2, v2) in v1 {
                    result.insert_or_merge(
                        self.local_to_global.users[namespace][k1].to_owned(),
                        HashMap::from([(
                            self.cual_to_asset_name(k2.to_owned()).unwrap(),
                            v2.to_owned(),
                        )]),
                    );
                }
            }
        }
        result
    }

    /// Convert a cual to NodeName::Asset
    pub fn cual_to_asset_name(&self, cual: Cual) -> Result<NodeName> {
        let connector_prefix = cual.connector_prefix();
        let connector = self
            .cual_prefix_to_namespace
            .get_by_left(&connector_prefix)
            .context(format!(
                "unable to match cual prefix ({:?})",
                &connector_prefix
            ))?
            .to_owned();

        Ok(NodeName::Asset {
            connector,
            asset_type: cual.asset_type(),
            path: cual.asset_path(),
        })
    }

    /// Convert a NodeName::Asset to cual
    pub fn asset_name_to_cual(&self, asset_name: &NodeName) -> Result<Cual> {
        match asset_name {
            NodeName::Asset {
                connector,
                asset_type,
                path,
            } => {
                let prefix = self
                    .cual_prefix_to_namespace
                    .get_by_right(connector)
                    .context("unable to find cual prefix")?
                    .to_owned()
                    .unwrap_or_default();
                let path = path
                    .components()
                    .iter()
                    .map(|c| urlencoding::encode(c))
                    .collect::<Vec<_>>()
                    .join("/");

                let mut uri = Url::parse(format!("{prefix}/{path}").as_str())?;
                match asset_type {
                    Some(asset_type) => {
                        uri.set_query(Some(format!("type={}", asset_type.to_string()).as_str()))
                    }
                    None => (),
                }
                Ok(Cual::new(uri.as_str()))
            }
            _ => bail!("cannot convert non-asset to cual"),
        }
    }

    pub(crate) fn translate_node_name_to_local(
        &self,
        node_name: &NodeName,
        connector: &ConnectorNamespace,
    ) -> String {
        match &node_name {
            NodeName::User(_n) => self.global_to_local.users[connector][node_name].to_owned(),
            // There may be groups that don't exist yet, so we'll just use the group name without the origin
            NodeName::Group { name, .. } => name.to_owned(),
            NodeName::Asset { .. } => {
                todo!()
            }
            NodeName::Policy { .. } => {
                self.global_to_local.policies[connector][node_name].to_owned()
            }
            NodeName::Tag(t) => t.to_owned(),
            // Default policies don't have names
            NodeName::DefaultPolicy { .. } => "".into(),
        }
    }

    pub(crate) fn try_translate_node_name_to_local(
        &self,
        node_name: &NodeName,
        connector: &ConnectorNamespace,
    ) -> Result<String> {
        Ok(match &node_name {
            NodeName::User(_n) => self
                .global_to_local
                .users
                .get(connector)
                .ok_or_else(|| anyhow!("unable to find connector for node translation"))?
                .get(node_name)
                .ok_or_else(|| anyhow!("unable to find username for connection"))?
                .to_owned(),
            // There may be groups that don't exist yet, so we'll just use the group name without the origin
            NodeName::Group { name, .. } => name.to_owned(),
            NodeName::Asset { .. } => {
                todo!()
            }
            NodeName::Policy { .. } => self
                .global_to_local
                .policies
                .get(connector)
                .ok_or_else(|| anyhow!("unable to find connector for node translation"))?
                .get(node_name)
                .ok_or_else(|| anyhow!("unable to find username for collection"))?
                .to_owned(),
            NodeName::Tag(t) => t.to_owned(),
            // Default policies don't have names
            NodeName::DefaultPolicy { .. } => "".into(),
        })
    }

    pub(crate) fn get_all_local_users(&self) -> HashMap<(ConnectorNamespace, String), NodeName> {
        self.local_to_global
            .users
            .iter()
            .flat_map(|(connector, user_map)| {
                user_map.iter().map(|(k, node_name)| {
                    ((connector.to_owned(), k.to_owned()), node_name.to_owned())
                })
            })
            .collect()
    }

    /// update the translator to reflect the new mapping of a local name to a jetty name
    /// This can be used whether the node exists already or is entirely new
    pub(crate) fn modify_user_mapping(
        &mut self,
        connector: &ConnectorNamespace,
        local_name: &String,
        old_node: &NodeName,
        new_node: &NodeName,
    ) -> Result<()> {
        let global_to_local = self
            .global_to_local
            .users
            .get_mut(connector)
            .ok_or_else(|| anyhow!("unable to find connector for user map update"))?;
        global_to_local.remove(old_node);
        global_to_local.insert(new_node.to_owned(), local_name.to_owned());

        let local_to_global = self
            .local_to_global
            .users
            .get_mut(connector)
            .ok_or_else(|| anyhow!("unable to find connector for user map update"))?;
        local_to_global.remove(local_name);
        local_to_global.insert(local_name.to_owned(), new_node.to_owned());

        Ok(())
    }
}
