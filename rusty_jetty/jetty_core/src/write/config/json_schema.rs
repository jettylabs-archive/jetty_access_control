//! Build the json schema necessary to validate configs in vscode

use anyhow::{Context, Result};
use serde::Serialize;
use tinytemplate::TinyTemplate;

use std::{
    collections::{BTreeSet, HashSet},
    fs,
};

use crate::{connectors::AssetType, jetty::ConnectorNamespace, project, write, Jetty};

#[derive(Serialize)]
struct TemplateContext {
    /// a vector of (ConnectorNamespace, AssetType, Vec<Privileges>)
    privilege_map: Vec<(ConnectorNamespace, AssetType, BTreeSet<String>)>,
    /// a vector of connector namespaces
    connectors: Vec<ConnectorNamespace>,
    /// a vector of user names from the config
    users: Vec<String>,
    /// a vector of group names form the config
    groups: Vec<String>,
}
/// Generate a json schmema including info from the config files, reading in the files directly
pub fn generate_env_schema_from_config(jetty: &Jetty) -> Result<String> {
    let users = write::config::user_names()?;
    let groups = write::config::group_names()?;
    generate_env_schema(users, groups, jetty)
}

/// Generate a json schema file including information from the config files
fn generate_env_schema(
    users: HashSet<String>,
    groups: HashSet<String>,
    jetty: &Jetty,
) -> Result<String> {
    let template = include_str!("../../../templates/schemas/config_template.txt");
    let context = build_context(users, groups, jetty);

    let mut tt = TinyTemplate::new();
    tt.add_template("env_schema", template)?;
    tt.render("env_schema", &context)
        .context("rendering template")
}

/// Generate the context necessary to build the config schema
fn build_context(
    users: HashSet<String>,
    groups: HashSet<String>,
    jetty: &Jetty,
) -> TemplateContext {
    let mut connectors: Vec<_> = jetty.connectors.keys().cloned().collect();
    // get connector, asset_type, privilege map
    let binding = jetty.get_asset_type_privileges();
    let mut privilege_map: Vec<_> = binding
        .iter()
        .map(|(conn, v)| {
            v.iter().map(|(a, p)| {
                (
                    conn.clone(),
                    a.clone(),
                    p.iter().cloned().collect::<BTreeSet<_>>(),
                )
            })
        })
        .flatten()
        .collect();

    connectors.sort_by(|a, b| a.to_string().cmp(&b.to_string()));
    privilege_map.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));
    let mut users: Vec<String> = users.into_iter().collect();
    users.sort();
    let mut groups: Vec<String> = groups.into_iter().collect();
    groups.sort();

    TemplateContext {
        connectors,
        privilege_map,
        users,
        groups,
    }
}

/// Write the config.json schema file
pub fn write_config_schema(schema: &String) -> Result<()> {
    fs::create_dir_all(project::DEFAULT_SCHEMA_DIR.as_path())?;
    let path = project::DEFAULT_SCHEMA_DIR.join("config.json");

    fs::write(path, schema).context("writing config schema")
}

/// save all other files to the right directories
pub fn write_settings_and_schema(jetty: &Jetty) -> Result<()> {
    let config_schema = generate_env_schema_from_config(jetty)?;
    write_config_schema(&config_schema)?;

    write_user_schema()?;
    write_group_schema()?;
    write_asset_schema()?;
    write_tag_schema()?;

    write_vs_code_settings()?;

    Ok(())
}

/// Write the user schema
fn write_user_schema() -> Result<()> {
    fs::create_dir_all(project::DEFAULT_SCHEMA_DIR.as_path())?;
    let user_schema = include_str!("../../../templates/schemas/users.json");

    fs::write(project::DEFAULT_SCHEMA_DIR.join("users.json"), user_schema)
        .context("writing users schema")
}

/// Write the group schema
fn write_group_schema() -> Result<()> {
    fs::create_dir_all(project::DEFAULT_SCHEMA_DIR.as_path())?;
    let group_schema = include_str!("../../../templates/schemas/groups.json");

    fs::write(
        project::DEFAULT_SCHEMA_DIR.join("groups.json"),
        group_schema,
    )
    .context("writing groups schema")
}

/// Write the tag schema
fn write_tag_schema() -> Result<()> {
    fs::create_dir_all(project::DEFAULT_SCHEMA_DIR.as_path())?;
    let tag_schema = include_str!("../../../templates/schemas/tags.json");

    fs::write(project::DEFAULT_SCHEMA_DIR.join("tags.json"), tag_schema)
        .context("writing tags schema")
}

/// Write the asset schema
fn write_asset_schema() -> Result<()> {
    fs::create_dir_all(project::DEFAULT_SCHEMA_DIR.as_path())?;
    let asset_schema = include_str!("../../../templates/schemas/assets.json");

    fs::write(
        project::DEFAULT_SCHEMA_DIR.join("assets.json"),
        asset_schema,
    )
    .context("writing assets schema")
}

/// Write vscode settings
fn write_vs_code_settings() -> Result<()> {
    fs::create_dir_all(project::VSCODE_SETTINGS_PATH.parent().unwrap())?;
    let asset_schema = include_str!("../../../templates/settings/settings.json");

    fs::write(project::VSCODE_SETTINGS_PATH.as_path(), asset_schema)
        .context("writing assets schema")
}
