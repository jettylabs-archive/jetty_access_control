//! Mod for the remove command-line argument

use anyhow::{anyhow, Result};
use colored::Colorize;
use jetty_core::{project, write};

use crate::{cmd::RemoveOrModifyNodeType, new_jetty_with_connectors};

pub(super) async fn remove(node_type: &RemoveOrModifyNodeType, name: &String) -> Result<()> {
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
        "Removing references to {node_type_string} {name} from your project configuration files"
    );
    match node_type {
        RemoveOrModifyNodeType::Group => write::remove_group_name(jetty, name)?,
        RemoveOrModifyNodeType::User => write::remove_user_name(jetty, name)?,
    };
    println!("{}", "Success".green());

    Ok(())
}
