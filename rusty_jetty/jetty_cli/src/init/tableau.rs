use std::collections::HashMap;

use anyhow::Result;
use inquire::{Password, PasswordDisplayMode, Text};
use jetty_core::jetty::CredentialsMap;

use super::validation::filled_validator;

pub(crate) fn tableau_connector_setup() -> Result<CredentialsMap> {
    let tableau_server_name = Text::new("Tableau server url:")
        .with_validator(filled_validator)
        .with_placeholder("fs.online.tableau.com")
        .with_help_message("This is the server that hosts your Tableau account.")
        .prompt()?;

    let tableau_site_name = Text::new("Tableau site name:")
        .with_validator(filled_validator)
        .with_placeholder("data_site")
        .with_help_message("This is the site name you want to set permissions for.")
        .prompt()?;

    let tableau_username = Text::new("Tableau username:")
        .with_validator(filled_validator)
        .with_placeholder("elliot@allsafe.com")
        .with_help_message(
            "Your Tableau email username. The associated user must be an account or site admin.",
        )
        .prompt()?;

    let tableau_password = Password::new("Tableau password:")
        .with_display_toggle_enabled()
        .with_display_mode(PasswordDisplayMode::Hidden)
        .with_validator(filled_validator)
        .with_help_message("Your password will be saved locally. Jetty doesn't store passwords. [Ctrl+R] to toggle visibility.")
        .prompt()?;
    Ok(HashMap::from([
        ("server_name".to_owned(), tableau_server_name),
        ("site_name".to_owned(), tableau_site_name),
        ("username".to_owned(), tableau_username),
        ("password".to_owned(), tableau_password),
    ]))
}
