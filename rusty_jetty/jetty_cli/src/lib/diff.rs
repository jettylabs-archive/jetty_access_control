//! Diff command execution

use anyhow::{anyhow, Result};

use jetty_core::{
    project,
    write::{
        assets::{get_default_policy_diffs, get_policy_diffs},
        groups::parse_and_validate_groups,
        users::diff::get_identity_diffs,
    },
};

use crate::new_jetty_with_connectors;

pub(super) async fn diff() -> Result<()> {
    let jetty = new_jetty_with_connectors().await.map_err(|_| {
        anyhow!(
            "unable to find {} - make sure you are in a \
        Jetty project directory, or create a new project by running `jetty init`",
            project::jetty_cfg_path_local().display()
        )
    })?;

    // make sure there's an existing access graph
    jetty.try_access_graph()?;

    // user identity diffs
    let user_identity_diffs = get_identity_diffs(&jetty)?;

    // group diffs
    let validated_group_config = parse_and_validate_groups(&jetty)?;
    let group_diff = jetty_core::write::get_group_diff(&validated_group_config, &jetty)?;

    // now get the policy diff
    // need to get the group configs and all available connectors
    let policy_diff = get_policy_diffs(&jetty, &validated_group_config)?;

    // now get the policy diff
    // need to get the group configs and all available connectors
    let default_policy_diff = get_default_policy_diffs(&jetty, &validated_group_config)?;

    // Now print out the diffs

    println!("\nUSER IDENTITIES\n----------------");
    if !user_identity_diffs.is_empty() {
        user_identity_diffs
            .iter()
            .for_each(|diff| println!("{diff}"));
    } else {
        println!("No changes found");
    };

    println!("\nGROUPS\n----------------");
    if !group_diff.is_empty() {
        group_diff.iter().for_each(|diff| println!("{diff}"));
    } else {
        println!("No changes found");
    };

    println!("\nPOLICIES\n----------------");
    if !policy_diff.is_empty() {
        policy_diff.iter().for_each(|diff| println!("{diff}"));
    } else {
        println!("No changes found");
    };

    println!("\nDEFAULT POLICIES\n----------------");
    if !default_policy_diff.is_empty() {
        default_policy_diff
            .iter()
            .for_each(|diff| println!("{diff}"));
    } else {
        println!("No changes found");
    };

    Ok(())
}
