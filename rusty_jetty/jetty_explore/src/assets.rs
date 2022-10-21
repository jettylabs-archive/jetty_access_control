use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use axum::{extract::Path, routing::get, Extension, Json, Router};
use jetty_core::{
    access_graph::{self, EdgeType, JettyNode, NodeName},
    cual::Cual,
};
use serde::Serialize;

use crate::{PrivilegeResponse, UserAssetsResponse};

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

#[derive(Serialize, Debug)]
struct AssetWithPaths {
    name: String,
    connector: String,
    paths: Vec<String>,
}

#[derive(Serialize, Debug)]
struct UserWithDownstreamAccess {
    name: String,
    connectors: HashSet<String>,
    assets: HashSet<String>,
}

#[derive(Serialize, Debug)]
struct AssetTagNames {
    direct: Vec<String>,
    via_lineage: Vec<String>,
    via_hierarchy: Vec<String>,
}

/// Return information about upstream assets, by hierarchy. Includes path to the current asset
async fn hierarchy_upstream_handler(
    // node_id is the cual for an asset
    Path(node_id): Path<String>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<AssetWithPaths>> {
    Json(asset_genealogy_with_path(node_id, ag, |e| {
        matches!(e, EdgeType::ChildOf)
    }))
}

/// Return information about downstream assets, by hierarchy. Includes path to the current asset
async fn hierarchy_downstream_handler(
    // node_id is the cual for an asset
    Path(node_id): Path<String>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<AssetWithPaths>> {
    Json(asset_genealogy_with_path(node_id, ag, |e| {
        matches!(e, EdgeType::ParentOf)
    }))
}

/// Return information about upstream assets, by data lineage. Includes path to the current asset
async fn lineage_upstream_handler(
    // node_id is the cual for an asset
    Path(node_id): Path<String>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<AssetWithPaths>> {
    Json(asset_genealogy_with_path(node_id, ag, |e| {
        matches!(e, EdgeType::DerivedFrom)
    }))
}

/// Return information about downstream assets, by data lineage. Includes path to the current asset
async fn lineage_downstream_handler(
    // node_id is the cual for an asset
    Path(node_id): Path<String>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<AssetWithPaths>> {
    Json(asset_genealogy_with_path(node_id, ag, |e| {
        matches!(e, EdgeType::DerivedTo)
    }))
}

/// Return information about the tags that an asset is tagged with
async fn tags_handler(
    // node_id is the cual for an asset
    Path(node_id): Path<String>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<AssetTagNames> {
    let tags = ag.tags_for_asset_by_source(&NodeName::Asset(node_id));

    Json(AssetTagNames {
        direct: tags
            .direct
            .into_iter()
            .map(|t| ag[t].get_string_name())
            .collect(),
        via_lineage: tags
            .via_lineage
            .into_iter()
            .map(|t| ag[t].get_string_name())
            .collect(),
        via_hierarchy: tags
            .via_hierarchy
            .into_iter()
            .map(|t| ag[t].get_string_name())
            .collect(),
    })
}

/// Return users that have direct access to the asset, including their levels of privilege and privilege explanation
async fn direct_users_handler(
    // node_id is the cual for an asset
    Path(node_id): Path<String>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<UserAssetsResponse>> {
    let users = ag.get_users_with_access_to_asset(Cual::new(&node_id));

    Json(
        users
            .iter()
            .map(|(u, ps)| {
                let user_name = u
                    .inner_value()
                    .map(|s| s.to_owned())
                    .unwrap_or_else(|| "".to_owned());
                UserAssetsResponse {
                    name: user_name.to_owned(),
                    privileges: ps
                        .iter()
                        .map(|p| PrivilegeResponse {
                            name: p.privilege.to_owned(),
                            explanations: p.reasons.to_owned(),
                        })
                        .collect(),
                    connectors: ag
                        .get_node(&NodeName::User(user_name))
                        .unwrap()
                        .get_node_connectors(),
                }
            })
            .collect(),
    )
}

/// Return users that have access to this asset directly, or through downstream assets (via data lineage)
async fn users_incl_downstream_handler(
    // node_id is the cual for an asset
    Path(node_id): Path<String>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<UserWithDownstreamAccess>> {
    let mut downstream_assets = asset_genealogy_with_path(node_id.to_owned(), ag.clone(), |e| {
        matches!(e, EdgeType::DerivedTo)
    })
    .iter()
    .map(|a| a.name.to_owned())
    .collect::<Vec<_>>();
    downstream_assets.push(node_id);

    let user_asset_map = downstream_assets
        .into_iter()
        .flat_map(|a| {
            ag.get_users_with_access_to_asset(Cual::new(&a))
                .keys()
                .map(|u| {
                    (
                        u.inner_value().map(|s| s.to_owned()).unwrap_or_default(),
                        a.to_owned(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .fold(
            HashMap::<String, HashSet<String>>::new(),
            |mut acc, (user, asset)| {
                acc.entry(user)
                    .and_modify(|a| {
                        a.insert(asset.to_owned());
                    })
                    .or_insert_with(|| HashSet::from([asset]));
                acc
            },
        );

    Json(
        user_asset_map
            .into_iter()
            .map(|(u, assets)| UserWithDownstreamAccess {
                name: u.to_owned(),
                connectors: ag
                    .get_node(&NodeName::User(u))
                    .unwrap()
                    .get_node_connectors(),
                assets,
            })
            .collect::<Vec<_>>(),
    )
}

/// get the ascending or descending assets with paths, based on edge matcher
fn asset_genealogy_with_path(
    // node_id is the cual for an asset
    node_id: String,
    ag: Arc<access_graph::AccessGraph>,
    edge_matcher: fn(&EdgeType) -> bool,
) -> Vec<AssetWithPaths> {
    let paths = ag.all_matching_simple_paths_to_children(
        &NodeName::Asset(node_id),
        edge_matcher,
        |n| matches!(n, JettyNode::Asset(_)),
        |n| matches!(n, JettyNode::Asset(_)),
        None,
        None,
    );

    paths
        .into_iter()
        .map(|(k, v)| {
            let node = &ag[k];
            AssetWithPaths {
                name: node.get_string_name(),
                connector: node
                    .get_node_connectors()
                    .iter()
                    // Asset should only have one connector. To be cleaned up in a future version.
                    .next()
                    .map(|s| s.to_owned())
                    .unwrap_or_else(|| "unknown".to_owned()),
                paths: v.iter().map(|p| ag.path_as_string(p)).collect::<Vec<_>>(),
            }
        })
        .collect::<Vec<_>>()
}
