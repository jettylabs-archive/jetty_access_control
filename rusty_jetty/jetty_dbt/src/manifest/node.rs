use serde::Deserialize;

use jetty_core::connectors::AssetType;

use std::collections::HashMap;

/// A node within Dbt, representing either a model
/// or a source.
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) enum DbtNode {
    ModelNode(DbtModelNode),
    SourceNode(DbtSourceNode),
}

/// A node within Dbt that represents a data source.
#[derive(Clone, Deserialize, PartialEq, Eq, Hash)]
pub(crate) struct DbtSourceNode {
    pub(crate) name: String,
    pub(crate) database: String,
    pub(crate) schema: String,
}

/// A node within Dbt that represents a model.
#[derive(Clone, PartialEq, Eq, Hash, Default)]
pub(crate) struct DbtModelNode {
    pub(crate) name: String,
    pub(crate) enabled: bool,
    pub(crate) database: String,
    pub(crate) schema: String,
    pub(crate) materialized_as: AssetType,
}

impl DbtModelNode {
    pub(crate) fn get_metadata(&self) -> HashMap<String, String> {
        HashMap::from([("enabled".to_owned(), self.enabled.to_string())])
    }
}
