//! Write path for Snowflake connector

mod groups;
mod users;
mod policies;

// need a snowflake coordinator. The diff will be used to update the environment. Then build the role grants. Then build effective permissions.

// Also need a function to take grants and generate the queries

// Then need to run the queries

use jetty_core::access_graph::translate::diffs::LocalConnectorDiffs;

use crate::SnowflakeConnector;

#[derive(Default)]
struct PrioritizedQueries(Vec<String>, Vec<String>, Vec<String>);

impl SnowflakeConnector {
    pub(super) fn generate_diff_queries(&self, diffs: &LocalConnectorDiffs) -> Vec<Vec<String>> {
        todo!()
        // let mut first_queries: Vec<String> = vec![];
        // let mut second_queries: Vec<String> = vec![];
        // let mut third_queries: Vec<String> = vec![];
        // // start with groups
        // for diff in &diffs.groups {
        //     match &diff.details {
        //         // Drop roles. This will transfer all ownership to the Jetty role. If there are grants that are owned by the role that is dropped, those grants are dropped too.
        //         // because of this, it may be necessary to run a double-apply.
        //         groups::LocalDiffDetails::RemoveGroup => {
        //             first_queries.push(format!("GRANT OWNERSHIP ON ROLE \"{}\" TO {}; --Only the owner of a role can drop it", diff.group_name, self.rest_client.get_snowflake_role()));
        //             second_queries.push(format!("DROP ROLE \"{}\";", diff.group_name))
        //         }
        //         groups::LocalDiffDetails::AddGroup { members } => {
        //             second_queries.push(format!("CREATE ROLE \"{}\";", diff.group_name));
        //             for user in &members.users {
        //                 third_queries.push(format!(
        //                     "GRANT ROLE \"{}\" TO USER \"{}\";",
        //                     diff.group_name, user
        //                 ))
        //             }
        //             for group in &members.groups {
        //                 third_queries.push(format!(
        //                     "GRANT ROLE \"{}\" TO ROLE \"{}\";",
        //                     diff.group_name, group
        //                 ))
        //             }
        //         }
        //         groups::LocalDiffDetails::ModifyGroup { add, remove } => {
        //             for user in &add.users {
        //                 third_queries.push(format!(
        //                     "GRANT ROLE \"{}\" TO USER \"{}\";",
        //                     diff.group_name, user
        //                 ))
        //             }
        //             for group in &add.groups {
        //                 third_queries.push(format!(
        //                     "GRANT ROLE \"{}\" TO ROLE \"{}\";",
        //                     diff.group_name, group
        //                 ))
        //             }
        //             for user in &remove.users {
        //                 third_queries.push(format!(
        //                     "REVOKE ROLE \"{}\" FROM USER \"{}\";",
        //                     diff.group_name, user
        //                 ))
        //             }
        //             for group in &remove.groups {
        //                 third_queries.push(format!(
        //                     "REVOKE ROLE {} FROM ROLE {};",
        //                     diff.group_name, group
        //                 ))
        //             }
        //         }
        //     }
        // }
        // vec![first_queries, second_queries, third_queries]
    }
}
