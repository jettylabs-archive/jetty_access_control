use std::collections::HashMap;

use anyhow::Result;
use colored::Colorize;
use inquire::Text;
use jetty_core::jetty::CredentialsMap;

use crate::init::inquiry::{
    autocomplete::FilepathCompleter,
    validation::{FilepathValidator, PathType},
};

use super::validation::filled_validator;

pub(crate) fn ask_dbt_connector_setup() -> Result<CredentialsMap> {
    println!(
        "{}",
        "Note: dbt only offers Snowflake support in this version".yellow()
    );
    let dbt_project_dir = Text::new("dbt project directory:")
        // Validate that they entered something.
        .with_validator(filled_validator)
        // Validate that this is a project.
        .with_validator(FilepathValidator::new(
            Some("dbt_project.yml".to_owned()),
            PathType::File,
            "Please enter a valid dbt project path (with dbt_project.yml)".to_owned(),
        ))
        // Validate that the manifest has been compiled.
        .with_validator(FilepathValidator::new(
            Some("target/manifest.json".to_owned()),
            PathType::File,
            "target/manifest.json not found. Please run 'dbt docs generate' in the directory to generate it and then try again.".to_string()
            ,
        ))
        .with_placeholder("/path/to/dbt/project")
        .with_autocomplete(FilepathCompleter::default())
        .with_help_message(&format!(
            "This will be the directory with {}",
            "dbt_project.yml".italic()
        ))
        .prompt()?;

    Ok(HashMap::from([("project_dir".to_owned(), dbt_project_dir)]))
}
