use std::{collections::HashSet, sync::Arc};

use axum::{extract::Path, routing::get, Extension, Json, Router};
use jetty_core::{
    access_graph::{self, graph::typed_indices::AssetIndex},
    permissions::matrix::Merge,
};

use uuid::Uuid;

use crate::{node_summaries::NodeSummary, NodeSummaryWithPaths};

/// Return a router to handle all tag-related requests
pub(super) fn router() -> Router {
    Router::new()
        .route("/:node_id/all_assets", get(all_assets_handler))
        .route("/:node_id/direct_assets", get(direct_assets_handler))
        .route("/:node_id/users", get(users_handler))
}

/// Return all assets tagged with a tag (directly or through inheritance)
async fn all_assets_handler(
    Path(node_id): Path<Uuid>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<NodeSummaryWithPaths>> {
    let from = ag.get_tag_index_from_id(&node_id).unwrap();

    let asset_paths = ag.asset_paths_for_tag(from);
    let mut combined_asset_paths = asset_paths.directly_tagged;
    combined_asset_paths
        .merge(asset_paths.via_hierarchy)
        .unwrap();
    combined_asset_paths.merge(asset_paths.via_lineage).unwrap();

    Json(
        combined_asset_paths
            .iter()
            .map(|(&k, v)| NodeSummaryWithPaths {
                node: ag[k].to_owned().into(),
                paths: v
                    .iter()
                    .map(|p| {
                        ag.path_as_jetty_nodes(p)
                            .iter()
                            .map(|&n| n.to_owned().into())
                            .collect()
                    })
                    .collect(),
            })
            .collect(),
    )
}

/// Return all assets directly tagged with a tag
async fn direct_assets_handler(
    Path(node_id): Path<Uuid>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<NodeSummary>> {
    let from = ag.get_tag_index_from_id(&node_id).unwrap();

    let asset_paths = ag.asset_paths_for_tag(from);

    Json(
        asset_paths
            .directly_tagged
            .iter()
            .map(|(&k, _)| ag[k].to_owned().into())
            .collect(),
    )
}

/// Return all users with access to assets tagged with a tag
async fn users_handler(
    Path(node_id): Path<Uuid>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<NodeSummary>> {
    let from = ag.get_tag_index_from_id(&node_id).unwrap();

    let asset_paths = ag.asset_paths_for_tag(from);

    // get all the tagged assets
    let mut combined_assets: HashSet<&access_graph::NodeIndex> =
        HashSet::from_iter(asset_paths.directly_tagged.keys());
    combined_assets.extend(asset_paths.via_hierarchy.keys());
    combined_assets.extend(asset_paths.via_lineage.keys());

    // now return all the users that can access those assets
    Json(
        combined_assets
            .into_iter()
            .flat_map(|&a| {
                ag.get_users_with_access_to_asset(AssetIndex::new(a))
                    .keys()
                    .map(|k| k.to_owned())
                    .collect::<HashSet<_>>()
            })
            .collect::<HashSet<_>>()
            .into_iter()
            .map(|k| ag[k].to_owned().into())
            .collect::<Vec<NodeSummary>>(),
    )
}
