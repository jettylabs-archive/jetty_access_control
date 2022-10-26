mod autocomplete;
mod dbt;
mod pki;
mod snowflake;
mod tableau;
mod validation;

use self::validation::filled_validator;
use crate::{
    ascii::{print_banner, JETTY_ACCENT, JETTY_ORANGE, JETTY_ORANGE_DARK, JETTY_ORANGE_I},
    init::{
        dbt::ask_dbt_connector_setup, snowflake::ask_snowflake_connector_setup,
        tableau::ask_tableau_connector_setup,
    },
    tui::AltScreenContext,
};

use std::collections::HashMap;

use anyhow::Result;
use colored::Colorize;
use inquire::{
    list_option::ListOption,
    set_global_render_config,
    ui::{Color, RenderConfig, StyleSheet, Styled},
    validator::Validation,
    MultiSelect, Text,
};

use jetty_core::jetty::{ConnectorConfig, ConnectorNamespace, CredentialsMap, JettyConfig};

pub(crate) async fn init() -> Result<()> {
    let (jetty_config, credentials) = inquire_init().await?;
    println!(
        "{}\n{}",
        "jetty_config.yaml".underline(),
        jetty_config.to_yaml()?
    );
    println!(
        "\n{}\n{}",
        "connectors.yaml".underline(),
        yaml_peg::serde::to_string(&credentials).map_err(anyhow::Error::from)?
    );
    Ok(())
}

/// Ask the user to respond to a series of questions to create the Jetty
/// config and the connectors config, producing both.
async fn inquire_init() -> Result<(JettyConfig, HashMap<String, CredentialsMap>)> {
    // Create an alternate screen for this scope.
    let alt_screen_context = AltScreenContext::start()?;
    // Print the Jetty Labs banner.
    print_banner();

    // Set up render configuration for the questions.
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

    let mut jetty_config = JettyConfig::new();
    let mut credentials = HashMap::new();

    jetty_config.set_name(ask_project_name()?);
    let connector_types = ask_select_connectors()?;

    for connector in connector_types {
        println!(
            "{}",
            format!("{} connector configuration", connector.color(JETTY_ORANGE)).underline()
        );
        let connector_namespace = ask_connector_namespace(connector)?;
        jetty_config.connectors.insert(
            ConnectorNamespace(connector_namespace.clone()),
            ConnectorConfig::new(connector.to_owned(), Default::default()),
        );

        let mut credentials_map = match connector {
            "dbt" => ask_dbt_connector_setup()?,
            "snowflake" => ask_snowflake_connector_setup().await?,
            "tableau" => ask_tableau_connector_setup().await?,
            &_ => panic!("Unrecognized input"),
        };
        credentials_map.insert("type".to_owned(), connector.to_owned());
        credentials.insert(connector_namespace.to_owned(), credentials_map);
    }

    // Leave the alternate screen.
    alt_screen_context.end();
    Ok((jetty_config, credentials))
}

fn ask_project_name() -> Result<String> {
    let project_name = Text::new("Project Name")
        .with_validator(filled_validator)
        .with_placeholder("jetty")
        .with_default("jetty")
        .prompt()?;
    Ok(project_name)
}

fn ask_select_connectors() -> Result<Vec<&'static str>> {
    let options = vec!["dbt", "snowflake", "tableau"];

    let validator = |connectors: &[ListOption<&&str>]| {
        if connectors.is_empty() {
            Ok(Validation::Invalid(
                "Please select one or more connectors.".into(),
            ))
        } else if connectors.iter().any(|i| *i.value == "dbt")
            && !connectors.iter().any(|i| *i.value == "snowflake")
        {
            Ok(Validation::Invalid("dbt depends on Snowflake".into()))
        } else {
            Ok(Validation::Valid)
        }
    };

    // let formatter: MultiOptionFormatter<&str> = &|a| format!("{} selectors available", a.len());

    let connectors = MultiSelect::new("Which connectors would you like to use?", options)
        .with_validator(validator)
        // .with_formatter(formatter)
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
