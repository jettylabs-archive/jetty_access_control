//! Types for node summaries that are sent to the web app

use std::collections::HashSet;

use jetty_core::{
    access_graph::{JettyNode, NodeName},
    connectors::AssetType,
    jetty::ConnectorNamespace,
    Connector,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Eq, PartialEq)]
pub(crate) enum NodeSummary {
    Asset {
        name: NodeName,
        asset_type: AssetType,
        connectors: HashSet<ConnectorNamespace>,
    },
    User {
        name: NodeName,
        connectors: HashSet<ConnectorNamespace>,
    },
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
            JettyNode::User(n) => NodeSummary::User {
                name: n.name,
                connectors: n.connectors,
            },
            JettyNode::Asset(n) => NodeSummary::Asset {
                name: n.name,
                asset_type: n.asset_type,
                connectors: n.connectors,
            },
            JettyNode::Tag(_) => todo!(),
            JettyNode::Policy(_) => todo!(),
        }
    }
}
