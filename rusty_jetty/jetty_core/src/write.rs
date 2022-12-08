//! Write user-configured groups and permissions back to the data stack.

pub mod groups;
mod parser_common;
mod policies;
pub(crate) mod tag_parser;

pub use groups::get_group_diff;

/// A collection of diffs
pub struct Diffs {
    /// All the group-level diffs
    pub groups: Vec<groups::Diff>,
}
