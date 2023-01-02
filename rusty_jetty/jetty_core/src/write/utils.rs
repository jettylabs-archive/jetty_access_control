//! write path utility functions

use std::collections::{BTreeSet, HashSet};
use std::hash::Hash;

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
pub(crate) fn error_vec_to_string(errors: &[String]) -> String {
    errors
        .iter()
        .map(|e| format!("{}", format!(" - {e}").as_str().red()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Take config and environment sets and return what elements need to be added to the environment
/// and what elements need to be removed
#[allow(clippy::type_complexity)]
pub(crate) fn diff_hashset<'a, T>(
    config: &'a HashSet<T>,
    env: &'a HashSet<T>,
) -> (
    std::iter::Cloned<
        std::collections::hash_set::Difference<'a, T, std::collections::hash_map::RandomState>,
    >,
    std::iter::Cloned<
        std::collections::hash_set::Difference<'a, T, std::collections::hash_map::RandomState>,
    >,
)
where
    T: Hash + std::clone::Clone,
    T: std::cmp::Eq,
{
    let add = config.difference(env).cloned();
    let remove = env.difference(config).cloned();

    (add, remove)
}

/// Take config and environment sets and return what elements need to be added to the environment
/// and what elements need to be removed
pub(crate) fn diff_btreeset<'a, T>(
    config: &'a BTreeSet<T>,
    env: &'a BTreeSet<T>,
) -> (
    std::iter::Cloned<std::collections::btree_set::Difference<'a, T>>,
    std::iter::Cloned<std::collections::btree_set::Difference<'a, T>>,
)
where
    T: Ord + std::clone::Clone,
    T: std::cmp::Eq,
{
    let add = config.difference(env).cloned();
    let remove = env.difference(config).cloned();

    (add, remove)
}
