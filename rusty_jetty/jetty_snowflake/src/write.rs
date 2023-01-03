//! Write path for Snowflake connector

mod default_policies;
mod groups;
mod policies;
mod users;

// need a snowflake coordinator. The diff will be used to update the environment. Then build the role grants. Then build effective permissions.

// Also need a function to take grants and generate the queries

// Then need to run the queries

use jetty_core::access_graph::translate::diffs::LocalConnectorDiffs;

use crate::SnowflakeConnector;

#[derive(Default)]
pub(crate) struct PrioritizedQueries(
    pub(crate) Vec<String>,
    pub(crate) Vec<String>,
    pub(crate) Vec<String>,
);

impl PrioritizedQueries {
    fn extend(&mut self, other: &PrioritizedQueries) {
        self.0.extend(other.0.clone());
        self.1.extend(other.1.clone());
        self.2.extend(other.2.clone());
    }
    pub(crate) fn flatten(&self) -> Vec<String> {
        [self.0.to_owned(), self.1.to_owned(), self.2.to_owned()]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
    }
}

impl SnowflakeConnector {
    pub(super) fn generate_diff_queries(&self, diffs: &LocalConnectorDiffs) -> PrioritizedQueries {
        let user_queries = users::prepare_queries(&diffs.users);
        let group_queries = groups::prepare_queries(&diffs.groups, self);
        let policy_queries = policies::prepare_queries(&diffs.policies);
        let default_policy_queries = default_policies::prepare_queries(&diffs.default_policies);

        let mut prioritized_queries = user_queries;
        prioritized_queries.extend(&group_queries);
        prioritized_queries.extend(&policy_queries);
        prioritized_queries.extend(&default_policy_queries);
        prioritized_queries
    }
}
