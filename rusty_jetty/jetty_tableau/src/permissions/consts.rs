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
    "Read",                   // View
    "Write",                  // Publish
    "ProjectLeader",          // Project Leader
    "InheritedProjectLeader", // SPECIAL (not a real capability)
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
