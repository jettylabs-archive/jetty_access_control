use std::collections::HashMap;

use crate::{nodes::user::SiteRole, rest::TableauAssetType};

pub(crate) const WORKBOOK_CAPABILITIES: &[&str] = &[
    "ViewComments",         // View Comments
    "Filter",               // Filter
    "CreateRefreshMetrics", // Create/Refresh Metrics
    "ChangePermissions",    // Set Permissions
    "Read",                 // View
    "Write",                // Overwrite
    "Delete",               // Delete
    "ShareView",            // Share Customized
    "ChangeHierarchy",      // Move
    "RunExplainData",       // Run Explain Data
    "ExportData",           // Download Summary Data
    "ExportXml",            // Download Workbook/Save a Copy
    "WebAuthoring",         // Web Edit
    "ViewUnderlyingData",   // Download Full Data
    "AddComment",           // Add Comments
    "ExportImage",          // Download Image/PDF
];

pub(crate) const LENS_CAPABILITIES: &[&str] = &[
    "ChangeHierarchy",   // Move
    "Delete",            // Delete
    "Write",             // Overwrite
    "Read",              // View
    "ChangePermissions", // Set Permissions
];

pub(crate) const DATASOURCE_CAPABILITIES: &[&str] = &[
    "Connect",           // Connect
    "ChangeHierarchy",   // Move
    "SaveAs",            // Save As
    "Delete",            // Delete
    "Write",             // Overwrite
    "Read",              // View
    "ExportXml",         // Download Data Source
    "ChangePermissions", // Set Permissions
];

pub(crate) const FLOW_CAPABILITIES: &[&str] = &[
    "ChangeHierarchy",      // Move
    "Delete",               // Delete
    "WebAuthoringForFlows", // Web Edit
    "Read",                 // View
    "ChangePermissions",    // Set Permissions
    "Execute",              // Run Flow
    "Write",                // Overwrite
    "ExportXml",            // Download Flow
];

pub(crate) const METRIC_CAPABILITIES: &[&str] = &[
    "ChangeHierarchy",   // Move
    "Delete",            // Delete
    "Write",             // Overwrite
    "Read",              // View
    "ChangePermissions", // Set Permissions
];

pub(crate) const PROJECT_CAPABILITIES: &[&str] = &[
    "Read",          // View
    "Write",         // Publish
    "ProjectLeader", // SPECIAL (not a real capability)
];

pub(crate) const VIEW_CAPABILITIES: &[&str] = &[
    "ViewComments",       // View Comments
    "Filter",             // Filter
    "ChangePermissions",  // Set Permissions
    "Read",               // View
    "Delete",             // Delete
    "ShareView",          // Share Customized
    "ExportData",         // Download Summary Data
    "WebAuthoring",       // Web Edit
    "ViewUnderlyingData", // Download Full Data
    "AddComment",         // Add Comments
    "ExportImage",        // Download Image/PDF
];

pub(crate) struct AssetCapabilityMap<'a> {
    map: HashMap<SiteRole, HashMap<TableauAssetType, Vec<&'a str>>>,
}

