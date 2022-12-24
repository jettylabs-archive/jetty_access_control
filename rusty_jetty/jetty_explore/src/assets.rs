use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use anyhow::Context;
use axum::{extract::Path, routing::get, Extension, Json, Router};
use jetty_core::{
    access_graph::{
        self,
        graph::typed_indices::{AssetIndex, UserIndex},
        EdgeType, JettyNode,
    },
    connectors::nodes::PermissionMode,
};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    node_summaries::NodeSummary, NodeSummaryWithPaths, NodeSummaryWithPrivileges,
    SummaryWithAssociatedSummaries,
};

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
struct AssetTagSummaries {
    direct: Vec<NodeSummary>,
    via_lineage: Vec<NodeSummary>,
    via_hierarchy: Vec<NodeSummary>,
}

/// Return information about upstream assets, by hierarchy. Includes path to the current asset
async fn hierarchy_upstream_handler(
    // node_id is the cual for an asset
    Path(node_id): Path<Uuid>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<NodeSummaryWithPaths>> {
    Json(asset_genealogy_with_path(node_id, ag, |e| {
        matches!(e, EdgeType::ChildOf)
    }))
}

/// Return information about downstream assets, by hierarchy. Includes path to the current asset
async fn hierarchy_downstream_handler(
    // node_id is the cual for an asset
    Path(node_id): Path<Uuid>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<NodeSummaryWithPaths>> {
    Json(asset_genealogy_with_path(node_id, ag, |e| {
        matches!(e, EdgeType::ParentOf)
    }))
}

/// Return information about upstream assets, by data lineage. Includes path to the current asset
async fn lineage_upstream_handler(
    // node_id is the cual for an asset
    Path(node_id): Path<Uuid>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<NodeSummaryWithPaths>> {
    Json(asset_genealogy_with_path(node_id, ag, |e| {
        matches!(e, EdgeType::DerivedFrom)
    }))
}

/// Return information about downstream assets, by data lineage. Includes path to the current asset
async fn lineage_downstream_handler(
    // node_id is the cual for an asset
    Path(node_id): Path<Uuid>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<NodeSummaryWithPaths>> {
    Json(asset_genealogy_with_path(node_id, ag, |e| {
        matches!(e, EdgeType::DerivedTo)
    }))
}

/// Return information about the tags that an asset is tagged with
async fn tags_handler(
    // node_id is the cual for an asset
    Path(node_id): Path<Uuid>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<AssetTagSummaries> {
    // convert the node_id to an AssetIndex
    let asset_index = ag.get_asset_index_from_id(&node_id).unwrap();

    let tags = ag.tags_for_asset_by_source(asset_index);

    Json(AssetTagSummaries {
        direct: tags
            .direct
            .into_iter()
            .map(|t| ag[t].to_owned().into())
            .collect(),
        via_lineage: tags
            .via_lineage
            .into_iter()
            .map(|t| ag[t].to_owned().into())
            .collect(),
        via_hierarchy: tags
            .via_hierarchy
            .into_iter()
            .map(|t| ag[t].to_owned().into())
            .collect(),
    })
}

/// Return users that have direct access to the asset, including their levels of privilege and privilege explanation
async fn direct_users_handler(
    // node_id is the cual for an asset
    Path(node_id): Path<Uuid>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<NodeSummaryWithPrivileges>> {
    // convert the node_id to an AssetIndex
    let asset_index = ag.get_asset_index_from_id(&node_id).unwrap();

    let users = ag.get_users_with_access_to_asset(asset_index);

    Json(
        users
            .iter()
            .map(|(u, ps)| NodeSummaryWithPrivileges {
                node: ag[*u].to_owned().into(),
                privileges: ps.iter().map(|p| (*p).to_owned()).collect(),
            })
            .collect(),
    )
}

/// Return users that have access to this asset directly, or through downstream assets (via data lineage)
async fn users_incl_downstream_handler(
    // node_id is the cual for an asset
    Path(node_id): Path<Uuid>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<SummaryWithAssociatedSummaries>> {
    // get all assets that that reference the given asset
    let asset_index = ag
        .get_asset_index_from_id(&node_id)
        .context("getting asset node index")
        .unwrap();

    let mut asset_list = ag.get_matching_descendants(
        asset_index,
        |e| matches!(e, EdgeType::DerivedTo),
        |n| matches!(n, JettyNode::Asset(_)),
        |n| matches!(n, JettyNode::Asset(_)),
        None,
        None,
    );

    asset_list.push(asset_index.into());

    let user_asset_map = asset_list
        .into_iter()
        .flat_map(|a| {
            ag.get_users_with_access_to_asset(AssetIndex::new(a))
                .iter()
                // If they don't have an allow privilege, it shouldn't count as access
                .filter_map(|(u, ep)| {
                    if ep.iter().any(|ep| ep.mode == PermissionMode::Allow) {
                        Some((u.to_owned(), AssetIndex::new(a)))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        })
        .fold(
            HashMap::<UserIndex, HashSet<AssetIndex>>::new(),
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
            .map(|(u, assets)| SummaryWithAssociatedSummaries {
                node: ag[u].to_owned().into(),
                associations: assets
                    .iter()
                    .map(|a| NodeSummary::from(ag[*a].to_owned()))
                    .collect(),
            })
            .collect::<Vec<_>>(),
    )
}

/// get the ascending or descending assets with paths, based on edge matcher
fn asset_genealogy_with_path(
    // node_id is the cual for an asset
    node_id: Uuid,
    ag: Arc<access_graph::AccessGraph>,
    edge_matcher: fn(&EdgeType) -> bool,
) -> Vec<NodeSummaryWithPaths> {
    let asset_index = ag
        .get_asset_index_from_id(&node_id)
        .context("getting asset node index")
        .unwrap();

    let paths = ag.all_matching_simple_paths_to_descendants(
        asset_index,
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
            NodeSummaryWithPaths {
                node: node.to_owned().into(),
                paths: v
                    .iter()
                    .map(|q| {
                        ag.path_as_jetty_nodes(q)
                            .iter()
                            .map(|v| NodeSummary::from((*v).to_owned()))
                            .collect()
                    })
                    .collect(),
            }
        })
        .collect::<Vec<_>>()
}
