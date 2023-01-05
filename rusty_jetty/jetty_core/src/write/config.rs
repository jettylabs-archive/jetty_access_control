//! module to help manage the config
mod json_schema;

use anyhow::Result;

use std::collections::HashSet;

use super::{groups, users};

pub use json_schema::{
    generate_env_schema_from_config, write_config_schema, write_settings_and_schema,
};

/// return all group names
fn group_names() -> Result<HashSet<String>> {
    let config = groups::parser::read_config_file().unwrap_or_default();
    Ok(groups::parser::get_all_group_names(&config))
}

/// return all user names
fn user_names() -> Result<HashSet<String>> {
    let paths = users::get_config_paths()?;
    let config = users::parser::read_config_files(paths)?;
    Ok(users::parser::get_all_user_names(&config))
}
