use anyhow::Result;
use inquire::validator::Validation;

pub(crate) fn filled_validator(
    input: &str,
) -> Result<Validation, Box<dyn std::error::Error + Send + Sync + 'static>> {
    if input.is_empty() {
        Ok(Validation::Invalid(
            "Please enter an answer.".into(),
        ))
    } else {
        Ok(Validation::Valid)
    }
}
