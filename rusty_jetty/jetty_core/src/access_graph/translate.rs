//! Types and functionality to translate between connectors' local representation
//! and Jetty's global representation

use std::collections::HashMap;

use anyhow::Result;

use super::NodeName;
use crate::{
    connectors::{
        nodes::{ConnectorData, SparseMatrix, User},
        processed_nodes::{ProcessedConnectorData, ProcessedUser},
        UserIdentifier,
    },
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
    policies: SparseMatrix<ConnectorNamespace, NodeName, String>,
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
    ) -> Result<ProcessedConnectorData> {
        let mut result = ProcessedConnectorData::default();

        for (cd, namespace) in data {
            // convert the users
            result.users.extend(
                cd.users
                    .into_iter()
                    .map(|u| self.translate_user(u, namespace.to_owned()))
                    .collect::<Result<Vec<ProcessedUser>>>()?,
            );
        }

        Ok(result)
    }

    fn translate_user(&self, user: User, connector: ConnectorNamespace) -> Result<ProcessedUser> {
        Ok(ProcessedUser {
            name: self.local_to_global.users[&connector][&user.name].to_owned(),
            identifiers: user.identifiers,
            metadata: user.metadata,
            member_of: user
                .member_of
                .iter()
                .map(|g| self.local_to_global.groups[&connector][g].to_owned())
                .collect(),
            granted_by: todo!(),
            connector,
        })
    }
}
