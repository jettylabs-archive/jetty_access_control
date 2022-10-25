use core::time;
use std::thread;

use anyhow::Result;
use colored::Colorize;
use inquire::{Confirm, Text};

use super::validation::filled_validator;

pub(crate) fn snowflake_connector_setup() -> Result<()> {
    let snowflake_account_id = Text::new("Snowflake Account Identifier:")
        .with_validator(filled_validator)
        .with_placeholder("org-account_name")
        .with_help_message("You can find your account ID on the bottom left of the Snowflake UI. See https://tinyurl.com/snow-account-id for more.")
        .prompt()?;

    let keypair_dir = Text::new("Keypair directory:")
        .with_default("~/.ssh")
        .with_help_message(
            "We will put your public and private keys in this local directory for safekeeping.",
        )
        .prompt()?;

    println!("Generating keypair...");
    // TODO: Actually do this
    let one_second = time::Duration::from_millis(1000);
    thread::sleep(one_second);

    let admin_username = Text::new("Jetty admin username:")
    .with_default("jetty")
        .with_help_message("We will use this user to authenticate Jetty runs. To see all permissions across your account, the Jetty user needs the SECURITYADMIN role or equivalent.")
        .prompt()?;

    println!("Authorize Jetty access to your account by copying the following SQL statement into Snowflake.");
    println!(
        "\n{}\n",
        format!(
            "ALTER USER {} SET rsa_public_key={}",
            admin_username, "asdlfkjasvnaerstoiutnboi"
        )
        .italic()
    );

    let confirmed = Confirm::new("Enter (y) once the ALTER USER is complete").prompt()?;

    // TODO: Check connection
    Ok(())
}
