use std::sync::Arc;

use axum::{extract::Path, routing::get, Extension, Json, Router};
use jetty_core::{
    access_graph::{self, NodeName},
    connectors::UserIdentifier,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

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

/// Struct used to return asset access information
#[derive(Serialize, Deserialize)]
struct UserAssetsResponse {
    name: String,
    privileges: Vec<PrivilegeResponse>,
    connector: String,
}

#[derive(Serialize, Deserialize)]
struct PrivilegeResponse {
    name: String,
    explanations: Vec<String>,
}

/// Return information about a user's access to assets, including privilege and explanation
async fn assets_handler(
    Path(node_id): Path<String>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<UserAssetsResponse>> {
    // use the effective permissions to get all the assets that a user has access to
    let assets_and_permissions = ag.get_user_accessible_assets(&UserIdentifier::Email(node_id));
    // get the name and connectors from each asset

    Json(
        assets_and_permissions
            .iter()
            // get the JettyNodes for all of the accessible assets
            .map(|(k, v)| (ag.get_node(&NodeName::Asset(k.to_string())).unwrap(), v))
            // adding second map for clarity
            // build Vec of UserAssetResponse structs
            .map(|(k, v)| UserAssetsResponse {
                name: k.get_string_name(),
                privileges: v
                    .iter()
                    .map(|p| PrivilegeResponse {
                        name: p.privilege.to_owned(),
                        explanations: p.reasons.to_owned(),
                    })
                    .collect(),
                connector: k.get_node_connectors().iter().next().unwrap().to_owned(),
            })
            .collect(),
    )
}

/// Return information about a users access to tagged assets, grouped by tag
async fn tags_handler() -> Json<Value> {
    Json(json! {
                [
      {
        "name": "Frozen Yogurt",
        "assets": [
          { "name": "asset 1 with a much longer name", "platform": "tableau" },
          { "name": "asset 2", "platform": "tableau" },
          { "name": "asset 3", "platform": "tableau" },
          { "name": "asset 4", "platform": "tableau" },
          { "name": "asset 5", "platform": "tableau" },
        ],
      },
      {
        "name": "Ice cream sandwich",
        "assets": [
          { "name": "asset 1", "platform": "tableau" },
          { "name": "asset 2", "platform": "tableau" },
          { "name": "asset 3", "platform": "tableau" },
          { "name": "asset 4", "platform": "tableau" },
          { "name": "asset 5", "platform": "tableau" },
        ],
      },
    ]
            })
}

/// Returns groups that user is a direct member of
async fn direct_groups_handler() -> Json<Value> {
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

/// Returns groups that user is an inherited member of
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
          "platforms": ["Tableau"],
          "membership_paths": ["group 1 > group 2 > group name",
          "group 3 > group 4 > group name"]
        },
      ]
        })
}
