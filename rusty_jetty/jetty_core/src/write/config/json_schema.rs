//! Build the json schema necessary to validate configs in vscode

use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use tinytemplate::TinyTemplate;

use std::{
    collections::{BTreeSet, HashSet},
    fs,
    path::{Path, PathBuf},
};

use crate::{connectors::AssetType, jetty::ConnectorNamespace, project, write, Jetty};

#[derive(Serialize)]
struct TemplateContext {
    /// a vector of (ConnectorNamespace, AssetType, Vec<Privileges>)
    privilege_map: Vec<(ConnectorNamespace, AssetType, BTreeSet<String>)>,
    /// a vector of (ConnectorNamespace, AssetType)
    type_map: Vec<(ConnectorNamespace, BTreeSet<AssetType>)>,
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
    generate_env_schema(&users, &groups, jetty)
}

/// Generate a json schema file including information from the config files
pub(crate) fn generate_env_schema(
    users: &HashSet<String>,
    groups: &HashSet<String>,
    jetty: &Jetty,
) -> Result<String> {
    let template = include_str!("../../../templates/schemas/config_template.txt");
    let context = build_context(users, groups, jetty);

    let mut tt = TinyTemplate::new();
    tt.add_template("env_schema", template)?;
    tt.set_default_formatter(&tinytemplate::format_unescaped);
    tt.render("env_schema", &context)
        .context("rendering template")
}

/// Generate the context necessary to build the config schema
fn build_context(
    users: &HashSet<String>,
    groups: &HashSet<String>,
    jetty: &Jetty,
) -> TemplateContext {
    let mut connectors: Vec<_> = jetty.connectors.keys().cloned().collect();
    // get connector, asset_type, privilege map
    let asset_type_privileges = jetty.get_asset_type_privileges();
    let mut privilege_map: Vec<_> = asset_type_privileges
        .iter()
        .flat_map(|(conn, v)| {
            v.iter().map(|(a, p)| {
                (
                    conn.clone(),
                    a.clone(),
                    p.iter().cloned().collect::<BTreeSet<_>>(),
                )
            })
        })
        .collect();

    let mut type_map: Vec<_> = asset_type_privileges
        .iter()
        .map(|(c, tm)| {
            (
                c.to_owned(),
                tm.iter()
                    .map(|(asset_type, _)| asset_type.to_owned())
                    .collect(),
            )
        })
        .collect();
    type_map.sort_by_key(|a| format!("{a:?}"));
    type_map.dedup();

    connectors.sort_by_key(|a| a.to_string());
    privilege_map.sort_by_key(|a| format!("{a:?}"));
    let mut users: Vec<String> = users.iter().cloned().collect();
    users.sort();
    let mut groups: Vec<String> = groups.iter().cloned().collect();
    groups.sort();

    TemplateContext {
        connectors,
        privilege_map,
        type_map,
        users,
        groups,
    }
}

/// Write the config.json schema file
pub fn write_config_schema<P: AsRef<Path>>(schema: &String, path_prefix: P) -> Result<()> {
    let path = PathBuf::from(path_prefix.as_ref())
        .join(project::DEFAULT_SCHEMA_DIR.as_path())
        .join("config.json");
    fs::create_dir_all(
        path.parent()
            .ok_or_else(|| anyhow!("unable to find parent directory for {path:?}"))?,
    )?;

    fs::write(path, schema).context("writing config schema")
}

/// save all other files to the right directories
pub fn write_settings_and_schema<P: AsRef<Path>>(jetty: &Jetty, path_prefix: P) -> Result<()> {
    let config_schema = generate_env_schema_from_config(jetty)?;
    write_config_schema(&config_schema, &path_prefix)?;

    write_user_schema(&path_prefix)?;
    write_group_schema(&path_prefix)?;
    write_asset_schema(&path_prefix)?;
    write_tag_schema(&path_prefix)?;

    write_vs_code_settings(&path_prefix)?;

    Ok(())
}

/// Write the user schema
fn write_user_schema<P: AsRef<Path>>(path_prefix: P) -> Result<()> {
    let path = PathBuf::from(path_prefix.as_ref())
        .join(project::DEFAULT_SCHEMA_DIR.as_path())
        .join("users.json");

    fs::create_dir_all(
        path.parent()
            .ok_or_else(|| anyhow!("unable to find parent directory for {path:?}"))?,
    )?;
    let user_schema = include_str!("../../../templates/schemas/users.json");

    fs::write(path, user_schema).context("writing users schema")
}

/// Write the group schema
fn write_group_schema<P: AsRef<Path>>(path_prefix: P) -> Result<()> {
    let path = PathBuf::from(path_prefix.as_ref())
        .join(project::DEFAULT_SCHEMA_DIR.as_path())
        .join("groups.json");

    fs::create_dir_all(
        path.parent()
            .ok_or_else(|| anyhow!("unable to find parent directory for {path:?}"))?,
    )?;
    let group_schema = include_str!("../../../templates/schemas/groups.json");

    fs::write(path, group_schema).context("writing groups schema")
}

/// Write the tag schema
fn write_tag_schema<P: AsRef<Path>>(path_prefix: P) -> Result<()> {
    let path = PathBuf::from(path_prefix.as_ref())
        .join(project::DEFAULT_SCHEMA_DIR.as_path())
        .join("tags.json");

    fs::create_dir_all(
        path.parent()
            .ok_or_else(|| anyhow!("unable to find parent directory for {path:?}"))?,
    )?;
    let tag_schema = include_str!("../../../templates/schemas/tags.json");

    fs::write(path, tag_schema).context("writing tags schema")
}

/// Write the asset schema
fn write_asset_schema<P: AsRef<Path>>(path_prefix: P) -> Result<()> {
    let path = PathBuf::from(path_prefix.as_ref())
        .join(project::DEFAULT_SCHEMA_DIR.as_path())
        .join("assets.json");

    fs::create_dir_all(
        path.parent()
            .ok_or_else(|| anyhow!("unable to find parent directory for {path:?}"))?,
    )?;
    let asset_schema = include_str!("../../../templates/schemas/assets.json");

    fs::write(path, asset_schema).context("writing assets schema")
}

/// Write vscode settings
fn write_vs_code_settings<P: AsRef<Path>>(path_prefix: P) -> Result<()> {
    let path = PathBuf::from(path_prefix.as_ref()).join(project::VSCODE_SETTINGS_PATH.as_path());

    fs::create_dir_all(
        path.parent()
            .ok_or_else(|| anyhow!("unable to find parent directory for {path:?}"))?,
    )?;

    let settings = include_str!("../../../templates/settings/settings.json");

    fs::write(path, settings).context("writing vscode settings")
}
