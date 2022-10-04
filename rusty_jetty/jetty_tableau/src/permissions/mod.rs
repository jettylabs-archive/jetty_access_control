//! Tableau permissions models and functionality
//!

use crate::rest::TableauAssetType;

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

const WORKBOOK_CAPABILITIES: &'static [&'static str] = &[
    "ViewComments",
    "Filter",
    "CreateRefreshMetrics",
    "ChangePermissions",
    "Read",
    "Write",
    "Delete",
    "ShareView",
    "ChangeHierarchy",
    "RunExplainData",
    "ExportData",
    "ExportXml",
    "WebAuthoring",
    "ViewUnderlyingData",
    "AddComment",
    "ExportImage",
];

const LENS_CAPABILITIES: &'static [&'static str] = &[
    "ChangeHierarchy",
    "Delete",
    "Write",
    "Read",
    "ChangePermissions",
];

const DATASOURCE_CAPABILITIES: &'static [&'static str] = &[
    "Connect",
    "ChangeHierarchy",
    "SaveAs",
    "Delete",
    "Write",
    "Read",
    "ExportXml",
    "ChangePermissions",
];

const FLOW_CAPABILITIES: &'static [&'static str] = &[
    "ChangeHierarchy",
    "Delete",
    "WebAuthoringForFlows",
    "Read",
    "ChangePermissions",
    "Execute",
    "Write",
    "ExportXml",
];

const METRIC_CAPABILITIES: &'static [&'static str] = &[
    "ChangeHierarchy",
    "Delete",
    "Write",
    "Read",
    "ChangePermissions",
];

const PROJECT_CAPABILITIES: &'static [&'static str] = &["Read", "Write", "ProjectLeader"];

const VIEW_CAPABILITIES: &'static [&'static str] = &[
    "ViewComments",
    "Filter",
    "ChangePermissions",
    "Read",
    "Delete",
    "ShareView",
    "ExportData",
    "WebAuthoring",
    "ViewUnderlyingData",
    "AddComment",
    "ExportImage",
];
