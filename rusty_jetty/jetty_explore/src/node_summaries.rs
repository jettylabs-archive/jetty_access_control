//! Types for node summaries that are sent to the web app

use std::collections::HashSet;

use jetty_core::{
    access_graph::{JettyNode, NodeName},
    jetty::ConnectorNamespace,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Eq, PartialEq)]
pub(crate) enum NodeSummary {
    Asset,
    User,
    Group {
        name: NodeName,
        connectors: HashSet<ConnectorNamespace>,
    },
    Tag,
    Policy,
}

impl From<JettyNode> for NodeSummary {
    fn from(node: JettyNode) -> Self {
        match node {
            JettyNode::Group(n) => NodeSummary::Group {
                name: n.name,
                connectors: n.connectors,
            },
            JettyNode::User(_) => todo!(),
            JettyNode::Asset(_) => todo!(),
            JettyNode::Tag(_) => todo!(),
            JettyNode::Policy(_) => todo!(),
        }
    }
}
