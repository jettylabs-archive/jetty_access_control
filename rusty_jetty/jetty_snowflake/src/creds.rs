use anyhow::{anyhow, Result};
use serde::Deserialize;

/// Credentials for authenticating to Snowflake.
///
/// The user sets these up by following Jetty documentation
/// and pasting their keys into their connector config.
#[derive(Deserialize, Debug, Default)]
pub(crate) struct SnowflakeCredentials {
    pub(crate) account: String,
    pub(crate) role: String,
    pub(crate) user: String,
    pub(crate) warehouse: String,
    pub(crate) private_key: String,
    pub(crate) public_key_fp: String,
    pub(crate) url: Option<String>,
}

impl SnowflakeCredentials {
    /// Perform simple field validation to catch bad input.
    pub(crate) fn validate(&self) -> Result<()> {
        if self.account.is_empty()
            || self.role.is_empty()
            || self.user.is_empty()
            || self.warehouse.is_empty()
            || self.private_key.is_empty()
            || self.public_key_fp.is_empty()
        {
            return Err(anyhow!(
                "Credentials are missing. Please make sure your connectors.yaml file is correct. Credentials received: {:#?}", self
            ));
        }
        Ok(())
    }
}
