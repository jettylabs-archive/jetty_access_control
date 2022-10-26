use std::path::Path;

use anyhow::Result;
use inquire::{
    validator::{StringValidator, Validation},
    CustomUserError,
};

pub(crate) fn filled_validator(input: &str) -> Result<Validation, CustomUserError> {
    if input.is_empty() {
        Ok(Validation::Invalid("Please enter an answer.".into()))
    } else {
        Ok(Validation::Valid)
    }
}

#[derive(Clone)]
pub(crate) struct FilepathValidator {
    filename: String,
    msg: String,
}

impl FilepathValidator {
    pub(crate) fn new(filename: String, msg: String) -> Self {
        Self { filename, msg }
    }
}

impl StringValidator for FilepathValidator {
    fn validate(&self, input: &str) -> Result<Validation, CustomUserError> {
        if !Path::new(input).join(self.filename.clone()).is_file() {
            Ok(Validation::Invalid(self.msg.clone().into()))
        } else {
            Ok(Validation::Valid)
        }
    }
}
