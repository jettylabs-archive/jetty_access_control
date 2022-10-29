use std::sync::Arc;

use axum::{routing::get, Extension, Json, Router};
use jetty_core::access_graph::AccessGraph;

use crate::node_summaries::NodeSummary;

/// Return a router to handle all group-related requests
pub(super) fn router() -> Router {
    Router::new()
        .route("/nodes", get(get_nodes))
        .route("/users", get(get_users))
        .route("/assets", get(get_assets))
        .route("/groups", get(get_groups))
        .route("/tags", get(get_tags))
}

/// Return all nodes in the graph
async fn get_nodes(Extension(ag): Extension<Arc<AccessGraph>>) -> Json<Vec<NodeSummary>> {
    let nodes = get_all_nodes(ag);
    // Exclude the policy nodes
    let mut nodes: Vec<_> = nodes
        .into_iter()
        .filter(|n| !matches!(n, NodeSummary::Policy { .. }))
        .collect();
    // sort on the server
    nodes.sort_by_key(|a| a.get_name());

    Json(nodes)
}

/// Return all user nodes
async fn get_users(Extension(ag): Extension<Arc<AccessGraph>>) -> Json<Vec<NodeSummary>> {
    let nodes = get_all_nodes(ag);
    // Limit to user nodes
    let mut nodes: Vec<_> = nodes
        .into_iter()
        .filter(|n| matches!(n, NodeSummary::User { .. }))
        .collect();
    // sort on the server
    nodes.sort_by_key(|a| a.get_name());

    Json(nodes)
}

/// Return all asset nodes
async fn get_assets(Extension(ag): Extension<Arc<AccessGraph>>) -> Json<Vec<NodeSummary>> {
    let nodes = get_all_nodes(ag);
    // Limit to asset nodes
    let mut nodes: Vec<_> = nodes
        .into_iter()
        .filter(|n| matches!(n, NodeSummary::Asset { .. }))
        .collect();
    // sort on the server
    nodes.sort_by_key(|a| a.get_name());

    Json(nodes)
}

/// Return all group nodes
async fn get_groups(Extension(ag): Extension<Arc<AccessGraph>>) -> Json<Vec<NodeSummary>> {
    let nodes = get_all_nodes(ag);
    // Limit to group nodes
    let mut nodes: Vec<_> = nodes
        .into_iter()
        .filter(|n| matches!(n, NodeSummary::Group { .. }))
        .collect();
    // sort on the server
    nodes.sort_by_key(|a| a.get_name());

    Json(nodes)
}

/// Return all tag nodes
async fn get_tags(Extension(ag): Extension<Arc<AccessGraph>>) -> Json<Vec<NodeSummary>> {
    let nodes = get_all_nodes(ag);
    // Limit to tag nodes
    let mut nodes: Vec<_> = nodes
        .into_iter()
        .filter(|n| matches!(n, NodeSummary::Tag { .. }))
        .collect();
    // sort on the server
    nodes.sort_by_key(|a| a.get_name());

    Json(nodes)
}

/// Pull all the nodes out of the graph and convert them in to a format that
/// explore can use.
fn get_all_nodes(ag: Arc<AccessGraph>) -> Vec<NodeSummary> {
    ag.get_nodes()
        .map(|(_, n)| n.clone())
        .map(NodeSummary::from)
        .collect()
}
