use std::{collections::HashMap, path::Path};

use anyhow::Result;
use colored::Colorize;
use inquire::{Confirm, Password, PasswordDisplayMode, Text};
use jetty_core::{
    jetty::{ConnectorConfig, CredentialsMap},
    Connector,
};
use jetty_snowflake::SnowflakeConnector;

use crate::init::{
    autocomplete::FilepathCompleter,
    pki::create_keypair,
    validation::{FilepathValidator, PathType},
};

use super::validation::filled_validator;

pub(crate) async fn ask_snowflake_connector_setup() -> Result<CredentialsMap> {
    loop {
        let snowflake_account_id = Text::new("Snowflake Account Identifier:")
        .with_validator(filled_validator)
        .with_placeholder("org-account_name")
        .with_help_message("You can find your account ID on the bottom left of the Snowflake UI. See https://tinyurl.com/snow-account-id for more.")
        .prompt()?;

        let admin_username = Text::new("Jetty admin username:")
            .with_default("jetty")
            .with_help_message("We will use this user to authenticate Jetty runs. To see all permissions across your account, the Jetty user needs the SECURITYADMIN role or equivalent.")
            .prompt()?;

        let warehouse = Text::new("Warehouse to query with:")
            .with_help_message("We will use this warehouse for any warehouse-required queries to manage permissions.")
            .prompt()?;

        let keypair_dir = Text::new("Keypair directory:")
            .with_validator(FilepathValidator::new(
                None,
                PathType::Dir,
                "Please enter a valid directory.".to_owned(),
            ))
            .with_autocomplete(FilepathCompleter::default())
            .with_default("~/.ssh")
            .with_help_message(
                "We will put your public and private keys in this local directory for safekeeping.",
            )
            .prompt()?;
        let keypair_dir_path = Path::new(&keypair_dir);

        println!("Generating keypair...");
        let keypair = create_keypair()?;
        keypair.save_to_files(keypair_dir_path)?;
        println!("Keypair generated!");

        println!("Authorize Jetty access to your account by copying the following SQL statement into Snowflake.");
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
        while confirmed == false {
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
        let connector = SnowflakeConnector::new(&ConnectorConfig::default(), &creds, None).await?;
        if connector.check().await {
            println!("successful connection!");
            return Ok(creds);
        }
    }
}
