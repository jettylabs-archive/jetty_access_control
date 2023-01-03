//! Get all the diffs. Return a Diffs object

use anyhow::Result;

use crate::Jetty;

use super::{assets, groups, users, GlobalDiffs};

/// Get all the diffs
pub fn get_diffs(jetty: &mut Jetty) -> Result<GlobalDiffs> {
    // make sure there's an existing access graph
    jetty.try_access_graph()?;

    // get group config
    let validated_group_config = &groups::parse_and_validate_groups(jetty)?;

    // get user_config
    let validated_user_config =
        &users::get_validated_file_config_map(jetty, validated_group_config)?;

    // user identity diffs
    let user_identity_diffs = users::diff::get_identity_diffs(jetty, validated_group_config)?;
    // update the graph before parsing other configs/generating other diffs
    users::diff::update_graph(jetty, &user_identity_diffs)?;
    // group membership diffs
    let group_membership_diffs =
        users::get_membership_diffs(jetty, validated_user_config, validated_group_config)?;

    // combined user diffs
    let user_diffs = users::diff::combine_diffs(&user_identity_diffs, &group_membership_diffs);

    // group diffs
    let group_diffs = groups::generate_diffs(validated_group_config, jetty)?;

    // now get the policy diff
    // need to get the group configs and all available connectors
    let policy_diffs = assets::get_policy_diffs(jetty, validated_group_config)?;

    // now get the policy diff
    // need to get the group configs and all available connectors
    let default_policy_diffs = assets::get_default_policy_diffs(jetty, validated_group_config)?;

    Ok(GlobalDiffs {
        groups: group_diffs,
        users: user_diffs.into_iter().collect(),
        default_policies: default_policy_diffs,
        policies: policy_diffs,
    })
}
