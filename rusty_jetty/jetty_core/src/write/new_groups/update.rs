//! functionality to update the config, when necessary

use std::fs;

use anyhow::Result;

use crate::{
    write::{new_groups, UpdateConfig},
    Jetty,
};

use super::write_env_config;

fn update_user_name(jetty: &Jetty, old: &String, new: &str) -> Result<()> {
    let mut config: Vec<_> = new_groups::parse_and_validate_groups(&jetty)?
        .into_iter()
        .collect();
    let mut modified = false;
    for group in &mut config {
        if group.update_user_name(old, new)? {
            modified = true;
        }
    }
    if modified {
        write_env_config(&config.into_iter().collect())?;
    };
    Ok(())
}
fn remove_user_name(jetty: &Jetty, name: &String) -> Result<()> {
    let mut config: Vec<_> = new_groups::parse_and_validate_groups(&jetty)?
        .into_iter()
        .collect();
    let mut modified = false;
    for group in &mut config {
        if group.remove_user_name(name)? {
            modified = true;
        }
    }
    if modified {
        write_env_config(&config.into_iter().collect())?;
    };
    Ok(())
}

fn update_group_name(jetty: &Jetty, old: &String, new: &str) -> Result<()> {
    let mut config: Vec<_> = new_groups::parse_and_validate_groups(&jetty)?
        .into_iter()
        .collect();
    let mut modified = false;
    for group in &mut config {
        if group.update_group_name(old, new)? {
            modified = true;
        }
    }
    if modified {
        write_env_config(&config.into_iter().collect())?;
    };
    Ok(())
}

fn remove_group_name(jetty: &Jetty, name: &String) -> Result<()> {
    let mut config: Vec<_> = new_groups::parse_and_validate_groups(&jetty)?
        .into_iter()
        .collect();
    let mut modified1 = false;
    let mut modified2 = false;
    let updated_config = config
        .into_iter()
        .filter(|g| {
            if &g.name != name {
                true
            } else {
                modified2 = true;
                false
            }
        })
        .map(|mut g| -> Result<_> {
            if g.remove_group_name(name)? {
                modified1 = true;
            };
            Ok(g)
        })
        .collect::<Result<_>>()?;
    if modified1 || modified2 {
        write_env_config(&updated_config)?;
    };
    Ok(())
}
