use serde::Deserialize;

/// Credentials for authenticating to Snowflake.
///
/// The user sets these up by following Jetty documentation
/// and pasting their keys into their connector config.
#[derive(Deserialize, Debug, Default)]
pub(crate) struct SnowflakeCredentials {
    pub(crate) account: String,
    pub(crate) password: String,
    pub(crate) role: String,
    pub(crate) user: String,
    pub(crate) warehouse: String,
    pub(crate) private_key: String,
    pub(crate) public_key_fp: String,
    pub(crate) url: Option<String>,
}
