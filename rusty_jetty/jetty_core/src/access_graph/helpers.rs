//! Helpers to represent data on its way into the graph

use std::collections::HashSet;

use super::{
    connectors::nodes, AssetAttributes, EdgeType, GroupAttributes, JettyEdge, JettyNode, NodeName,
    PolicyAttributes, TagAttributes, UserAttributes,
};

#[derive(Debug)]
/// Wrapper for including the connector name and updated
/// connector data.
pub struct ProcessedConnectorData {
    /// Connector name to identify where this data came from
    pub connector: String,
    /// Connector data straight from the source
    pub data: nodes::ConnectorData,
}

/// All helper types implement NodeHelpers.
pub(crate) trait NodeHelper {
    /// Return a JettyNode from the helper
    fn get_node(&self, connector: String) -> JettyNode;
    /// Return a set of JettyEdges from the helper
    fn get_edges(&self) -> HashSet<JettyEdge>;
}

impl NodeHelper for nodes::Group {
    fn get_node(&self, connector: String) -> JettyNode {
        JettyNode::Group(GroupAttributes {
            name: self.name.to_owned(),
            metadata: self.metadata.to_owned(),
            connectors: HashSet::from([connector]),
        })
    }

    fn get_edges(&self) -> HashSet<JettyEdge> {
        let mut hs = HashSet::<JettyEdge>::new();
        for v in &self.member_of {
            insert_edge_pair(
                &mut hs,
                NodeName::Group(self.name.to_owned()),
                NodeName::Group(v.to_owned()),
                EdgeType::MemberOf,
            );
        }
        for v in &self.includes_users {
            insert_edge_pair(
                &mut hs,
                NodeName::Group(self.name.to_owned()),
                NodeName::User(v.to_owned()),
                EdgeType::Includes,
            );
        }
        for v in &self.includes_groups {
            insert_edge_pair(
                &mut hs,
                NodeName::Group(self.name.to_owned()),
                NodeName::Group(v.to_owned()),
                EdgeType::Includes,
            );
        }
        for v in &self.granted_by {
            insert_edge_pair(
                &mut hs,
                NodeName::Group(self.name.to_owned()),
                NodeName::Policy(v.to_owned()),
                EdgeType::GrantedBy,
            );
        }
        hs
    }
}

/// Object used to populate user nodes and edges in the graph

impl NodeHelper for nodes::User {
    fn get_node(&self, connector: String) -> JettyNode {
        JettyNode::User(UserAttributes {
            name: self.name.to_owned(),
            identifiers: self.identifiers.to_owned(),
            other_identifiers: self.other_identifiers.to_owned(),
            metadata: self.metadata.to_owned(),
            connectors: HashSet::from([connector]),
        })
    }

    fn get_edges(&self) -> HashSet<JettyEdge> {
        let mut hs = HashSet::<JettyEdge>::new();
        for v in &self.member_of {
            insert_edge_pair(
                &mut hs,
                NodeName::User(self.name.to_owned()),
                NodeName::Group(v.to_owned()),
                EdgeType::MemberOf,
            );
        }
        for v in &self.granted_by {
            insert_edge_pair(
                &mut hs,
                NodeName::User(self.name.to_owned()),
                NodeName::Policy(v.to_owned()),
                EdgeType::GrantedBy,
            );
        }
        hs
    }
}

impl NodeHelper for nodes::Asset {
    fn get_node(&self, connector: String) -> JettyNode {
        JettyNode::Asset(AssetAttributes {
            cual: self.cual.clone(),
            asset_type: self.asset_type.to_owned(),
            metadata: self.metadata.to_owned(),
            connectors: HashSet::from([connector]),
        })
    }

