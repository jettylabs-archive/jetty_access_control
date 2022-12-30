//! managing the write path for policies

use std::fmt::Display;

use jetty_core::{access_graph::translate::diffs::default_policies, write::assets};

use crate::{SnowflakeAsset, SnowflakeConnector};

use super::{policies::AgentType, PrioritizedQueries};

pub(super) fn prepare_queries(
    policy_diffs: &Vec<default_policies::LocalDiff>,
) -> PrioritizedQueries {
    let mut res = PrioritizedQueries::default();

    for policy in policy_diffs {
        let asset = crate::cual::cual_to_snowflake_asset(&policy.asset);
        for (user, details) in &policy.users {
            res.2.extend(generate_queries_for_diff_details(
                details,
                &asset,
                &policy.asset_type,
                AgentType::User,
                user,
            ))
        }

        for (group, details) in &policy.groups {
            res.2.extend(generate_queries_for_diff_details(
                details,
                &asset,
                &policy.asset_type,
                AgentType::Group,
                group,
            ))
        }
    }

    res
}

fn generate_queries_for_diff_details(
    details: &assets::diff::policies::DiffDetails,
    asset: &SnowflakeAsset,
    asset_type: &String,
    agent_type: AgentType,
    agent: &String,
) -> Vec<String> {
    match details {
        assets::diff::policies::DiffDetails::AddAgent { add } => {
            let privileges = add
                .privileges
                .to_owned()
                .into_iter()
                .collect::<Vec<_>>()
                .join(", ");
            vec![format!(
                "GRANT {privileges} ON FUTURE {asset_type}S IN {} {} TO {agent_type} {agent}",
                asset.asset_type(),
                asset.fqn()
            )]
        }
        assets::diff::policies::DiffDetails::RemoveAgent { .. } => {
            vec![format!(
                "REVOKE ALL ON FUTURE {asset_type}S IN {} {} FROM {agent_type} {agent}",
                asset.asset_type(),
                asset.fqn()
            )]
        }
        assets::diff::policies::DiffDetails::ModifyAgent { add, remove } => {
            let privileges = add
                .privileges
                .to_owned()
                .into_iter()
                .collect::<Vec<_>>()
                .join(", ");
            let mut res = vec![format!(
                "GRANT {privileges} ON FUTURE {asset_type}S IN {} {} TO {agent_type} {agent}",
                asset.asset_type(),
                asset.fqn()
            )];
            let privileges = remove
                .privileges
                .to_owned()
                .into_iter()
                .collect::<Vec<_>>()
                .join(", ");
            res.push(format!(
                "REVOKE {privileges} ON FUTURE {asset_type}S IN {} {} FROM {agent_type} {agent}",
                asset.asset_type(),
                asset.fqn()
            ));
            res
        }
    }
}
