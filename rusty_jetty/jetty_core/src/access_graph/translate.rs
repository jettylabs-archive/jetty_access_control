//! Types and functionality to translate between connectors' local representation
//! and Jetty's global representation

use super::NodeName;
use crate::{
    connectors::{
        nodes::{Asset, ConnectorData, Group, Policy, SparseMatrix, Tag, User},
        processed_nodes::{
            ProcessedAsset, ProcessedConnectorData, ProcessedGroup, ProcessedPolicy, ProcessedTag,
            ProcessedUser,
        },
        UserIdentifier,
    },
    cual::Cual,
    jetty::ConnectorNamespace,
    permissions::matrix::DoubleInsert,
};

/// Struct to translate local data to global data and back again
#[derive(Default)]
pub struct Translator {
    global_to_local: GlobalToLocalIdentifiers,
    local_to_global: LocalToGlobalIdentifiers,
}

#[derive(Default)]
pub(crate) struct GlobalToLocalIdentifiers {
    users: SparseMatrix<ConnectorNamespace, NodeName, String>,
    groups: SparseMatrix<ConnectorNamespace, NodeName, String>,
}

#[derive(Default)]
pub(crate) struct LocalToGlobalIdentifiers {
    users: SparseMatrix<ConnectorNamespace, String, NodeName>,
    groups: SparseMatrix<ConnectorNamespace, String, NodeName>,
    policies: SparseMatrix<ConnectorNamespace, String, NodeName>,
}

impl Translator {
    /// Use the ConnectorData from all connectors to populate the mappings
    pub fn new(data: &Vec<(ConnectorData, ConnectorNamespace)>) -> Self {
        let mut t = Translator::default();

        // Start by pulling out all the user nodes and resolving them to single identities
        t.resolve_users(data);
        t.resolve_groups(data);
        t.resolve_policies(data);
        t
    }

    /// This is entity resolution for users. Right now it is very simple, but it can be built out as needed
    fn resolve_users(&mut self, data: &Vec<(ConnectorData, ConnectorNamespace)>) {
        let user_data: Vec<_> = data.iter().map(|(c, n)| (&c.users, n)).collect();
        // for each connector, look over all the users.
        for connector in user_data {
            for user in connector.0 {
                for id in &user.identifiers {
                    //If they have an Email address, make that the identifier.
                    if let UserIdentifier::Email(email) = id {
                        self.local_to_global.users.double_insert(
                            connector.1.to_owned(),
                            user.name.to_owned(),
                            NodeName::User(email.to_owned()),
                        );
                        self.global_to_local.users.double_insert(
                            connector.1.to_owned(),
                            NodeName::User(email.to_owned()),
                            user.name.to_owned(),
                        );
                    }
                    // Otherwise, use whatever the connector is using for their name
                    else {
                        self.local_to_global.users.double_insert(
                            connector.1.to_owned(),
                            user.name.to_owned(),
                            NodeName::User(user.name.to_owned()),
                        );
                        self.global_to_local.users.double_insert(
                            connector.1.to_owned(),
                            NodeName::User(user.name.to_owned()),
                            user.name.to_owned(),
                        );
                    }
                }
            }
        }
    }

    /// This resolves groups. When we start allowing cross-platform Jetty groups, this will need an update.
    /// This takes the name of a group and creates a NodeName::Group from it
    fn resolve_groups(&mut self, data: &Vec<(ConnectorData, ConnectorNamespace)>) {
        let group_data: Vec<_> = data.iter().map(|(c, n)| (&c.groups, n)).collect();
        // for each connector, look over all the users.
        for connector in group_data {
            for group in connector.0 {
                self.local_to_global.groups.double_insert(
                    connector.1.to_owned(),
                    group.name.to_owned(),
                    NodeName::Group {
                        name: group.name.to_owned(),
                        origin: connector.1.to_owned(),
                    },
                );
                self.global_to_local.groups.double_insert(
                    connector.1.to_owned(),
                    NodeName::Group {
                        name: group.name.to_owned(),
                        origin: connector.1.to_owned(),
                    },
                    group.name.to_owned(),
                );
            }
        }
    }

