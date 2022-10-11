use axum::{extract::Path, routing::get, Json, Router};
use serde_json::{json, Value};

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
async fn direct_groups_handler(Path(node_id): Path<String>) -> Json<Value> {

  
    Json(json! {
    [
        {
          "name": "Frozen Yogurt",
          "platforms": ["Tableau", "snowflake"],
        },
        {
          "name": "Ice cream sandwich",
          "platforms": ["Tableau"],
        },
      ]
        })
}

/// Return the groups that this group is an inherited member of
async fn inherited_groups_handler() -> Json<Value> {
    Json(json! {
    [
        {
          "name": "Frozen Yogurt",
          "platforms": ["snowflake"],
          "membership_paths": ["group 1 > group 2 > group name",
          ]
        },
        {
          "name": "Ice cream sandwich",
          "platforms": ["Tableau", "jetty"],
          "membership_paths": ["group 1 > group 2 > group name",
          "group 3 > group 4 > group name"]
        },
      ]
        })
}

/// Return the groups that are direct members of this group
async fn direct_members_groups_handler() -> Json<Value> {
    Json(json! {
    [
        {
          "name": "Frozen Yogurt",
          "platforms": ["snowflake"],
        },
        {
          "name": "Ice cream sandwich",
          "platforms": ["Tableau"],
        },
      ]
        })
}

/// Return the users that are direct members of this group
async fn direct_members_users_handler() -> Json<Value> {
    Json(json! {
    [
        {
          "name": "Frozen Yogurt",
          "platforms": ["snowflake"],
        },
        {
          "name": "Ice cream sandwich",
          "platforms": ["Tableau"],
        },
      ]
        })
}

/// Return all users that are members of the group, directly or through inheritance
async fn all_members_handler() -> Json<Value> {
    Json(json! {
    [
        {
          "name": "Frozen Yogurt",
          "platforms": ["snowflake", "tableau"],
          "membership_paths": ["group 1 > group 2 > group name",
          "Direct Access"]
        },
        {
          "name": "Ice cream sandwich yum",
          "platforms": ["snowflake", "tableau"],
          "membership_paths": ["group 1 > group 2 > group name",
          "Direct Access"]
        },
      ]
        })
}
