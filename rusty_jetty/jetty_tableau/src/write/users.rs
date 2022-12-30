//! Functionality for handling group diffs in tableau

use anyhow::{anyhow, Context, Result};

use jetty_core::access_graph::translate::diffs::users;
use serde_json::json;

use crate::TableauConnector;

use super::PrioritizedPlans;

impl TableauConnector {
    pub(crate) fn prepare_users_plan(
        &self,
        user_diffs: &Vec<users::LocalDiff>,
    ) -> Result<PrioritizedPlans> {
        let mut plans = PrioritizedPlans::default();

        let base_url = format![
            "https://{}/api/{}/sites/{}/",
            self.coordinator.rest_client.get_server_name(),
            self.coordinator.rest_client.get_api_version(),
            self.coordinator.rest_client.get_site_id()?,
        ];

        for diff in user_diffs {
            for group in &diff.group_membership.add {
                // get the group_id
                let group_id = self
                    .coordinator
                    .env
                    .get_group_id_by_name(group)
                    .unwrap_or(format!("group_id_for_new_group_{}", group));
                plans
                    .1
                    .push(self.build_add_user_request(&group_id, &diff.user)?);
            }
            for group in &diff.group_membership.remove {
                // get the group_id
                let group_id = self
                    .coordinator
                    .env
                    .get_group_id_by_name(group)
                    .unwrap_or(format!("group_id_for_new_group_{}", group));
                plans
                    .1
                    .push(self.build_remove_user_request(&group_id, &diff.user)?);
            }
        }
        Ok(plans)
    }

    /// build a request to add a group
    fn build_add_user_request(
        &self,
        group_id: &String,
        user_id: &String,
    ) -> Result<reqwest::Request> {
        // Add the user
        let req_body = json!(
            {
                "user": {
                  "id": user_id,
                }
            }
        );
        self.coordinator
            .rest_client
            .build_request(
                format!("groups/{group_id}/users").to_string(),
                Some(req_body),
                reqwest::Method::POST,
            )?
            .build()
            .context("building request")
    }

    /// build a request to remove a group
    fn build_remove_user_request(
        &self,
        group_id: &String,
        user_id: &String,
    ) -> Result<reqwest::Request> {
        self.coordinator
            .rest_client
            .build_request(
                format!("groups/{group_id}/users/{}", user_id),
                None,
                reqwest::Method::DELETE,
            )?
            .build()
            .context("building request")
    }
}
