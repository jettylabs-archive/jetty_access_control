use self::validation::{filled_validator, project_dir_does_not_exist_validator};
use crate::{
    ascii::{print_banner, JETTY_ACCENT, JETTY_ORANGE, JETTY_ORANGE_DARK},
    new::inquiry::{
        dbt::ask_dbt_connector_setup, snowflake::ask_snowflake_connector_setup,
        tableau::ask_tableau_connector_setup,
    },
    tui::AltScreenContext,
};
use jetty_core::project;

use std::{collections::HashMap, path::Path};

use anyhow::{Context, Result};
use colored::Colorize;
use inquire::{
    list_option::ListOption,
    set_global_render_config,
    ui::{RenderConfig, StyleSheet, Styled},
    validator::Validation,
    MultiSelect, Text,
};
use jetty_core::jetty::{ConnectorConfig, ConnectorNamespace, CredentialsMap, JettyConfig};

mod autocomplete;
mod dbt;
mod snowflake;
mod tableau;
mod validation;

const SKIP_CMD: &str = "/skip";

/// Ask the user to respond to a series of questions to create the Jetty
/// config and the connectors config, producing both.
pub(crate) async fn inquire_init(
    overwrite_project_dir: bool,
    project_name: &Option<String>,
) -> Result<(JettyConfig, HashMap<String, CredentialsMap>)> {
    // Create an alternate screen for this scope.
    let alt_screen_context = AltScreenContext::start()?;
    // Print the Jetty Labs banner.
    print_banner();

    // Set up render configuration for inquire questions.
    setup_render_config();

    let mut jetty_config = JettyConfig::new();
    let mut credentials = HashMap::new();

    // Set project ID
    jetty_config.project_id = jetty_core::jetty::new_project_id();

    jetty_config.set_name(ask_project_name(overwrite_project_dir, project_name)?);
    let connector_types = ask_select_connectors(false)?;

    update_connector_info(connector_types, &mut jetty_config, &mut credentials).await?;

    // Leave the alternate screen.
    alt_screen_context.end();
    Ok((jetty_config, credentials))
}

fn setup_render_config() {
    let stylesheet = StyleSheet::new().with_fg(JETTY_ORANGE_DARK);
    let accent_stylesheet = StyleSheet::new().with_fg(JETTY_ACCENT);
    let mut render_config = RenderConfig::default()
        .with_prompt_prefix(Styled::new(">").with_style_sheet(accent_stylesheet))
        .with_answer(stylesheet)
        .with_selected_checkbox(Styled::new("[x]").with_style_sheet(accent_stylesheet))
        .with_help_message(accent_stylesheet);
    render_config.answered_prompt_prefix = Styled::new("👍");
    render_config.highlighted_option_prefix = Styled::new("➡");
    set_global_render_config(render_config);
}

fn ask_project_name(
    overwrite_project_dir: bool,
    project_name_input: &Option<String>,
) -> Result<String> {
    let project_name = if let Some(s) = project_name_input {
        s.to_owned()
    } else {
        let mut project_name_prompt = Text::new("Project Name")
            .with_validator(filled_validator)
            .with_placeholder("jetty")
            .with_default("jetty");

        if !overwrite_project_dir {
            project_name_prompt =
                project_name_prompt.with_validator(project_dir_does_not_exist_validator)
        }

        project_name_prompt.prompt()?
    };

    // Check to see if the directory <project_name> exists. This is also checked with
    // project_dir_does_not_exist_validator, but this is still necessary in the case
    // that a project name is specified via the init command.
    if Path::new(&project_name).is_dir() && !overwrite_project_dir {
        return Err(anyhow::anyhow!(
            "The directory {project_name} already exists. Choose a different project name or \
            use the -o flag to overwrite the existing directory."
        ));
    }

    Ok(project_name)
}

fn ask_select_connectors(skip_dbt_validation: bool) -> Result<Vec<&'static str>> {
    let options = vec!["dbt", "snowflake", "tableau"];

    let validator = move |connectors: &[ListOption<&&str>]| {
        if connectors.is_empty() {
            Ok(Validation::Invalid(
                "Please select one or more connectors.".into(),
            ))
        } else if connectors.iter().any(|i| *i.value == "dbt")
            && !connectors.iter().any(|i| *i.value == "snowflake")
            && !skip_dbt_validation
        {
            Ok(Validation::Invalid("dbt depends on Snowflake".into()))
        } else {
            Ok(Validation::Valid)
        }
    };

    let connectors = MultiSelect::new("Which connectors would you like to use?", options)
        .with_validator(validator)
        .prompt()?;
    Ok(connectors)
}

fn ask_connector_namespace(name: &str) -> Result<String> {
    // TODO: update default for multiple instances
    let connector_namespace = Text::new(&format!("Connector Name for {name}"))
        .with_validator(filled_validator)
        .with_default(name)
        .with_help_message("The name Jetty will use to refer to this specific connection. We recommend a single descriptive word.")
        .prompt()?;
    Ok(connector_namespace)
}

/// Allow the user at add new connectors to an existing project.
pub(crate) async fn inquire_add() -> Result<(JettyConfig, HashMap<String, CredentialsMap>)> {
    // Create an alternate screen for this scope.
    let alt_screen_context = AltScreenContext::start()?;

    // Set up render configuration for inquire questions.
    setup_render_config();

    // Read in the existing configuration
    let mut jetty_config =
        JettyConfig::read_from_file(project::jetty_cfg_path_local()).context(format!(
            "unable to read Jetty Config file at ({}); you must be in an existing project to run 'jetty add'",
            project::jetty_cfg_path_local().to_string_lossy()
        ))?;
    // Read in the existing credentials
    let mut credentials =
        jetty_core::fetch_credentials(project::connector_cfg_path()).context(format!(
            "unable to read Jetty connectors file file at ({}); you must set up a project with 'jetty init' before running 'jetty add'",
            project::connector_cfg_path().to_string_lossy()
        ))?;

    let connector_types = ask_select_connectors(true)?;

    update_connector_info(connector_types, &mut jetty_config, &mut credentials).await?;

    // Leave the alternate screen.
    alt_screen_context.end();
    Ok((jetty_config, credentials))
}

/// Given a list of requested connectors and a config and credentials object, fetch and update the config and credentials.
async fn update_connector_info(
    connectors: Vec<&str>,
    config: &mut JettyConfig,
    credentials: &mut HashMap<String, CredentialsMap>,
) -> Result<()> {
    for connector in connectors {
        println!(
            "{}",
            format!("{} connector configuration", connector.color(JETTY_ORANGE)).underline()
        );
        let connector_namespace_user_input = ask_connector_namespace(connector)?;
        let connector_namespace = ConnectorNamespace(connector_namespace_user_input.clone());

        let credentials_map = match connector {
            "dbt" => ask_dbt_connector_setup(),
            "snowflake" => ask_snowflake_connector_setup(connector_namespace.clone()).await,
            "tableau" => ask_tableau_connector_setup().await,
            &_ => panic!("Unrecognized input"),
        };
        let mut credentials_map = match credentials_map {
            Ok(c) => c,
            Err(e) => {
                if e.to_string().contains("skipped") {
                    continue;
                } else {
                    return Err(e);
                }
            }
        };

        config.connectors.insert(
            connector_namespace,
            ConnectorConfig::new(connector.to_owned(), Default::default()),
        );

        credentials_map.insert("type".to_owned(), connector.to_owned());
        credentials.insert(connector_namespace_user_input.to_owned(), credentials_map);
    }
    Ok(())
}
