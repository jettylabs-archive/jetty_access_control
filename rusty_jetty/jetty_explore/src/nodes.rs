use axum::{routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;

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
async fn get_nodes() -> Json<Vec<Node>> {
    let val = json! {[
      { "type": "user", "name": "Isaac", "platforms": ["snowflake", "tableau"] },
      { "type": "asset", "name": "table1", "platforms": ["snowflake"] },
      { "type": "group", "name": "Group 1", "platforms": ["tableau"] },
      { "type": "tag", "name": "my_tag", "platforms": ["jetty"] }
    ]};

    let mut node_info: Vec<Node> = serde_json::from_value(val).unwrap();

    // sort on the server
    node_info.sort_by(|a, b| a.name.cmp(&b.name));

    Json(node_info)
}

/// Return all user nodes
async fn get_users() -> Json<Vec<Node>> {
    let val = json! {[
      { "type": "user", "name": "Isaac", "platforms": ["snowflake", "tableau"] },
    ]};

    let mut node_info: Vec<Node> = serde_json::from_value(val).unwrap();

    // sort on the server
    node_info.sort_by(|a, b| a.name.cmp(&b.name));

    Json(node_info)
}

/// Return all asset nodes
async fn get_assets() -> Json<Vec<Node>> {
    let val = json! {[
        { "type": "asset", "name": "table1", "platforms": ["snowflake"] },
    ]};

    let mut node_info: Vec<Node> = serde_json::from_value(val).unwrap();

    // sort on the server
    node_info.sort_by(|a, b| a.name.cmp(&b.name));

    Json(node_info)
}

/// Return all group nodes
async fn get_groups() -> Json<Vec<Node>> {
    let val = json! {[
        { "type": "group", "name": "Group 1", "platforms": ["tableau"] },
    ]};

    let mut node_info: Vec<Node> = serde_json::from_value(val).unwrap();

    // sort on the server
    node_info.sort_by(|a, b| a.name.cmp(&b.name));

    Json(node_info)
}

/// Return all tag nodes
async fn get_tags() -> Json<Vec<Node>> {
    let val = json! {[
        { "type": "tag", "name": "my_tag", "platforms": ["jetty"] }
    ]};

    let mut node_info: Vec<Node> = serde_json::from_value(val).unwrap();

    // sort on the server
    node_info.sort_by(|a, b| a.name.cmp(&b.name));

    Json(node_info)
}
