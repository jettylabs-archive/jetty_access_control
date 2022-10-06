pub(crate) const WORKBOOK_CAPABILITIES: &[&str] = &[
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

pub(crate) const LENS_CAPABILITIES: &[&str] = &[
    "ChangeHierarchy",
    "Delete",
    "Write",
    "Read",
    "ChangePermissions",
];

pub(crate) const DATASOURCE_CAPABILITIES: &[&str] = &[
    "Connect",
    "ChangeHierarchy",
    "SaveAs",
    "Delete",
    "Write",
    "Read",
    "ExportXml",
    "ChangePermissions",
];

pub(crate) const FLOW_CAPABILITIES: &[&str] = &[
    "ChangeHierarchy",
    "Delete",
    "WebAuthoringForFlows",
    "Read",
    "ChangePermissions",
    "Execute",
    "Write",
    "ExportXml",
];

pub(crate) const METRIC_CAPABILITIES: &[&str] = &[
    "ChangeHierarchy",
    "Delete",
    "Write",
    "Read",
    "ChangePermissions",
];

pub(crate) const PROJECT_CAPABILITIES: &[&str] =
    &["Read", "Write", "ProjectLeader"];

pub(crate) const VIEW_CAPABILITIES: &[&str] = &[
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
