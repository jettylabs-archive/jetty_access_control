use std::collections::HashMap;

use anyhow::Result;
use colored::Colorize;
use inquire::{Password, PasswordDisplayMode, Text};
use jetty_core::jetty::CredentialsMap;
use jetty_tableau::{TableauCredentials, TableauRestClient};

use super::validation::filled_validator;

pub(crate) async fn ask_tableau_connector_setup() -> Result<CredentialsMap> {
    let mut tableau_server_name;
    let mut tableau_site_name;
    let mut tableau_username;
    let mut tableau_password;
    let mut tries_left = 3;
    loop {
        tableau_server_name = Text::new("Tableau url:")
            .with_validator(filled_validator)
            .with_placeholder("fs.online.tableau.com")
            .with_help_message("This is the server that hosts your Tableau instance.")
            .prompt()?;

        tableau_site_name = Text::new("Tableau site name:")
            .with_validator(filled_validator)
            .with_placeholder("data_site")
            .with_help_message("This is the site name you want to set permissions for.")
            .prompt()?;

        tableau_username = Text::new("Tableau username:")
        .with_validator(filled_validator)
        .with_placeholder("elliot@allsafe.com")
        .with_help_message(
            "Your Tableau email username. The associated user must be an account or site admin.",
        )
        .prompt()?;

        tableau_password = Password::new("Tableau password:")
            .with_display_toggle_enabled()
            .without_confirmation()
            .with_display_mode(PasswordDisplayMode::Hidden)
            .with_validator(filled_validator)
            .with_help_message(
                "Your password will only be saved locally. [Ctrl+R] to toggle visibility.",
            )
            .prompt()?;
        // TODO: Verify connnection
        let creds = TableauCredentials::new(
            tableau_username.clone(),
            tableau_password.clone(),
            tableau_server_name.clone(),
            tableau_site_name.clone(),
        );
        // If the rest client is created successfully, the credentials are valid.
        if TableauRestClient::new(creds).await.is_ok() {
            break;
        } else if tries_left > 0 {
            println!(
                "{}",
                "Could not connect to Tableau. Please enter your credentials again.".red()
            );
            tries_left -= 1;
        } else {
            panic!(
                "{}",
                "Could not connect to Tableau. Please reach out to us at support@get-jetty.com"
                    .red()
            );
        }
    }
    Ok(HashMap::from([
        ("server_name".to_owned(), tableau_server_name),
        ("site_name".to_owned(), tableau_site_name),
        ("username".to_owned(), tableau_username),
        ("password".to_owned(), tableau_password),
    ]))
}
