use axum::{routing::get, Json, Router};
use serde_json::{json, Value};

/// Return a router to handle all tag-related requests
pub(super) fn router() -> Router {
    Router::new()
        .route("/:node_id/all_assets", get(all_assets_handler))
        .route("/:node_id/direct_assets", get(direct_assets_handler))
        .route("/:node_id/users", get(users_handler))
}

/// Return all assets tagged with a tag (directly or through inheritance)
async fn all_assets_handler() -> Json<Value> {
    Json(json! {
    [
        {
          "name": "Frozen Yogurt",
          "platform": "snowflake",
          "tag_paths": ["asset 1 > asset 2 > asset name",
          ]
        },
        {
          "name": "Ice cream sandwich",
          "platform": "Tableau",
          "tag_paths": ["asset 1 > asset 2 > asset name",
          "asset 3 > asset 4 > asset name"]
        },
      ]
        })
}

/// Return all assets directly tagged with a tag
async fn direct_assets_handler() -> Json<Value> {
    Json(json! {
    [
        {
          "name": "Frozen Yogurt",
          "platform": "snowflake",
        },
        {
          "name": "Ice cream sandwich",
          "platform": "Tableau",
        },
      ]
        })
}

/// Return all users with access to assets tagged with a tag
async fn users_handler() -> Json<Value> {
    Json(json! {
    [
        {
          "name": "Isaac",
          "platforms": ["snowflake", "tableau"],
        },
        {
          "name": "Ice cream sandwich yum",
          "platforms": ["snowflake", "tableau"],
        },
      ]
        })
}
