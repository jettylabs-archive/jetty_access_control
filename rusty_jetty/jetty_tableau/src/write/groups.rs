//! Functionality for handling group diffs in tableau

use anyhow::{anyhow, Context, Result};

use jetty_core::access_graph::translate::diffs::groups;
use serde_json::json;

use crate::TableauConnector;

use super::PrioritizedPlans;

impl TableauConnector {
    pub(crate) fn prepare_groups_plan(
        &self,
        group_diffs: &Vec<groups::LocalDiff>,
    ) -> Result<PrioritizedPlans> {
        let mut plans = PrioritizedPlans::default();

        let base_url = format![
            "https://{}/api/{}/sites/{}/",
            self.coordinator.rest_client.get_server_name(),
            self.coordinator.rest_client.get_api_version(),
            self.coordinator.rest_client.get_site_id()?,
        ];

        // Starting with groups
        for diff in group_diffs {
            match &diff.details {
                groups::LocalDiffDetails::AddGroup { member_of } => {
                    if !member_of.is_empty() {
                        panic!("tableau does not support nested groups")
                    }

                    // Request to create the group
                    plans
                        .0
                        .push(self.build_add_group_request(&diff.group_name)?);
                }
                groups::LocalDiffDetails::RemoveGroup => {
                    // get the group_id
                    let group_id = self
                        .coordinator
                        .env
                        .get_group_id_by_name(&diff.group_name)
                        .ok_or(anyhow!(
                            "can't delete group {}: group doesn't exist",
                            &diff.group_name
                        ))?;

                    plans.0.push(self.build_delete_group_request(&group_id)?);
                }
                groups::LocalDiffDetails::ModifyGroup {
                    add_member_of,
                    remove_member_of,
                } => {
                    // This only modifies group hierarchy, which Tableau doesn't have.
                    if !add_member_of.is_empty() || !remove_member_of.is_empty() {
                        panic!("tableau does not support nested groups")
                    }
                }
            }
        }
        Ok(plans)
    }

    /// build a request to add a group
    fn build_add_group_request(&self, group_name: &String) -> Result<reqwest::Request> {
        // Add the user
        let req_body = json!(
            {
                "group": {
                  "name": group_name,
                }
            }
        );
        self.coordinator
            .rest_client
            .build_request("groups".to_string(), Some(req_body), reqwest::Method::POST)?
            .build()
            .context("building request")
    }

    /// build a request to remove a group
    fn build_delete_group_request(&self, group_id: &String) -> Result<reqwest::Request> {
        self.coordinator
            .rest_client
            .build_request(format!("groups/{group_id}"), None, reqwest::Method::DELETE)?
            .build()
            .context("building request")
    }
}
