use std::{collections::HashSet, sync::Arc};

use axum::{extract::Path, routing::get, Extension, Json, Router};
use jetty_core::access_graph::{self, EdgeType, JettyNode, NodeName};
use serde::{Deserialize, Serialize};
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
async fn direct_groups_handler(
    Path(node_id): Path<String>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Vec<access_graph::GroupAttributes>> {
    let from = NodeName::Group(node_id);

    println!("{:?}", ag.extract_graph(&from, 1).dot());

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
        .filter_map(|n| {
            if let JettyNode::Group(g) = n {
                Some(g)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    Json(group_attributes)
}

/// Return the groups that this group is an inherited member of
async fn inherited_groups_handler(
    Path(node_id): Path<String>,
    Extension(ag): Extension<Arc<access_graph::AccessGraph>>,
) -> Json<Value> {
    #[derive(Serialize, Deserialize)]
    struct GroupWithPath {
        name: String,
        connectors: HashSet<String>,
        membership_paths: Vec<String>,
    }

    let from = NodeName::Group(node_id);

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
        .filter_map(|(n, p)| {
            if let JettyNode::Group(g) = n {
                Some(GroupWithPath {
                    name: g.name.to_owned(),
                    connectors: g.connectors,
                    membership_paths: p.iter().map(|p| p.to_string()).collect(),
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    Json(serde_json::to_value(&group_attributes).unwrap())
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
