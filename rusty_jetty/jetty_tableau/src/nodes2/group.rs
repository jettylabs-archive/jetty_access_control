use anyhow::{Context, Result};
use serde::Deserialize;

fn to_node(val: &serde_json::Value) -> Result<super::Group> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct GroupInfo {
        name: String,
        id: String,
    }
    let group_info: GroupInfo =
        serde_json::from_value(val.to_owned()).context("parsing group information")?;

    Ok(super::Group {
        id: group_info.id,
        name: group_info.name,
        includes: Vec::new(),
    })
}
