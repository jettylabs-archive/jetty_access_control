//! managing the write path for policies

use std::fmt::Display;

use jetty_core::{access_graph::translate::diffs::policies, write::assets};

use crate::SnowflakeConnector;

use super::PrioritizedQueries;

// fn prepare_queries(policy_diffs: &Vec<policies::LocalDiff>) -> PrioritizedQueries {
//     let mut res = PrioritizedQueries::default();

//     for policy in policy_diffs {
//         for user in policy.users {
//             u
//         }
//     }

//     user_diffs.iter().for_each(|diff| {
//         res.2.extend(
//             diff.group_membership
//                 .add
//                 .iter()
//                 .map(|g| format!("GRANT ROLE \"{}\" TO USER \"{}\";", g, &diff.user)),
//         );
//         res.2.extend(
//             diff.group_membership
//                 .remove
//                 .iter()
//                 .map(|g| format!("REVOKE ROLE \"{}\" FROM USER \"{}\";", g, &diff.user)),
//         );
//     });
//     res
// }

enum AgentType {
    User,
    Group,
}

impl Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentType::User => write!(f, "USER"),
            AgentType::Group => write!(f, "GROUP"),
        }
    }
}

fn generate_query_for_diff_details(
    details: assets::diff::policies::DiffDetails,
    asset: &String,
    agent_type: AgentType,
    agent: &String,
) -> String {
    match details {
        assets::diff::policies::DiffDetails::AddAgent { add } => {
            let privileges = add
                .privileges
                .to_owned()
                .into_iter()
                .collect::<Vec<_>>()
                .join(", ");
            format!("GRANT {privileges} ON {asset} TO {agent_type} {agent}")
        }
        assets::diff::policies::DiffDetails::RemoveAgent => {
            format!("REVOKE ALL ON {asset} TO {agent_type} {agent}")
        }
        assets::diff::policies::DiffDetails::ModifyAgent { add, remove } => todo!(),
    }
}
