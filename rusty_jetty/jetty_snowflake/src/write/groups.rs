//! managing the write path for groups

use jetty_core::access_graph::translate::diffs::groups;

use crate::SnowflakeConnector;

use super::PrioritizedQueries;

pub(super) fn prepare_queries(
    group_diffs: &[groups::LocalDiff],
    snow: &SnowflakeConnector,
) -> PrioritizedQueries {
    let mut res = PrioritizedQueries::default();
    group_diffs.iter().for_each(|diff| {
        match &diff.details {
            groups::LocalDiffDetails::AddGroup { member_of } => {
                res.1.push(format!("CREATE ROLE \"{}\";", diff.group_name));
                for group in member_of {
                    res.2.push(format!(
                        "GRANT ROLE \"{}\" TO ROLE \"{}\";",
                        group, diff.group_name
                    ))
                }
            }
            groups::LocalDiffDetails::RemoveGroup => {
                // Drop roles. This will transfer all ownership to the Jetty role. If there are grants that are owned by the role that is dropped, those grants are dropped too.
                // because of this, it may be necessary to run a double-apply.
                res.0.push(format!(
                    "GRANT OWNERSHIP ON ROLE \"{}\" TO \"{}\"; --Only the owner of a role can drop it",
                    diff.group_name,
                    snow.rest_client.get_snowflake_role()
                ));
                res.1.push(format!("DROP ROLE \"{}\";", diff.group_name));
            }
            groups::LocalDiffDetails::ModifyGroup {
                add_member_of,
                remove_member_of,
            } => {
                for group in add_member_of {
                    res.2.push(format!(
                        "GRANT ROLE \"{}\" TO ROLE \"{}\";",
                        group, diff.group_name
                    ))
                }
                for group in remove_member_of {
                    res.2.push(format!(
                        "REVOKE ROLE \"{}\" FROM ROLE \"{}\";",
                        group, diff.group_name
                    ))
                }
            }
        }
    });
    res
}
