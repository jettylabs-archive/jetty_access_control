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

use crate::nodes2::{self};
use crate::rest::{self, FetchJson};

struct TableauEnvironment {
    users: HashMap<String, nodes2::User>,
    groups: HashMap<String, nodes2::Group>,
    projects: HashMap<String, nodes2::Project>,
    datasources: HashMap<String, nodes2::Datasource>,
    data_connections: HashMap<String, nodes2::DataConnection>,
    flows: HashMap<String, nodes2::Flow>,
    lenses: HashMap<String, nodes2::Lens>,
    metrics: HashMap<String, nodes2::Metric>,
    views: HashMap<String, nodes2::View>,
    workbooks: HashMap<String, nodes2::Workbook>,
}

#[async_trait]
trait TableauFetcher {
    async fn get_basic_users(&self) -> Result<HashMap<String, nodes2::User>>;
}

#[async_trait]
impl TableauFetcher for rest::TableauRestClient {
    #[allow(dead_code)]
    async fn get_basic_users(&self) -> Result<HashMap<String, nodes2::User>> {
        let users = self
            .build_request("users".to_owned(), None, reqwest::Method::GET)
            .context("fetching users")?
            .fetch_json_response(Some(vec!["users".to_owned(), "user".to_owned()]))
            .await?;
        nodes2::to_asset_map(users, &nodes2::user::to_node)
    }
}

impl TableauEnvironment {
    async fn get_users(&mut self) {
        todo!()
    }
    async fn get_groups(&mut self) {
        todo!()
    }
    async fn get_projects(&mut self) {
        todo!()
    }
    async fn get_datasources(&mut self) {
        todo!()
    }
    async fn get_flows(&mut self) {
        todo!()
    }
    async fn get_lenses(&mut self) {
        todo!()
    }
    async fn get_metrics(&mut self) {
        todo!()
    }
    async fn get_views(&mut self) {
        todo!()
    }
    async fn get_workbooks(&mut self) {
        todo!()
    }
}

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
