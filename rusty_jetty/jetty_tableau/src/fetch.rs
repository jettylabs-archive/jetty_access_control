//! This module fetches all of the relevant information from Tableau. It does
//! so in several stages:
//!
//! - Fetch the basic details of each user, group, workbook, etc.
//! - Fetch information about datasources and connections to determine
//!    lineage
//! - Fetch all users of each group
//! - Fetch permissions for every asset
//!
//! The module uses `updated_at` information to avoid fetching up-to-date information.

use std::collections::HashMap;

use anyhow::{Context, Result};
use async_trait::async_trait;

use crate::nodes2::{self, User};
use crate::rest::{self, FetchJson};

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;

    #[test]
    fn test_fetching_token_works() -> Result<()> {
        crate::connector_setup().context("running tableau connector setup")?;
        Ok(())
    }
}
