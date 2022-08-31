use crate::{Database, Schema, Table, View};

/// Marker trait for asset types.
#[derive(Clone)]
pub enum Asset {
    Table(Table),
    View(View),
    Schema(Schema),
    Database(Database),
}
