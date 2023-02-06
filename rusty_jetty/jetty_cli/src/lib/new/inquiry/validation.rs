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
            "A directory called {input} already exists. Choose a different project name, or run jetty \
            with the -o flag to overwrite."
        )
            .into(),
        ))
    } else {
        Ok(Validation::Valid)
    }
}

#[derive(Clone, Debug)]
pub(crate) enum PathType {
    File,
    #[allow(dead_code)]
    Dir,
}

#[derive(Clone, Debug)]
pub(crate) enum FilepathValidatorMode {
    #[allow(dead_code)]
    /// Only allow existing paths.
    Strict,
    /// Allow empty paths to be replaced with a defualt.
    AllowedValues { allowed_values: Vec<String> },
}

#[derive(Clone, Debug)]
pub(crate) struct FilepathValidator {
    filename: Option<String>,
    msg: String,
    path_type: PathType,
    mode: FilepathValidatorMode,
}

impl FilepathValidator {
    pub(crate) fn new(
        filename: Option<String>,
        path_type: PathType,
        msg: String,
        mode: FilepathValidatorMode,
    ) -> Self {
        Self {
            filename,
            msg,
            path_type,
            mode,
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

        let condition = match (&self.mode, &self.path_type) {
            (FilepathValidatorMode::Strict, PathType::File) => path.is_file(),
            (FilepathValidatorMode::Strict, PathType::Dir) => path.is_dir(),
            (FilepathValidatorMode::AllowedValues { allowed_values }, PathType::File) => {
                allowed_values.contains(&input.to_owned()) || path.is_file()
            }
            (FilepathValidatorMode::AllowedValues { allowed_values }, PathType::Dir) => {
                allowed_values.contains(&input.to_owned()) || path.is_dir()
            }
        };

        if !condition {
            Ok(Validation::Invalid(self.msg.clone().into()))
        } else {
            Ok(Validation::Valid)
        }
    }
}
