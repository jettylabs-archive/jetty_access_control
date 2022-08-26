use serde::Deserialize;
use structmap::FromMap;
use structmap_derive::FromMap;

/// Snowflake View entry.
#[derive(FromMap, Default, Deserialize, Debug)]
pub struct View {
    /// The view name in Snowflake.
    pub name: String,
    pub schema_name: String,
    pub database_name: String,
}
