//! Diffing for user configurations <-> Env

mod identity;
mod membership;

pub use identity::{get_identity_diffs, update_graph};
