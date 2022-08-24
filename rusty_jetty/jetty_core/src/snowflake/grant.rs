use serde::Deserialize;
use structmap::FromMap;
use structmap_derive::FromMap;

/// Snowflake Grant entry.
#[derive(Deserialize, Debug)]
pub struct Grant {}
