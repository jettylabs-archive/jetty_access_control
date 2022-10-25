mod dbt;
mod snowflake;
mod tableau;
mod validation;

use crate::ascii::{print_banner, JETTY_ORANGE};

use std::{collections::HashMap, io::stdout};

use anyhow::Result;
use colored::Colorize;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use inquire::{
    list_option::ListOption, set_global_render_config, ui::RenderConfig, validator::Validation,
    MultiSelect, Text,
};
use jetty_core::jetty::{CredentialsMap, JettyConfig};

use self::{
    dbt::dbt_connector_setup, snowflake::snowflake_connector_setup,
    tableau::tableau_connector_setup, validation::filled_validator,
};

pub(crate) fn init() -> Result<(JettyConfig, HashMap<String, CredentialsMap>)> {
    let mut jetty_config = JettyConfig::new();
    let mut credentials = HashMap::new();
    execute!(stdout(), EnterAlternateScreen)?;
    print_banner();

    let render_config = RenderConfig::default();
    set_global_render_config(render_config);

    jetty_config.name = project_name()?;
    let connector_types = connector_select()?;

    for connector in connector_types {
        println!(
            "{}",
            format!("{} connector configuration", connector.color(JETTY_ORANGE)).underline()
        );
        let connector_namespace = connector_namespace(connector)?;

        match connector {
            "dbt" => dbt_connector_setup()?,
            "snowflake" => snowflake_connector_setup()?,
            "tableau" => tableau_connector_setup()?,
            &_ => panic!("Unrecognized input"),
        };
    }

    execute!(stdout(), LeaveAlternateScreen).map_err(anyhow::Error::from)?;
    Ok((jetty_config, credentials))
}

fn project_name() -> Result<String> {
    let project_name = Text::new("Project Name")
        .with_validator(filled_validator)
        .with_placeholder("jetty")
        .with_default("jetty")
        .prompt()?;
    Ok(project_name)
}

fn connector_select() -> Result<Vec<&'static str>> {
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

fn connector_namespace(name: &str) -> Result<String> {
    // TODO: update default for multiple instances
    let connector_namespace = Text::new(&format!("Connector Name for {}", name)).with_validator(filled_validator).with_default(name).with_help_message("The name Jetty will use to refer to this specific connection. We recommend a single descriptive word.").prompt()?;
    Ok(connector_namespace)
}
