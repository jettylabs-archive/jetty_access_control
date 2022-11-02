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

pub(crate) fn project_dir_does_not_exist_validator(
    input: &str,
) -> Result<Validation, CustomUserError> {
    if Path::new(input).is_dir() {
        Ok(Validation::Invalid(
            format!(
            "A directory called {} already exists. Choose a different project name, or run jetty \
            with the -o flag to overwrite.",
            input
        )
            .into(),
        ))
    } else {
        Ok(Validation::Valid)
    }
}

#[derive(Clone)]
pub(crate) enum PathType {
    File,
    Dir,
}

#[derive(Clone)]
pub(crate) struct FilepathValidator {
    filename: Option<String>,
    msg: String,
    path_type: PathType,
}

impl FilepathValidator {
    pub(crate) fn new(filename: Option<String>, path_type: PathType, msg: String) -> Self {
        Self {
            filename,
            msg,
            path_type,
        }
    }
}

impl StringValidator for FilepathValidator {
    fn validate(&self, input: &str) -> Result<Validation, CustomUserError> {
        let path = if let Some(filename) = self.filename.clone() {
            Path::new(input).join(filename)
        } else {
            Path::new(input).to_path_buf()
        };

        let condition = match self.path_type {
            PathType::File => path.is_file(),
            PathType::Dir => path.is_dir(),
        };

        if !condition {
            Ok(Validation::Invalid(self.msg.clone().into()))
        } else {
            Ok(Validation::Valid)
        }
    }
}
