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

impl Asset {
    pub(crate) fn fqn(&self) -> String {
        match self {
            Asset::Database(d) => d.name.to_owned(),
            Asset::Schema(s) => s.fqn(),
            Asset::Object(o) => o.fqn(),
            Asset::Unknown => panic!("unknown asset type"),
        }
    }
}
