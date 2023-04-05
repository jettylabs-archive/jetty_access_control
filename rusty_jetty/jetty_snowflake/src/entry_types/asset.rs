use serde::Serialize;

use crate::{Database, Object, Schema};

/// Marker trait for asset types.
#[derive(Clone, Serialize, Default)]
pub enum Asset {
    Schema(Schema),
    Database(Database),
    Object(Object),
    #[default]
    Unknown,
}
