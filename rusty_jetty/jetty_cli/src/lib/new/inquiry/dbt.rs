use std::{collections::HashMap, path::PathBuf};

use anyhow::{bail, Result};
use colored::Colorize;
use inquire::Text;
use jetty_core::jetty::CredentialsMap;

use crate::new::inquiry::{
    autocomplete::FilepathCompleter,
    validation::{FilepathValidator, FilepathValidatorMode, PathType},
    SKIP_CMD,
};

use super::validation::filled_validator;

pub(crate) fn ask_dbt_connector_setup() -> Result<CredentialsMap> {
    let skip_message = &format!(
        "{}\n\nTo skip {} setup, enter {}. You can add connectors later by running {}.",
        "".yellow(),
        "dbt",
        SKIP_CMD.italic().yellow(),
        "jetty add".italic().yellow()
    );

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
            FilepathValidatorMode::AllowedValues { allowed_values: vec![SKIP_CMD.to_owned()] },
        ))
        // Validate that the manifest has been compiled.
        .with_validator(FilepathValidator::new(
            Some("target/manifest.json".to_owned()),
            PathType::File,
            "target/manifest.json not found. Please run 'dbt docs generate' in the directory to generate it and then try again.".to_string()
            ,
            FilepathValidatorMode::AllowedValues { allowed_values: vec![SKIP_CMD.to_owned()] },
        ))
        .with_placeholder("/path/to/dbt/project")
        .with_autocomplete(FilepathCompleter::default())
        .with_help_message(&format!(
            "This will be the directory with {}{skip_message}",
            "dbt_project.yml".italic(),
        ))
        .prompt()?;

    if dbt_project_dir == SKIP_CMD {
        bail!("skipped");
    }

    let snowflake_account_id = Text::new("Account Identifier of Snowflake Account Used with dbt:")
        .with_validator(filled_validator)
        .with_placeholder("org-account_name")
        .with_help_message(&format!("This field can be the account locator (like 'cea29483' or 'cea29483.us-east-1') or org account name, dash-separated (like 'MRLDK-ESA98348') See https://tinyurl.com/snow-account-id for more.{skip_message}"))
        .prompt()?;

    if snowflake_account_id == SKIP_CMD {
        bail!("skipped");
    }

    // Get the full path to the dbt project directory. Safe to unwrap because the path
    // is validated by the validator above.
    let canonical_dbt_project_dir = PathBuf::from(dbt_project_dir)
        .canonicalize()
        .unwrap()
        .to_string_lossy()
        .to_string();

    Ok(HashMap::from([
        ("project_dir".to_owned(), canonical_dbt_project_dir),
        ("snowflake_account".to_owned(), snowflake_account_id),
    ]))
}
