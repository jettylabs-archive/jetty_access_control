//! functionality to update the config, when necessary

use std::fs;

use anyhow::Result;

use crate::{
    write::{groups, UpdateConfig},
    Jetty,
};

use super::{bootstrap::write_user_config_file, get_validated_file_config_map};

pub(crate) fn update_user_name(jetty: &Jetty, old: &str, new: &str) -> Result<()> {
    let validated_group_config = &groups::parse_and_validate_groups(jetty)?;
    let mut config = get_validated_file_config_map(jetty, validated_group_config)?;

    for (path, user) in config.iter_mut() {
        if user.update_user_name(old, new)? {
            write_user_config_file(path, user)?
        }
    }
    Ok(())
}
pub(crate) fn remove_user_name(jetty: &Jetty, name: &str) -> Result<()> {
    let validated_group_config = &groups::parse_and_validate_groups(jetty)?;
    let config = get_validated_file_config_map(jetty, validated_group_config)?;

    for (path, user) in config {
        if user.name == name {
            fs::remove_file(path)?;
            return Ok(());
        }
    }
    Ok(())
}
pub(crate) fn update_group_name(jetty: &Jetty, old: &str, new: &str) -> Result<()> {
    let validated_group_config = &groups::parse_and_validate_groups(jetty)?;
    let mut config = get_validated_file_config_map(jetty, validated_group_config)?;

    for (path, user) in config.iter_mut() {
        if user.update_group_name(old, new)? {
            write_user_config_file(path, user)?
        }
    }
    Ok(())
}
pub(crate) fn remove_group_name(jetty: &Jetty, name: &str) -> Result<()> {
    let validated_group_config = &groups::parse_and_validate_groups(jetty)?;
    let mut config = get_validated_file_config_map(jetty, validated_group_config)?;

    for (path, user) in config.iter_mut() {
        if user.remove_group_name(name)? {
            write_user_config_file(path, user)?
        }
    }
    Ok(())
}
