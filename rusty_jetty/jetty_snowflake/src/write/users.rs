//! managing the write path for users

use jetty_core::access_graph::translate::diffs::users;

use super::PrioritizedQueries;

pub(super) fn prepare_queries(user_diffs: &[users::LocalDiff]) -> PrioritizedQueries {
    let mut res = PrioritizedQueries::default();

    user_diffs.iter().for_each(|diff| {
        res.2.extend(
            diff.group_membership
                .add
                .iter()
                .map(|g| format!("GRANT ROLE \"{}\" TO USER \"{}\";", g, &diff.user)),
        );
        res.2.extend(
            diff.group_membership
                .remove
                .iter()
                .map(|g| format!("REVOKE ROLE \"{}\" FROM USER \"{}\";", g, &diff.user)),
        );
    });
    res
}
