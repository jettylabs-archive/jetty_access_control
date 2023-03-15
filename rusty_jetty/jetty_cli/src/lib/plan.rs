//! plan the changes for Jetty and each connector

use std::collections::HashMap;

use anyhow::{anyhow, Result};
use jetty_core::{project, write::diff::get_diffs};

use crate::new_jetty_with_connectors;

pub(super) async fn plan() -> Result<()> {
    let jetty = &mut new_jetty_with_connectors(".", true).await.map_err(|_| {
        anyhow!(
            "unable to find {} - make sure you are in a \
        Jetty project directory, or create a new project by running `jetty new`",
            project::jetty_cfg_path_local().display()
        )
    })?;

    let diffs = get_diffs(jetty)?;

    // make sure there's an existing access graph
    let ag = jetty.try_access_graph()?;

    let connector_specific_diffs = diffs.split_by_connector();

    let tr = ag.translator();

    let local_diffs = connector_specific_diffs
        .iter()
        .map(|(k, v)| (k.to_owned(), tr.translate_diffs_to_local(v, k)))
        .collect::<HashMap<_, _>>();

    // Exit early if there haven't been any changes
    if local_diffs.is_empty() {
        println!("No changes found");
        return Ok(());
    }

    let plans: HashMap<_, _> = local_diffs
        .iter()
        .map(|(k, v)| (k.to_owned(), jetty.connectors[k].plan_changes(v)))
        .collect();

    for (c, plan) in plans {
        println!("{c}:");
        if !plan.is_empty() {
            plan.iter()
                .for_each(|s| println!("{}\n", textwrap::indent(s, "  ")));
            println!("\n")
        } else {
            println!("  No changes planned\n");
        }
    }
    Ok(())
}
