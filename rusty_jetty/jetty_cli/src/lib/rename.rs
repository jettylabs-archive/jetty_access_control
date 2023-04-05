//! Mod for the remove command-line argument

use anyhow::{anyhow, Result};
use colored::Colorize;
use jetty_core::{project, write};

use crate::{cmd::RemoveOrModifyNodeType, new_jetty_with_connectors};

pub(super) async fn rename(
    node_type: &RemoveOrModifyNodeType,
    old: &String,
    new: &String,
) -> Result<()> {
    let jetty = &new_jetty_with_connectors(".", true).await.map_err(|_| {
        anyhow!(
            "unable to find {} - make sure you are in a \
        Jetty project directory, or create a new project by running `jetty new`",
            project::jetty_cfg_path_local().display()
        )
    })?;

    let node_type_string = match node_type {
        RemoveOrModifyNodeType::Group => "group",
        RemoveOrModifyNodeType::User => "user",
    };
    println!(
        "updating references from {node_type_string} `{old}` to `{new}` in your project configuration files"
    );
    match node_type {
        RemoveOrModifyNodeType::Group => write::update_group_name(jetty, old, new)?,
        RemoveOrModifyNodeType::User => write::update_user_name(jetty, old, new)?,
    };
    println!("{}", "Success".green());

    Ok(())
}
