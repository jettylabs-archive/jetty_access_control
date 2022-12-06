//! Parse and manage user-configured groups

mod parser;

use serde::Deserialize;

use crate::jetty::ConnectorNamespace;

/// group configuration, as represented in the yaml
#[derive(Deserialize, Debug)]
pub(crate) struct GroupConfig {
    name: String,
    connector_names: Option<Vec<ConnectorName>>,
    members: GroupMembers,
    pos: u64,
}

#[derive(Deserialize, Debug)]
pub(crate) struct ConnectorName {
    connector: ConnectorNamespace,
    group_name: String,
    pos: u64,
}

#[derive(Deserialize, Debug)]
pub(crate) struct GroupMembers {
    groups: Option<Vec<MemberGroup>>,
    users: Option<Vec<MemberUser>>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct MemberGroup {
    name: String,
    pos: u64,
}

#[derive(Deserialize, Debug)]
pub(crate) struct MemberUser {
    name: String,
    pos: u64,
}

// Parse groups config into this struct

// Diff with existing graph

// Send diff to connectors