    fn get_edges(&self) -> HashSet<JettyEdge> {
        let mut hs = HashSet::<JettyEdge>::new();
        for v in &self.governed_by {
            insert_edge_pair(
                &mut hs,
                NodeName::Asset(self.cual.uri()),
                NodeName::Policy(v.to_owned()),
                EdgeType::GovernedBy,
            );
        }
        for v in &self.child_of {
            insert_edge_pair(
                &mut hs,
                NodeName::Asset(self.cual.uri()),
                NodeName::Asset(v.to_owned()),
                EdgeType::ChildOf,
            );
        }
        for v in &self.parent_of {
            insert_edge_pair(
                &mut hs,
                NodeName::Asset(self.cual.uri()),
                NodeName::Asset(v.to_owned()),
                EdgeType::ParentOf,
            );
        }
        for v in &self.derived_from {
            insert_edge_pair(
                &mut hs,
                NodeName::Asset(self.cual.uri()),
                NodeName::Asset(v.to_owned()),
                EdgeType::DerivedFrom,
            );
        }
        for v in &self.derived_to {
            insert_edge_pair(
                &mut hs,
                NodeName::Asset(self.cual.uri()),
                NodeName::Asset(v.to_owned()),
                EdgeType::DerivedTo,
            );
        }
        for v in &self.tagged_as {
            insert_edge_pair(
                &mut hs,
                NodeName::Asset(self.cual.uri()),
                NodeName::Tag(v.to_owned()),
                EdgeType::TaggedAs,
            );
        }
        hs
    }
}

impl NodeHelper for nodes::Tag {
    fn get_node(&self, _connector: String) -> JettyNode {
        JettyNode::Tag(TagAttributes {
            name: self.name.to_owned(),
            value: self.value.to_owned(),
            description: self.description.to_owned(),
            pass_through_hierarchy: self.pass_through_hierarchy,
            pass_through_lineage: self.pass_through_lineage,
        })
    }

    fn get_edges(&self) -> HashSet<JettyEdge> {
        let mut hs = HashSet::<JettyEdge>::new();
        for v in &self.applied_to {
            insert_edge_pair(
                &mut hs,
                NodeName::Tag(self.name.to_owned()),
                NodeName::Asset(v.to_owned()),
                EdgeType::AppliedTo,
            );
        }
        for v in &self.governed_by {
            insert_edge_pair(
                &mut hs,
                NodeName::Tag(self.name.to_owned()),
                NodeName::Policy(v.to_owned()),
                EdgeType::GovernedBy,
            );
        }
        hs
    }
}

impl NodeHelper for nodes::Policy {
    fn get_node(&self, connector: String) -> JettyNode {
        JettyNode::Policy(PolicyAttributes {
            name: self.name.to_owned(),
            privileges: self.privileges.to_owned(),
            pass_through_hierarchy: self.pass_through_hierarchy,
            pass_through_lineage: self.pass_through_lineage,
            connectors: HashSet::from([connector]),
        })
    }

    fn get_edges(&self) -> HashSet<JettyEdge> {
        let mut hs = HashSet::<JettyEdge>::new();
        for v in &self.governs_assets {
            insert_edge_pair(
                &mut hs,
                NodeName::Policy(self.name.to_owned()),
                NodeName::Asset(v.to_owned()),
                EdgeType::Governs,
            );
        }
        for v in &self.governs_tags {
            insert_edge_pair(
                &mut hs,
                NodeName::Policy(self.name.to_owned()),
                NodeName::Tag(v.to_owned()),
                EdgeType::Governs,
            );
        }
        for v in &self.granted_to_groups {
            insert_edge_pair(
                &mut hs,
                NodeName::Policy(self.name.to_owned()),
                NodeName::Group(v.to_owned()),
                EdgeType::GrantedTo,
            );
        }
        for v in &self.granted_to_users {
            insert_edge_pair(
                &mut hs,
                NodeName::Policy(self.name.to_owned()),
                NodeName::User(v.to_owned()),
                EdgeType::GrantedTo,
            );
        }
        hs
    }
}

fn insert_edge_pair(
    hs: &mut HashSet<JettyEdge>,
    from: NodeName,
    to: NodeName,
    edge_type: EdgeType,
) {
    hs.insert(JettyEdge {
        from: from.to_owned(),
        to: to.to_owned(),
        edge_type: edge_type.to_owned(),
    });
    hs.insert(JettyEdge {
        from: to,
        to: from,
        edge_type: super::get_edge_type_pair(&edge_type),
    });
}
