pub(crate) const WORKBOOK_CAPABILITIES: &'static [&'static str] = &[
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

pub(crate) const LENS_CAPABILITIES: &'static [&'static str] = &[
    "ChangeHierarchy",
    "Delete",
    "Write",
    "Read",
    "ChangePermissions",
];

pub(crate) const DATASOURCE_CAPABILITIES: &'static [&'static str] = &[
    "Connect",
    "ChangeHierarchy",
    "SaveAs",
    "Delete",
    "Write",
    "Read",
    "ExportXml",
    "ChangePermissions",
];

pub(crate) const FLOW_CAPABILITIES: &'static [&'static str] = &[
    "ChangeHierarchy",
    "Delete",
    "WebAuthoringForFlows",
    "Read",
    "ChangePermissions",
    "Execute",
    "Write",
    "ExportXml",
];

pub(crate) const METRIC_CAPABILITIES: &'static [&'static str] = &[
    "ChangeHierarchy",
    "Delete",
    "Write",
    "Read",
    "ChangePermissions",
];

pub(crate) const PROJECT_CAPABILITIES: &'static [&'static str] =
    &["Read", "Write", "ProjectLeader"];

pub(crate) const VIEW_CAPABILITIES: &'static [&'static str] = &[
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
