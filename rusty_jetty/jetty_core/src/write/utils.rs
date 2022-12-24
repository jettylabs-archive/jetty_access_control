//! write path utility functions

use colored::Colorize;

/// This cleans out illegal characters so that asset names can be used in paths
pub(crate) fn clean_string_for_path(val: String) -> String {
    // Remove illegal characters
    let no_stinky_chars = val
        .split(
            &[
                '/', '\\', '?', '|', '<', '>', ':', '*', '"', '+', ',', ';', '=', '[', ']',
            ][..],
        )
        .collect::<Vec<_>>()
        .join("_");
    // Can't end in a period
    if no_stinky_chars.ends_with('.') {
        format!("{no_stinky_chars}_")
    } else {
        no_stinky_chars
    }
}

/// Turn a Vec<String> of errors into a single formatted string
pub(crate) fn error_vec_to_string(errors: &Vec<String>) -> String {
    errors
        .into_iter()
        .map(|e| format!("{}", format!(" - {e}").as_str().red()))
        .collect::<Vec<_>>()
        .join("\n")
}
