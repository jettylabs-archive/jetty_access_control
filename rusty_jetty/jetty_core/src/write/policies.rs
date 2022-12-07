//! Parse and manage user-configured policies

use std::collections::HashSet;

use crate::{access_graph::NodeName, jetty::ConnectorNamespace};

pub(crate) struct Diff {
    pub(crate) asset: NodeName,
    pub(crate) agent: NodeName,
    // This is a vec because you might both add and remove users
    pub(crate) details: Vec<DiffDetails>,
    pub(crate) connectors: HashSet<ConnectorNamespace>,
}

pub(crate) enum DiffDetails {
    AddPolicy {
        privileges: HashSet<String>,
    },
    RemovePolicy,
    ModifyPolicy {
        add: HashSet<String>,
        remove: HashSet<String>,
    },
}
