use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::Result;
use colored::Colorize;
use inquire::{Confirm, Text};
use jetty_core::{
    connectors::NewConnector,
    jetty::{ConnectorConfig, ConnectorNamespace, CredentialsMap},
    Connector,
};
use jetty_snowflake::SnowflakeConnector;

use crate::{
    init::{
        inquiry::{
            autocomplete::FilepathCompleter,
            validation::{FilepathValidator, FilepathValidatorMode, PathType},
        },
        pki::create_keypair,
    },
    project::default_keypair_dir_path,
};

use super::validation::filled_validator;

pub(crate) async fn ask_snowflake_connector_setup(
    connector_namespace: ConnectorNamespace,
) -> Result<CredentialsMap> {
    // Loop until a successful connection.
    loop {
        let snowflake_account_id = Text::new("Snowflake Account Identifier:")
            .with_validator(filled_validator)
            .with_placeholder("org-account_name")
            .with_help_message("This is easiest to get in SQL with 'SELECT current_account();'. This field can be the account locator (like 'cea29483') or org account name, dash-separated (like 'MRLDK-ESA98348') See https://tinyurl.com/snow-account-id for more.")
            .prompt()?;

        let admin_username = Text::new("Jetty admin username:")
            .with_default("jetty")
            .with_help_message("We will use this user to authenticate Jetty runs. To see all permissions across your account, the Jetty user needs the SECURITYADMIN role or equivalent.")
            .prompt()?;

        let warehouse = Text::new("Warehouse to query with:")
            .with_help_message("We will use this warehouse for any warehouse-required queries to manage permissions.")
            .prompt()?;

        let default_filepath =
            default_keypair_dir_path().join(format!("~/.ssh/{}.p8", connector_namespace));
        let keypair_answer = Text::new("Input a path to a pkcs8 private key file (`.p8`) to use for authentication or press enter to create a new keypair.")
            .with_help_message("You can attach two keys to each Snowflake user.")
            .with_default(&default_filepath.to_str().unwrap())
            .with_validator(FilepathValidator::new(
                None,
                PathType::File,
                "File not found.".to_string(),
                FilepathValidatorMode::AllowDefault{default_filepath:default_filepath.clone()},
            ))
            .with_autocomplete(FilepathCompleter::default())
            .prompt()?;
        let keypair_filename = keypair_answer
            .split(".p8")
            .next()
            .expect("Couldn't parse pkcs8 private key filename.");

        println!("Generating keypair...");
        let keypair = create_keypair()?;
        keypair.save_to_files(&connector_namespace.to_string(), keypair_filename)?;
        println!("Keypair generated!");

        println!("Authorize Jetty access to your account by running the following SQL statement in Snowflake.");
        println!(
            "\n{}\n",
            format!(
                "ALTER USER {} SET rsa_public_key='{}';",
                admin_username,
                keypair.public_inner()
            )
            .italic()
        );

        let mut confirmed = false;
        while !confirmed {
            confirmed = Confirm::new("Enter (y) once the ALTER USER is complete").prompt()?;
        }

        let creds = HashMap::from([
            ("account".to_owned(), snowflake_account_id),
            ("user".to_owned(), admin_username),
            ("warehouse".to_owned(), warehouse),
            ("public_key_fp".to_owned(), keypair.fingerprint()),
            ("private_key".to_owned(), keypair.private_key()),
            ("role".to_owned(), "SECURITYADMIN".to_owned()),
        ]);
        let connector =
            SnowflakeConnector::new(&ConnectorConfig::default(), &creds, None, None).await?;
        if connector.check().await {
            println!("successful connection!");
            return Ok(creds);
        }
    }
}
