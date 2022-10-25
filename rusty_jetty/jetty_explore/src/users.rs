use std::{collections::HashSet, sync::Arc};

use anyhow::Context;
use axum::{extract::Path, routing::get, Extension, Json, Router};
use serde::Serialize;

use crate::{
    node_summaries::NodeSummary, NodeWithPrivileges, PrivilegeResponse, UserAssetsResponse,
};

use super::ObjectWithPathResponse;
use jetty_core::{
    access_graph::{self, EdgeType, JettyNode, NodeName},
    connectors::UserIdentifier,
};

/// Return a router to handle all user-related requests
pub(super) fn router() -> Router {
    Router::new()
        .route("/:user_name/assets", get(assets_handler))
        .route("/:user_name/tags", get(tags_handler))
        .route("/:user_name/direct_groups", get(direct_groups_handler))
        .route(
            "/:user_name/inherited_groups",
            get(inherited_groups_handler),
        )
}

/// Return information about a user's access to assets, including privilege and explanation
async fn assets_handler(
    Path(node_id): Path<String>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<NodeWithPrivileges>> {
    let from = ag
        .get_user_index_from_name(&NodeName::User(node_id))
        .context("fetching user node")
        .unwrap();
    // use the effective permissions to get all the assets that a user has access to
    let assets_and_permissions = ag.get_user_accessible_assets(from);
    // get the name and connectors from each asset

    Json(
        assets_and_permissions
            .iter()
            // get the JettyNodes for all of the accessible assets
            .map(|(k, v)| (&ag[*k], v))
            // adding second map for clarity
            // build Vec of UserAssetResponse structs
            .map(|(k, v)| NodeWithPrivileges {
                node: k.to_owned().into(),
                privileges: v.to_owned().into_iter().map(|p| p.to_owned()).collect(),
            })
            .collect(),
    )
}

#[derive(Serialize)]
pub(crate) struct NodeWithListOfNodes {
    node: NodeSummary,
    list: Vec<NodeSummary>,
}

/// Return information about a users access to tagged assets, grouped by tag
async fn tags_handler(
    Path(node_id): Path<String>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<NodeWithListOfNodes>> {
    let from = ag
        .get_user_index_from_name(&NodeName::User(node_id))
        .context("fetching user node")
        .unwrap();

    // get all the user_accessable assets
    let tag_asset_map = ag.get_user_accessible_tags(from);

    let response = tag_asset_map
        .into_iter()
        .map(|(t, v)| (&ag[t], v))
        .map(|(t, v)| NodeWithListOfNodes {
            node: t.to_owned().into(),
            list: v
                .into_iter()
                .map(|a| NodeSummary::from(ag[a].to_owned()))
                .collect(),
        })
        .collect::<Vec<_>>();

    Json(response)
}

/// Returns groups that user is a direct member of
async fn direct_groups_handler(
    Path(node_id): Path<String>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<access_graph::GroupAttributes>> {
    let from = ag
        .get_user_index_from_name(&NodeName::User(node_id))
        .context("fetching user node")
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

/// Returns groups that user is an inherited member of
async fn inherited_groups_handler(
    Path(node_id): Path<String>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<ObjectWithPathResponse>> {
    let from = ag
        .get_user_index_from_name(&NodeName::User(node_id))
        .context("fetching user node")
        .unwrap();

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
