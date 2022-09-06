use std::collections::HashMap;

use crate::rest::{self, FetchJson};
use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Clone, Debug)]
pub(crate) struct Group {
    pub id: String,
    pub name: String,
    pub includes: Vec<String>,
}

pub(crate) fn to_node(val: &serde_json::Value) -> Result<Group> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct GroupInfo {
        name: String,
        id: String,
    }
    let group_info: GroupInfo =
        serde_json::from_value(val.to_owned()).context("parsing group information")?;

    Ok(Group {
        id: group_info.id,
        name: group_info.name,
        includes: Vec::new(),
    })
}