impl<'a> AssetCapabilityMap<'a> {
    pub(crate) fn new() -> Self {
        let no_restrictions_map = HashMap::from([
            (TableauAssetType::Project, vec![]),
            (TableauAssetType::Workbook, vec![]),
            (TableauAssetType::Datasource, vec![]),
            (TableauAssetType::Flow, vec![]),
            (TableauAssetType::Lens, vec![]),
            (TableauAssetType::Metric, vec![]),
            (TableauAssetType::View, vec![]),
        ]);
        let all_restrictions_map = HashMap::from([
            (TableauAssetType::Project, PROJECT_CAPABILITIES.to_vec()),
            (TableauAssetType::Workbook, WORKBOOK_CAPABILITIES.to_vec()),
            (
                TableauAssetType::Datasource,
                DATASOURCE_CAPABILITIES.to_vec(),
            ),
            (TableauAssetType::Flow, FLOW_CAPABILITIES.to_vec()),
            (TableauAssetType::Lens, LENS_CAPABILITIES.to_vec()),
            (TableauAssetType::Metric, METRIC_CAPABILITIES.to_vec()),
            (TableauAssetType::View, VIEW_CAPABILITIES.to_vec()),
        ]);
        let explorer_map = HashMap::from([
            (TableauAssetType::Project, vec!["Write"]),
            (
                TableauAssetType::Workbook,
                vec![
                    "Write",
                    "CreateRefreshMetrics",
                    // Explorers can be granted move (ChangeHierarchy) but it does
                    // nothing. See https://tinyurl.com/workbook-move-capa
                    "ChangeHierarchy",
                    "Delete",
                    "ChangePermissions",
                ],
            ),
            (
                TableauAssetType::Datasource,
                vec!["Write", "Delete", "ChangePermissions"],
            ),
            (
                TableauAssetType::Flow,
                vec![
                    "Execute",
                    "Write",
                    // Explorers can be granted move (ChangeHierarchy) but it does
                    // nothing. See https://tinyurl.com/workbook-move-capa
                    "ChangeHierarchy",
                    "Delete",
                    "ChangePermissions",
                ],
            ),
            (
                TableauAssetType::Lens,
                vec![
                    "Write",
                    // Explorers can be granted move (ChangeHierarchy) but it does
                    // nothing. See https://tinyurl.com/workbook-move-capa
                    "ChangeHierarchy",
                    "Delete",
                    "ChangePermissions",
                ],
            ),
            (
                TableauAssetType::Metric,
                vec![
                    "Write",
                    // Explorers can be granted move (ChangeHierarchy) but it does
                    // nothing. See https://tinyurl.com/workbook-move-capa
                    "ChangeHeirarchy",
                    "Delete",
                    "ChangePermissions",
                ],
            ),
            (TableauAssetType::View, vec!["Delete", "ChangePermissions"]),
        ]);
        let viewer_map = HashMap::from([
            (TableauAssetType::Project, vec!["Write"]),
            (
                TableauAssetType::Workbook,
                vec![
                    "ShareView",
                    "ViewUnderlyingData",
                    "WebAuthoring",
                    "ExportXml",
                    "Write",
                    "CreateRefreshMetrics",
                    "ChangeHierarchy",
                    "Delete",
                    "ChangePermissions",
                ],
            ),
            (
                TableauAssetType::Datasource,
                vec!["ExportXml", "Write", "Delete", "ChangePermissions"],
            ),
            (
                TableauAssetType::Flow,
                vec![
                    "ExportXml",
                    "WebAuthoringForFlows",
                    "Execute",
                    "Write",
                    "ChangeHierarchy",
                    "Delete",
                    "ChangePermissions",
                ],
            ),
            (
                TableauAssetType::Lens,
                vec!["Write", "ChangeHierarchy", "Delete", "ChangePermissions"],
            ),
            (
                TableauAssetType::Metric,
                vec!["ChangeHierarchy", "Delete", "Write", "ChangePermissions"],
            ),
            (
                TableauAssetType::View,
                vec![
                    "ShareView",
                    "ViewUnderlyingData",
                    "WebAuthoring",
                    "Delete",
                    "ChangePermissions",
                ],
            ),
        ]);

        Self {
            map: HashMap::from([
                (SiteRole::ServerAdministrator, no_restrictions_map.clone()),
                (
                    SiteRole::SiteAdministratorCreator,
                    no_restrictions_map.clone(),
                ),
                (SiteRole::Creator, no_restrictions_map.clone()),
                (SiteRole::ExplorerCanPublish, no_restrictions_map.clone()),
                (SiteRole::SiteAdministratorExplorer, explorer_map.clone()),
                (SiteRole::Explorer, explorer_map),
                (
                    // ReadOnly is a precursor to Viewer.
                    // See https://tinyurl.com/read-only-site-role
                    SiteRole::ReadOnly,
                    viewer_map.clone(),
                ),
                (SiteRole::Viewer, viewer_map),
                (SiteRole::Unlicensed, all_restrictions_map.clone()),
                (SiteRole::Unknown, all_restrictions_map),
            ]),
        }
    }

    pub(crate) fn get<'b>(
        &'b self,
        site_role: SiteRole,
        asset_type: TableauAssetType,
    ) -> Option<&'b Vec<&'b str>> {
        self.map
            .get(&site_role)
            .and_then(|role_restrictions| role_restrictions.get(&asset_type))
    }
}
