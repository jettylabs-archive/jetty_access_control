//! Diff command execution

use anyhow::{anyhow, Result};

use jetty_core::{project, write::diff::get_diffs};

use crate::new_jetty_with_connectors;

pub(super) async fn diff() -> Result<()> {
    let jetty = &mut new_jetty_with_connectors(".", true).await.map_err(|_| {
        anyhow!(
            "unable to find {} - make sure you are in a \
        Jetty project directory, or create a new project by running `jetty new`",
            project::jetty_cfg_path_local().display()
        )
    })?;

    let diffs = get_diffs(jetty)?;

    // Now print out the diffs
    println!("\nUSERS\n──────────────────");
    if !diffs.users.is_empty() {
        diffs.users.iter().for_each(|diff| println!("{diff}"));
    } else {
        println!("No changes found");
    };

    println!("\nGROUPS\n──────────────────");
    if !diffs.groups.is_empty() {
        diffs.groups.iter().for_each(|diff| println!("{diff}"));
    } else {
        println!("No changes found");
    };

    println!("\nPOLICIES\n──────────────────");
    if !diffs.policies.is_empty() {
        diffs.policies.iter().for_each(|diff| println!("{diff}"));
    } else {
        println!("No changes found");
    };

    println!("\nDEFAULT POLICIES\n──────────────────");
    if !diffs.default_policies.is_empty() {
        diffs
            .default_policies
            .iter()
            .for_each(|diff| println!("{diff}"));
    } else {
        println!("No changes found");
    };

    Ok(())
}
