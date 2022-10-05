use axum::{routing::get, Json, Router};
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
struct UserAssets {
    name: String,
    privileges: Vec<Privilege>,
    platform: String,
}

#[derive(Serialize, Deserialize)]
struct Privilege {
    name: String,
    explanations: Vec<String>,
}

/// Return information about a user's access to assets, including privilege and explanation
async fn assets_handler() -> Json<Vec<UserAssets>> {
    let val = json!(
    [
        {
            "name": "Frozen Yogurt",
            "privileges": [
            {
                "name": "p1",
                "explanations": [
                "what happens we have really long explanations what happens we have really long explanations",
                "what happens we have really long",
                ],
            },
            { "name": "p2", "explanations": ["reason 1", "reason 2"] },
            { "name": "p3", "explanations": ["reason 1", "reason 2"] },
            ],
            "platform": "tableau",
        },
        {
            "name": "Ice cream sandwich",
            "privileges": [{ "name": "p1", "explanations": ["reason 1", "reason 2"] }],
            "platform": "snowflake",
        },
    ]
    );

    let user_asset_info: Vec<UserAssets> = serde_json::from_value(val).unwrap();

    Json(user_asset_info)
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
