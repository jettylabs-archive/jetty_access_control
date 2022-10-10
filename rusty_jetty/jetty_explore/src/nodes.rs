use std::sync::Arc;

use axum::{routing::get, Extension, Json, Router};
use jetty_core::access_graph::{AccessGraph, JettyNode};
use serde::{Deserialize, Serialize};

/// Return a router to handle all group-related requests
pub(super) fn router() -> Router {
    Router::new()
        .route("/nodes", get(get_nodes))
        .route("/users", get(get_users))
        .route("/assets", get(get_assets))
        .route("/groups", get(get_groups))
        .route("/tags", get(get_tags))
}

/// A simple type that corresponds to a node in the access graph.
#[derive(Serialize, Deserialize)]
struct Node {
    // TODO: Type the type field
    r#type: String,
    name: String,
    platforms: Vec<String>,
}

/// Return all nodes in the graph
async fn get_nodes(Extension(ag): Extension<Arc<AccessGraph>>) -> Json<Vec<Node>> {
    let nodes = get_all_nodes(ag);
    // Exclude the policy nodes
    let mut nodes: Vec<Node> = nodes.into_iter().filter(|n| n.r#type != "policy").collect();
    // sort on the server
    nodes.sort_by(|a, b| a.name.cmp(&b.name));

    Json(nodes)
}

/// Return all user nodes
async fn get_users(Extension(ag): Extension<Arc<AccessGraph>>) -> Json<Vec<Node>> {
    let nodes = get_all_nodes(ag);
    // Exclude the policy nodes
    let mut nodes: Vec<Node> = nodes.into_iter().filter(|n| n.r#type == "user").collect();
    // sort on the server
    nodes.sort_by(|a, b| a.name.cmp(&b.name));

    Json(nodes)
}

/// Return all asset nodes
async fn get_assets(Extension(ag): Extension<Arc<AccessGraph>>) -> Json<Vec<Node>> {
    let nodes = get_all_nodes(ag);
    // Exclude the policy nodes
    let mut nodes: Vec<Node> = nodes.into_iter().filter(|n| n.r#type == "asset").collect();
    // sort on the server
    nodes.sort_by(|a, b| a.name.cmp(&b.name));

    Json(nodes)
}

/// Return all group nodes
async fn get_groups(Extension(ag): Extension<Arc<AccessGraph>>) -> Json<Vec<Node>> {
    let nodes = get_all_nodes(ag);
    // Exclude the policy nodes
    let mut nodes: Vec<Node> = nodes.into_iter().filter(|n| n.r#type == "group").collect();
    // sort on the server
    nodes.sort_by(|a, b| a.name.cmp(&b.name));

    Json(nodes)
}

/// Return all tag nodes
async fn get_tags(Extension(ag): Extension<Arc<AccessGraph>>) -> Json<Vec<Node>> {
    let nodes = get_all_nodes(ag);
    // Exclude the policy nodes
    let mut nodes: Vec<Node> = nodes.into_iter().filter(|n| n.r#type == "tag").collect();
    // sort on the server
    nodes.sort_by(|a, b| a.name.cmp(&b.name));

    Json(nodes)
}

/// Pull all the nodes out of the graph and convert them in to a format that
/// explore can use.
fn get_all_nodes(ag: Arc<AccessGraph>) -> Vec<Node> {
    ag.get_nodes()
        .map(|(_, n)| {
            let node_type = match n {
                JettyNode::Group(_) => "group".to_owned(),
                JettyNode::User(_) => "user".to_owned(),
                JettyNode::Asset(_) => "asset".to_owned(),
                JettyNode::Tag(_) => "tag".to_owned(),
                JettyNode::Policy(_) => "policy".to_owned(),
            };

            Node {
                r#type: node_type,
                name: n.get_node_name(),
                platforms: n
                    .get_node_connectors()
                    .iter()
                    .map(|n| n.to_owned())
                    .collect(),
            }
        })
        .collect()
}