    /// This resolves policies. When we start allowing cross-platform Jetty policies, this will need an update.
    /// This takes the name of a policy and creates a NodeName::Policy from it
    fn resolve_policies(&mut self, data: &Vec<(ConnectorData, ConnectorNamespace)>) {
        let policy_data: Vec<_> = data.iter().map(|(c, n)| (&c.policies, n)).collect();
        // for each connector, look over all the users.
        for connector in policy_data {
            for policy in connector.0 {
                self.local_to_global.policies.double_insert(
                    connector.1.to_owned(),
                    policy.name.to_owned(),
                    NodeName::Policy {
                        name: policy.name.to_owned(),
                        origin: connector.1.to_owned(),
                    },
                );
                self.global_to_local.groups.double_insert(
                    connector.1.to_owned(),
                    NodeName::Policy {
                        name: policy.name.to_owned(),
                        origin: connector.1.to_owned(),
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
        let mut result = ProcessedConnectorData::default();

        for (cd, namespace) in data {
            // convert the users
            result.users.extend(
                cd.users
                    .into_iter()
                    .map(|u| self.translate_user(u, namespace.to_owned()))
                    .collect::<Vec<ProcessedUser>>(),
            );
            // convert the groups
            result.groups.extend(
                cd.groups
                    .into_iter()
                    .map(|g| self.translate_group(g, namespace.to_owned()))
                    .collect::<Vec<ProcessedGroup>>(),
            );
            // convert the assets
            result.assets.extend(
                cd.assets
                    .into_iter()
                    .map(|a| self.translate_asset(a, namespace.to_owned()))
                    .collect::<Vec<ProcessedAsset>>(),
            );
            // convert the tags
            result.tags.extend(
                cd.tags
                    .into_iter()
                    .map(|t| self.translate_tag(t, namespace.to_owned()))
                    .collect::<Vec<ProcessedTag>>(),
            );
            // convert the policies
            result.policies.extend(
                cd.policies
                    .into_iter()
                    .map(|p| self.translate_policy(p, namespace.to_owned()))
                    .collect::<Vec<ProcessedPolicy>>(),
            );
        }

        result
    }

    fn translate_user(&self, user: User, connector: ConnectorNamespace) -> ProcessedUser {
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

    fn translate_group(&self, group: Group, connector: ConnectorNamespace) -> ProcessedGroup {
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

    fn translate_asset(&self, asset: Asset, connector: ConnectorNamespace) -> ProcessedAsset {
        ProcessedAsset {
            name: NodeName::Asset(asset.cual),
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
                .map(|g| NodeName::Asset(Cual::new(g.as_str())))
                .collect(),
            parent_of: asset
                .parent_of
                .into_iter()
                .map(|g| NodeName::Asset(Cual::new(g.as_str())))
                .collect(),
            derived_from: asset
                .derived_from
                .into_iter()
                .map(|g| NodeName::Asset(Cual::new(g.as_str())))
                .collect(),
            derived_to: asset
                .derived_to
                .into_iter()
                .map(|g| NodeName::Asset(Cual::new(g.as_str())))
                .collect(),
            tagged_as: asset
                .tagged_as
                .into_iter()
                .map(|t| NodeName::Tag(t))
                .collect(),
            connector,
        }
    }

    fn translate_tag(&self, tag: Tag, connector: ConnectorNamespace) -> ProcessedTag {
        ProcessedTag {
            name: NodeName::Tag(tag.name),
            value: tag.value,
            description: tag.description,
            pass_through_hierarchy: tag.pass_through_hierarchy,
            pass_through_lineage: tag.pass_through_lineage,
            applied_to: tag
                .applied_to
                .into_iter()
                .map(|t| NodeName::Asset(Cual::new(t.as_str())))
                .collect(),
            removed_from: tag
                .removed_from
                .into_iter()
                .map(|t| NodeName::Asset(Cual::new(t.as_str())))
                .collect(),
            governed_by: tag
                .governed_by
                .iter()
                .map(|p| self.local_to_global.policies[&connector][p].to_owned())
                .collect(),
            connector,
        }
    }

    fn translate_policy(&self, policy: Policy, connector: ConnectorNamespace) -> ProcessedPolicy {
        ProcessedPolicy {
            name: self.local_to_global.policies[&connector][&policy.name].to_owned(),
            privileges: policy.privileges,
            governs_assets: policy
                .governs_assets
                .into_iter()
                .map(|a| NodeName::Asset(Cual::new(a.as_str())))
                .collect(),
            governs_tags: policy
                .governs_tags
                .into_iter()
                .map(|t| NodeName::Tag(t))
                .collect(),
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
}
