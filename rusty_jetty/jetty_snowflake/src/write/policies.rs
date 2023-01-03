//! managing the write path for policies

use std::fmt::Display;

use jetty_core::{access_graph::translate::diffs::policies, write::assets};

use crate::SnowflakeAsset;

use super::PrioritizedQueries;

pub(super) fn prepare_queries(policy_diffs: &Vec<policies::LocalDiff>) -> PrioritizedQueries {
    let mut res = PrioritizedQueries::default();

    for policy in policy_diffs {
        let asset = crate::cual::cual_to_snowflake_asset(&policy.asset);
        for (user, details) in &policy.users {
            res.2.extend(generate_queries_for_diff_details(
                details,
                &asset,
                AgentType::User,
                user,
            ))
        }

        for (group, details) in &policy.groups {
            res.2.extend(generate_queries_for_diff_details(
                details,
                &asset,
                AgentType::Group,
                group,
            ))
        }
    }

    res
}

pub(crate) enum AgentType {
    User,
    Group,
}

impl Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentType::User => write!(f, "USER"),
            AgentType::Group => write!(f, "ROLE"),
        }
    }
}

pub(crate) fn generate_queries_for_diff_details(
    details: &assets::diff::policies::DiffDetails,
    asset: &SnowflakeAsset,
    agent_type: AgentType,
    agent: &String,
) -> Vec<String> {
    match details {
        assets::diff::policies::DiffDetails::AddAgent { add } => {
            let add = &mut add.to_owned();
            let mut res = Vec::new();

            // FUTURE: How we handle ownership today may make double-applies necessary as it
            // also affects other grants
            if add.privileges.remove("OWNERSHIP") {
                res.push(format!(
                    "GRANT OWNERSHIP ON {} {} TO {agent_type} \"{agent}\" COPY CURRENT GRANTS",
                    asset.asset_type(),
                    asset.fqn()
                ))
            }
            let privileges = add
                .privileges
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            res.push(format!(
                "GRANT {privileges} ON {} {} TO {agent_type} \"{agent}\"",
                asset.asset_type(),
                asset.fqn()
            ));
            res
        }
        assets::diff::policies::DiffDetails::RemoveAgent { .. } => {
            vec![format!(
                "REVOKE ALL ON {} {} FROM {agent_type} \"{agent}\"",
                asset.asset_type(),
                asset.fqn()
            )]
        }
        assets::diff::policies::DiffDetails::ModifyAgent { add, remove } => {
            let add = &mut add.to_owned();
            let mut res = Vec::new();
            if !add.privileges.is_empty() {
                if add.privileges.remove("OWNERSHIP") {
                    res.push(format!(
                        "GRANT OWNERSHIP ON {} {} TO {agent_type} \"{agent}\" COPY CURRENT GRANTS",
                        asset.asset_type(),
                        asset.fqn()
                    ))
                }
                let privileges = add
                    .privileges
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ");
                res.push(format!(
                    "GRANT {privileges} ON {} {} TO {agent_type} \"{agent}\"",
                    asset.asset_type(),
                    asset.fqn()
                ));
            }
            if !remove.privileges.is_empty() {
                let privileges = remove
                    .privileges
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ");
                res.push(format!(
                    "REVOKE {privileges} ON {} {} FROM {agent_type} \"{agent}\"",
                    asset.asset_type(),
                    asset.fqn()
                ));
            }
            res
        }
    }
}
