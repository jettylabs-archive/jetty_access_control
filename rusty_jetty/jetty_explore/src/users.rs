use std::sync::Arc;

use anyhow::Context;
use axum::{extract::Path, routing::get, Extension, Json, Router};
use uuid::Uuid;

use crate::{
    node_summaries::NodeSummary, NodeSummaryWithPaths, NodeSummaryWithPrivileges,
    SummaryWithAssociatedSummaries,
};

use jetty_core::access_graph::{self, EdgeType, JettyNode};

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
    Path(node_id): Path<Uuid>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<NodeSummaryWithPrivileges>> {
    let from = ag
        .get_user_index_from_id(&node_id)
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
            .map(|(k, v)| NodeSummaryWithPrivileges {
                node: k.to_owned().into(),
                privileges: v.iter().copied().map(|p| p.to_owned()).collect(),
            })
            .collect(),
    )
}

/// Return information about a users access to tagged assets, grouped by tag
async fn tags_handler(
    Path(node_id): Path<Uuid>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<SummaryWithAssociatedSummaries>> {
    let from = ag
        .get_user_index_from_id(&node_id)
        .context("fetching user node")
        .unwrap();

    // get all the user_accessable assets
    let tag_asset_map = ag.get_user_accessible_tags(from);

    let response = tag_asset_map
        .into_iter()
        .map(|(t, v)| (&ag[t], v))
        .map(|(t, v)| SummaryWithAssociatedSummaries {
            node: t.to_owned().into(),
            associations: v
                .into_iter()
                .map(|a| NodeSummary::from(ag[a].to_owned()))
                .collect(),
        })
        .collect::<Vec<_>>();

    Json(response)
}

/// Returns groups that user is a direct member of
async fn direct_groups_handler(
    Path(node_id): Path<Uuid>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<NodeSummary>> {
    let from = ag
        .get_user_index_from_id(&node_id)
        .context("fetching user node")
        .unwrap();

    let group_nodes = ag.get_matching_descendants(
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
            let jetty_node = &ag.graph()[i];
            if let JettyNode::Group(_) = jetty_node {
                Some(jetty_node.to_owned().into())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    Json(group_attributes)
}

/// Returns groups that user is an inherited member of
async fn inherited_groups_handler(
    Path(node_id): Path<Uuid>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<NodeSummaryWithPaths>> {
    let from = ag
        .get_user_index_from_id(&node_id)
        .context("fetching user node")
        .unwrap();

    let res = ag.all_matching_simple_paths_to_descendants(
        from,
        |n| matches!(n, EdgeType::MemberOf),
        |n| matches!(n, JettyNode::Group(_)),
        |n| matches!(n, JettyNode::Group(_)),
        // looking for inherited groups, so skip the first level of connection, which would be the
        // directly assigned (rather than inherited groups)
        Some(2),
        None,
    );

    let group_attributes = res
        .into_iter()
        .filter_map(|(i, p)| {
            let jetty_node = &ag.graph()[i];
            if let JettyNode::Group(_) = jetty_node {
                Some(NodeSummaryWithPaths {
                    node: jetty_node.to_owned().into(),
                    paths: p
                        .iter()
                        .map(|q| {
                            ag.path_as_jetty_nodes(q)
                                .iter()
                                .map(|v| NodeSummary::from((*v).to_owned()))
                                .collect()
                        })
                        .collect(),
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    Json(group_attributes)
}
