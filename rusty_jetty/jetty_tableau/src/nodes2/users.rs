use std::collections::HashMap;

use anyhow::{Context, Result};

pub(crate) fn to_node(val: &serde_json::Value) -> Result<super::User> {
    serde_json::from_value(val.to_owned()).context("parsing user information")
}
