pub const AUTH_HEADER: &str = "Authorization";
pub const CONTENT_TYPE_HEADER: &str = "Content-Type";
pub const ACCEPT_HEADER: &str = "Accept";
pub const SNOWFLAKE_AUTH_HEADER: &str = "X-Snowflake-Authorization-Token-Type";
pub const USER_AGENT_HEADER: &str = "User-Agent";

/// Valid asset types for Snowflake.
///
/// Ignored types here:
/// ACCOUNT, FUNCTION, WAREHOUSE: These are TODOs for a future iteration.
/// ROLE: We don't need children groups. Those relationships will be taken care of
/// as parent roles.
pub const ASSET_TYPES: [&str; 4] = ["TABLE", "VIEW", "SCHEMA", "DATABASE"];

pub const DATABASE: &str = "DATABASE";
pub const SCHEMA: &str = "SCHEMA";
pub const VIEW: &str = "VIEW";
pub const TABLE: &str = "TABLE";
