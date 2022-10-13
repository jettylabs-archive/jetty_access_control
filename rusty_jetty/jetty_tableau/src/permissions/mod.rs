//! Tableau permissions models and functionality
//!

pub(crate) mod consts;
mod manager;

use crate::rest::TableauAssetType;
use consts::*;
pub(crate) use manager::PermissionManager;

pub(crate) fn get_capabilities_for_asset_type(
    asset_type: TableauAssetType,
) -> &'static [&'static str] {
    match asset_type {
        TableauAssetType::Workbook => WORKBOOK_CAPABILITIES,
        TableauAssetType::Lens => LENS_CAPABILITIES,
        TableauAssetType::Datasource => DATASOURCE_CAPABILITIES,
        TableauAssetType::Flow => FLOW_CAPABILITIES,
        TableauAssetType::Metric => METRIC_CAPABILITIES,
        TableauAssetType::Project => PROJECT_CAPABILITIES,
        TableauAssetType::View => VIEW_CAPABILITIES,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_capabilities_for_asset_type_works() {
        let capas = get_capabilities_for_asset_type(TableauAssetType::Datasource);
        assert!(!capas.is_empty());
        assert!(!capas[0].is_empty());
    }
}
