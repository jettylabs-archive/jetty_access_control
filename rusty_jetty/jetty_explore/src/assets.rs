use std::sync::Arc;

use axum::{extract::Path, routing::get, Extension, Json, Router};
use jetty_core::access_graph::{self, explore2::AssetTags, EdgeType, JettyNode, NodeName};
use serde::Serialize;
use serde_json::{json, Value};

/// Return a router to handle all asset-related requests
pub(super) fn router() -> Router {
    Router::new()
        .route(
            "/:node_id/hierarchy_upstream",
            get(hierarchy_upstream_handler),
        )
        .route(
            "/:node_id/hierarchy_downstream",
            get(hierarchy_downstream_handler),
        )
        .route("/:node_id/lineage_upstream", get(lineage_upstream_handler))
        .route(
            "/:node_id/lineage_downstream",
            get(lineage_downstream_handler),
        )
        .route("/:node_id/users", get(direct_users_handler))
        .route("/:node_id/all_users", get(users_incl_downstream_handler))
        .route("/:node_id/tags", get(tags_handler))
}

#[derive(Serialize)]
struct AssetWithPaths {
    name: String,
    connector: String,
    paths: Vec<String>,
}

/// Return information about upstream assets, by hierarchy. Includes path to the current asset
async fn hierarchy_upstream_handler(
    Path(node_id): Path<String>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<AssetWithPaths>> {
    asset_genealogy_with_path(node_id, ag, |e| matches!(e, EdgeType::ChildOf))
}

/// Return information about downstream assets, by hierarchy. Includes path to the current asset
async fn hierarchy_downstream_handler(
    Path(node_id): Path<String>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<AssetWithPaths>> {
    asset_genealogy_with_path(node_id, ag, |e| matches!(e, EdgeType::ParentOf))
}

/// Return information about upstream assets, by data lineage. Includes path to the current asset
async fn lineage_upstream_handler(
    Path(node_id): Path<String>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<AssetWithPaths>> {
    asset_genealogy_with_path(node_id, ag, |e| matches!(e, EdgeType::DerivedFrom))
}

/// Return information about downstream assets, by data lineage. Includes path to the current asset
async fn lineage_downstream_handler(
    Path(node_id): Path<String>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<AssetWithPaths>> {
    asset_genealogy_with_path(node_id, ag, |e| matches!(e, EdgeType::DerivedTo))
}

/// Return information about the tags that an asset is tagged with
async fn tags_handler(
    Path(node_id): Path<String>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<AssetTags> {
    Json(ag.tags_for_asset_by_source(&NodeName::Asset(node_id)))
}

/// Return users that have direct access to the asset, including there level of privilege and privilege explanation
async fn direct_users_handler() -> Json<Value> {
    Json(json!(
    [
        {
            "name": "Isaac",
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
            "platforms": ["tableau", "snowflake"],
        },
        {
            "name": "Ice cream sandwich",
            "privileges": [{ "name": "p1", "explanations": ["reason 1", "reason 2"] }],
            "platforms": ["snowflake"],
        },
    ]
    ))
}

/// Return users that have access to this asset directly, or through downstream assets (via data lineage)
async fn users_incl_downstream_handler() -> Json<Value> {
    Json(json!(
    [
        {
            "name": "Isaac",
            "platforms": ["tableau", "snowflake"],
            "assets": ["downstream asset 1", "this asset"]
        },
        {
            "name": "Ice cream sandwich",
            "platforms": ["snowflake"],
            "assets": ["downstream asset 2", "downstream asset 3"]
        },
    ]
    ))
}

/// get the ascending or descending assets with paths, based on edge matcher
fn asset_genealogy_with_path(
    node_id: String,
    ag: Arc<access_graph::AccessGraph>,
    edge_matcher: fn(&EdgeType) -> bool,
) -> Json<Vec<AssetWithPaths>> {
    let paths = ag.all_matching_simple_paths_to_children(
        &NodeName::Asset(node_id),
        edge_matcher,
        |n| matches!(n, JettyNode::Asset(_)),
        |n| matches!(n, JettyNode::Asset(_)),
        None,
        None,
    );

    Json(
        paths
            .into_iter()
            .map(|(k, v)| {
                let node = &ag[k];
                AssetWithPaths {
                    name: node.get_string_name(),
                    connector: node
                        .get_node_connectors()
                        .iter()
                        .next()
                        .and_then(|s| Some(s.to_owned()))
                        .unwrap_or("unknown".to_owned()),
                    paths: v.iter().map(|p| ag.path_as_string(p)).collect::<Vec<_>>(),
                }
            })
            .collect::<Vec<_>>(),
    )
}
