use anyhow::Result;
use colored::Colorize;
use inquire::Text;

use super::validation::filled_validator;

pub(crate) fn dbt_connector_setup() -> Result<()> {
    println!(
        "{}",
        "Note: dbt only offers Snowflake support in this version".red()
    );
    let dbt_project_dir = Text::new("dbt project directory:")
        .with_validator(filled_validator)
        .with_placeholder("/path/to/dbt/project")
        .with_help_message(&format!(
            "This will be the directory with {}",
            "dbt_project.yml".italic()
        ))
        .prompt()?;

    // TODO: Verify the project exists.
    Ok(())
}
