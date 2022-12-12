//! Parse and manage user-configured policies

mod bootstrap;

use std::collections::{HashMap, HashSet};

use crate::{access_graph::NodeName, jetty::ConnectorNamespace};

pub(crate) struct Diff {
    pub(crate) assets: Vec<NodeName>,
    pub(crate) agents: Vec<NodeName>,
    // This is a vec because you might both add and remove users
    pub(crate) details: Vec<DiffDetails>,
    pub(crate) connectors: HashSet<ConnectorNamespace>,
}

pub(crate) enum DiffDetails {
    AddPolicy {
        privileges: PolicyChanges,
    },
    RemovePolicy,
    ModifyPolicy {
        add: PolicyChanges,
        remove: PolicyChanges,
    },
}

pub(crate) struct PolicyChanges {
    privileges: HashSet<String>,
    metadata: HashMap<String, String>,
}
