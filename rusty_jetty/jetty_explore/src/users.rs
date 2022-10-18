use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use anyhow::Context;
use axum::{extract::Path, routing::get, Extension, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::ObjectWithPathResponse;
use jetty_core::{
    access_graph::{self, EdgeType, JettyNode, NodeIndex, NodeName},
    connectors::UserIdentifier,
    cual::Cual,
    logging::info,
};

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
            .map(|(k, v)| {
                (
                    ag.get_node(&NodeName::Asset(k.to_string()))
                        .context("fetching asset index from node map")
                        .unwrap(),
                    v,
                )
            })
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
                connector: k
                    .get_node_connectors()
                    .iter()
                    .next()
                    .context(format!("asset is missing a connector: {:?}", k))
                    .unwrap()
                    .to_owned(),
            })
            .collect(),
    )
}

#[derive(Serialize)]
pub(crate) struct TagWithAssets {
    name: String,
    assets: Vec<AssetBasics>,
}

#[derive(Serialize)]
pub(crate) struct AssetBasics {
    name: String,
    connectors: HashSet<String>,
}

/// Return information about a users access to tagged assets, grouped by tag
async fn tags_handler(
    Path(node_id): Path<String>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<TagWithAssets>> {
    // get all the user_accessable assets
    let accessable_assets = ag.get_user_accessible_assets(&UserIdentifier::Email(node_id));
    let tag_asset_map = accessable_assets
        .iter()
        .map(|(c, _)| (c, ag.tags_for_asset(&NodeName::Asset(c.to_string()))))
        .map(|(c, i)| i.iter().map(|n| (n.clone(), c)).collect::<Vec<_>>())
        .flatten()
        .fold(
            HashMap::<NodeIndex, Vec<JettyNode>>::new(),
            |mut acc, (tag_node, asset_cual)| {
                acc.entry(tag_node)
                    .and_modify(|e| {
                        e.push(
                            ag.get_node(&NodeName::Asset(asset_cual.to_string()))
                                .context("nonexistent asset")
                                .unwrap()
                                .to_owned(),
                        );
                    })
                    .or_insert(vec![ag
                        .get_node(&NodeName::Asset(asset_cual.to_string()))
                        .context("nonexistent asset")
                        .unwrap()
                        .to_owned()]);
                acc
            },
        );

    let response = tag_asset_map
        .into_iter()
        .map(|(t, v)| (&ag[t], v))
        .map(|(t, v)| TagWithAssets {
            name: t.get_string_name(),
            assets: v
                .iter()
                .map(|v| AssetBasics {
                    name: v.get_string_name(),
                    connectors: v.get_node_connectors(),
                })
                .collect(),
        })
        .collect::<Vec<_>>();

    Json(response)
}

/// Returns groups that user is a direct member of
async fn direct_groups_handler(
    Path(node_id): Path<String>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<access_graph::GroupAttributes>> {
    let from = NodeName::User(node_id);

    let group_nodes = ag.get_matching_children(
        &from,
        |n| matches!(n, EdgeType::MemberOf),
        |n| matches!(n, JettyNode::Group(_)),
        |n| matches!(n, JettyNode::Group(_)),
        None,
        Some(1),
    );

    let group_attributes = group_nodes
        .into_iter()
        .filter_map(|i| {
            if let JettyNode::Group(g) = &ag.graph()[i] {
                Some(g.to_owned())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    Json(group_attributes)
}

/// Returns groups that user is an inherited member of
async fn inherited_groups_handler(
    Path(node_id): Path<String>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<ObjectWithPathResponse>> {
    let from = NodeName::User(node_id);

    let res = ag.all_matching_simple_paths_to_children(
        &from,
        |n| matches!(n, EdgeType::MemberOf),
        |n| matches!(n, JettyNode::Group(_)),
        |n| matches!(n, JettyNode::Group(_)),
        None,
        None,
    );

    let group_attributes = res
        .into_iter()
        .filter_map(|(i, p)| {
            if let JettyNode::Group(g) = &ag.graph()[i] {
                Some(ObjectWithPathResponse {
                    name: g.name.to_owned(),
                    connectors: g.connectors.to_owned(),
                    membership_paths: p.iter().map(|p| ag.path_as_string(p)).collect(),
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    Json(group_attributes)
}
