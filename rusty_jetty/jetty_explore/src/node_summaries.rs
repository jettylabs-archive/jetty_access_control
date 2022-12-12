//! Types for node summaries that are sent to the web app

use std::collections::HashSet;

use jetty_core::{
    access_graph::{JettyNode, NodeName},
    connectors::AssetType,
    jetty::ConnectorNamespace,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone)]
pub(crate) enum NodeSummary {
    Asset {
        name: NodeName,
        id: Uuid,
        asset_type: AssetType,
        connectors: HashSet<ConnectorNamespace>,
    },
    User {
        name: NodeName,
        id: Uuid,
        connectors: HashSet<ConnectorNamespace>,
    },
    Group {
        name: NodeName,
        id: Uuid,
        connectors: HashSet<ConnectorNamespace>,
    },
    Tag {
        name: NodeName,
        id: Uuid,
        description: Option<String>,
        pass_through_hierarchy: bool,
        pass_through_lineage: bool,
        connectors: HashSet<ConnectorNamespace>,
    },
    Policy {
        name: NodeName,
        connectors: HashSet<ConnectorNamespace>,
    },
    DefaultPolicy {
        name: NodeName,
        connectors: HashSet<ConnectorNamespace>,
    },
}

impl NodeSummary {
    pub(crate) fn get_name(&self) -> NodeName {
        match self {
            NodeSummary::Asset { name, .. } => name,
            NodeSummary::User { name, .. } => name,
            NodeSummary::Group { name, .. } => name,
            NodeSummary::Tag { name, .. } => name,
            NodeSummary::Policy { name, .. } => name,
            NodeSummary::DefaultPolicy { name, .. } => name,
        }
        .clone()
    }
}

impl From<JettyNode> for NodeSummary {
    fn from(node: JettyNode) -> Self {
        match node {
            JettyNode::Group(n) => NodeSummary::Group {
                name: n.name,
                id: n.id,
                connectors: n.connectors,
            },
            JettyNode::User(n) => NodeSummary::User {
                name: n.name,
                id: n.id,
                connectors: n.connectors,
            },
            JettyNode::Asset(n) => NodeSummary::Asset {
                name: n.name,
                id: n.id,
                asset_type: n.asset_type,
                connectors: n.connectors,
            },
            JettyNode::Tag(n) => NodeSummary::Tag {
                name: n.name,
                id: n.id,
                description: n.description,
                pass_through_hierarchy: n.pass_through_hierarchy,
                pass_through_lineage: n.pass_through_lineage,
                connectors: n.connectors,
            },
            JettyNode::Policy(n) => NodeSummary::Policy {
                name: n.name,
                connectors: n.connectors,
            },
            JettyNode::DefaultPolicy(n) => NodeSummary::DefaultPolicy {
                name: n.name,
                connectors: n.connectors,
            },
        }
    }
}
