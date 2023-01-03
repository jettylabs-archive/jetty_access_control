use anyhow::{bail, Result};
use colored::Colorize;
use inquire::{Password, PasswordDisplayMode, Select, Text};
use jetty_core::jetty::CredentialsMap;
use jetty_tableau::{TableauCredentials, TableauRestClient};

use super::{validation::filled_validator, SKIP_CMD};

pub(crate) async fn ask_tableau_connector_setup() -> Result<CredentialsMap> {
    let skip_message = &format!(
        "{}\n\nTo skip {} setup, enter {}. You can add connectors later by running {}.",
        "".yellow(),
        "Tableau",
        SKIP_CMD.italic().yellow(),
        "jetty add".italic().yellow()
    );

    let mut creds;
    let mut tableau_server_name;
    let mut tableau_site_name;
    let mut tries_left = 3;
    loop {
        tableau_server_name = Text::new("Tableau url:")
            .with_validator(filled_validator)
            .with_placeholder("fs.online.tableau.com")
            .with_help_message(&format!(
                "This is the server that hosts your Tableau instance.{skip_message}",
            ))
            .prompt()?;
        if tableau_server_name == SKIP_CMD {
            bail!("skipped");
        }

        tableau_site_name = Text::new("Tableau site name:")
            .with_validator(filled_validator)
            .with_placeholder("data_site")
            .with_help_message(&format!(
                "This is the site name you want to manage permissions for.{skip_message}",
            ))
            .prompt()?;
        if tableau_site_name == SKIP_CMD {
            bail!("skipped");
        }

        // Choose authentication type (username/password or key)
        let options: Vec<&str> = vec!["Username and Password", "Personal Access Token"];

        let authentication_type =
            Select::new("How would you like to authenticate with Tableau?", options)
                .with_help_message("If you use MFA, you must select Personal Access Token.")
                .prompt()?;

        match authentication_type {
            "Username and Password" => {
                let tableau_username = Text::new("Tableau username:")
                .with_validator(filled_validator)
                .with_placeholder("elliot@allsafe.com")
                .with_help_message(&format!(
                    "Your Tableau email username. Jetty needs credentials to an account with at least Site Administrator Explorer privileges.{skip_message}"),
                )
                .prompt()?;
                if tableau_username == SKIP_CMD {
                    bail!("skipped");
                }

                let tableau_password = Password::new("Tableau password:")
                    .with_display_toggle_enabled()
                    .without_confirmation()
                    .with_display_mode(PasswordDisplayMode::Hidden)
                    .with_validator(filled_validator)
                    .with_help_message(
                        "Your password will only be saved locally. [Ctrl+R] to toggle visibility.",
                    )
                    .prompt()?;

                creds = TableauCredentials::new(
                    jetty_tableau::LoginMethod::UsernameAndPassword {
                        username: tableau_username,
                        password: tableau_password,
                    },
                    tableau_server_name.clone(),
                    tableau_site_name.clone(),
                );
            }
            "Personal Access Token" => {
                let token_name = Text::new("Token Name:")
                .with_validator(filled_validator)
                .with_placeholder("MY_TOKEN")
                .with_help_message(
                    &format!("Jetty needs credentials to an account with at least Site Administrator Explorer privileges. Read about creating a personal access token here: https://help.tableau.com/current/pro/desktop/en-us/useracct.htm#create-and-revoke-personal-access-tokens.{skip_message}"),
                )
                .prompt()?;
                if token_name == SKIP_CMD {
                    bail!("skipped");
                }

                let token_secret = Text::new("Token Secret:")
                .with_validator(filled_validator)
                .with_help_message(
                    &format!("Read about creating a personal access token here: https://help.tableau.com/current/pro/desktop/en-us/useracct.htm#create-and-revoke-personal-access-tokens.{skip_message}"),
                )
                .prompt()?;
                if token_secret == SKIP_CMD {
                    bail!("skipped");
                }

                creds = TableauCredentials::new(
                    jetty_tableau::LoginMethod::PersonalAccessToken {
                        token_name,
                        secret: token_secret,
                    },
                    tableau_server_name.clone(),
                    tableau_site_name.clone(),
                );
            }
            _ => {
                panic!();
            }
        }

        // If the rest client is created successfully, the credentials are valid.
        if TableauRestClient::new(creds.clone()).await.is_ok() {
            break;
        } else if tries_left > 0 {
            println!(
                "{}",
                "Could not connect to Tableau. Please enter your credentials again.".red()
            );
            tries_left -= 1;
        } else {
            println!(
                "{}",
                "Could not connect to Tableau. Please reach out to us at support@get-jetty.com"
                    .red()
            );
            bail!("skipped");
        }
    }
    Ok(creds.to_map())
}
