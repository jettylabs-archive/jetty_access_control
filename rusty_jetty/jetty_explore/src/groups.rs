use std::sync::Arc;

use anyhow::Context;
use axum::{extract::Path, routing::get, Extension, Json, Router};
use jetty_core::{
    access_graph::{self, EdgeType, JettyNode, NodeName},
    jetty::ConnectorNamespace,
};

use super::ObjectWithPathResponse;

/// Return a router to handle all group-related requests
pub(super) fn router() -> Router {
    Router::new()
        .route("/:node_id/direct_groups", get(direct_groups_handler))
        .route("/:node_id/inherited_groups", get(inherited_groups_handler))
        .route(
            "/:node_id/direct_members_groups",
            get(direct_members_groups_handler),
        )
        .route(
            "/:node_id/direct_members_users",
            get(direct_members_users_handler),
        )
        .route("/:node_id/all_members", get(all_members_handler))
}

/// Return the groups that this group is a direct member of
async fn direct_groups_handler(
    Path(node_id): Path<String>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<access_graph::GroupAttributes>> {
    // Group names in the url will be written as origin::group_name, so
    // we need to parse that out
    // Eventually, we could switch this to a hash
    let (origin, name) = node_id.split_once("::").unwrap();
    let from = ag
        .get_group_index_from_name(&NodeName::Group {
            name: name.to_owned(),
            origin: ConnectorNamespace(origin.to_owned()),
        })
        .context("fetching group node")
        .unwrap();

    let group_nodes = ag.get_matching_children(
        from,
        |n| matches!(n, EdgeType::MemberOf),
        |n| matches!(n, JettyNode::Group(_)),
        |n| matches!(n, JettyNode::Group(_)),
        None,
        Some(1),
    );

    let group_attributes = group_nodes
        .into_iter()
        .filter_map(|i| {
            if let JettyNode::Group(g) = &ag.graph()[i] {
                Some(g.to_owned())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    Json(group_attributes)
}

/// Return the groups that this group is an inherited member of
async fn inherited_groups_handler(
    Path(node_id): Path<String>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<ObjectWithPathResponse>> {
    // Group names in the url will be written as origin::group_name, so
    // we need to parse that out
    // Eventually, we could switch this to a hash
    let (origin, name) = node_id.split_once("::").unwrap();
    let from = ag
        .get_group_index_from_name(&NodeName::Group {
            name: name.to_owned(),
            origin: ConnectorNamespace(origin.to_owned()),
        })
        .context("fetching group node")
        .unwrap();

    // return simple paths to all group children
    let res = ag.all_matching_simple_paths_to_children(
        from,
        |n| matches!(n, EdgeType::MemberOf),
        |n| matches!(n, JettyNode::Group(_)),
        |n| matches!(n, JettyNode::Group(_)),
        None,
        None,
    );

    let group_attributes = res
        .into_iter()
        .filter_map(|(i, p)| {
            if let JettyNode::Group(g) = &ag.graph()[i] {
                Some(ObjectWithPathResponse {
                    name: g.name.to_string(),
                    connectors: g.connectors.iter().map(|n| n.to_string()).collect(),
                    membership_paths: p.iter().map(|p| ag.path_as_string(p)).collect(),
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    Json(group_attributes)
}

/// Return the groups that are direct members of this group
async fn direct_members_groups_handler(
    Path(node_id): Path<String>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<access_graph::GroupAttributes>> {
    // Group names in the url will be written as origin::group_name, so
    // we need to parse that out
    // Eventually, we could switch this to a hash
    let (origin, name) = node_id.split_once("::").unwrap();
    let from = ag
        .get_group_index_from_name(&NodeName::Group {
            name: name.to_owned(),
            origin: ConnectorNamespace(origin.to_owned()),
        })
        .context("fetching group node")
        .unwrap();

    let group_nodes = ag.get_matching_children(
        from,
        |n| matches!(n, EdgeType::Includes),
        |n| matches!(n, JettyNode::Group(_)),
        |n| matches!(n, JettyNode::Group(_)),
        None,
        Some(1),
    );

    let group_attributes = group_nodes
        .into_iter()
        .filter_map(|i| {
            if let JettyNode::Group(g) = &ag.graph()[i] {
                Some(g.to_owned())
            } else {
                panic!("found wrong node type - expected group")
            }
        })
        .collect::<Vec<_>>();

    Json(group_attributes)
}

/// Return the users that are direct members of this group
async fn direct_members_users_handler(
    Path(node_id): Path<String>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<access_graph::UserAttributes>> {
    // Group names in the url will be written as origin::group_name, so
    // we need to parse that out
    // Eventually, we could switch this to a hash
    let (origin, name) = node_id.split_once("::").unwrap();
    let from = ag
        .get_group_index_from_name(&NodeName::Group {
            name: name.to_owned(),
            origin: ConnectorNamespace(origin.to_owned()),
        })
        .context("fetching group node")
        .unwrap();
    let group_nodes = ag.get_matching_children(
        from,
        |n| matches!(n, EdgeType::Includes),
        |n| matches!(n, JettyNode::Group(_)),
        |n| matches!(n, JettyNode::User(_)),
        None,
        Some(1),
    );

    let user_attributes = group_nodes
        .into_iter()
        .filter_map(|i| {
            if let JettyNode::User(u) = &ag.graph()[i] {
                Some(u.to_owned())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    Json(user_attributes)
}

/// Return all users that are members of the group, directly or through inheritance
async fn all_members_handler(
    Path(node_id): Path<String>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<ObjectWithPathResponse>> {
    // Group names in the url will be written as origin::group_name, so
    // we need to parse that out
    // Eventually, we could switch this to a hash
    let (origin, name) = node_id.split_once("::").unwrap();
    let from = ag
        .get_group_index_from_name(&NodeName::Group {
            name: name.to_owned(),
            origin: ConnectorNamespace(origin.to_owned()),
        })
        .context("fetching group node")
        .unwrap();

    let res = ag.all_matching_simple_paths_to_children(
        from,
        |n| matches!(n, EdgeType::Includes),
        |n| matches!(n, JettyNode::Group(_)),
        |n| matches!(n, JettyNode::User(_)),
        None,
        None,
    );

    let group_attributes = res
        .into_iter()
        .filter_map(|(i, p)| {
            if let JettyNode::User(u) = &ag.graph()[i] {
                Some(ObjectWithPathResponse {
                    name: u.name.to_string(),
                    connectors: u.connectors.iter().map(|n| n.to_string()).collect(),
                    membership_paths: p.iter().map(|p| ag.path_as_string(p)).collect(),
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    Json(group_attributes)
}
