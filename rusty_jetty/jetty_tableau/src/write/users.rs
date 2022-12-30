//! Functionality for handling group diffs in tableau

use anyhow::{anyhow, Result};

use jetty_core::access_graph::translate::diffs::users;

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
                    .unwrap_or(format!("<group_id name for new group: {}>", group));
                plans.1.push(format!(
                    r#"POST {base_url}groups/{group_id}/users
body:
  {{
  "user": {{
    "id": {},
  }}
  }}"#,
                    diff.user
                ));
            }
            for group in &diff.group_membership.remove {
                // get the group_id
                let group_id = self
                    .coordinator
                    .env
                    .get_group_id_by_name(group)
                    .unwrap_or(format!("<group_id name for new group: {}>", group));
                plans.1.push(format!(
                    r#"DELETE {base_url}groups/{group_id}/users/{}"#,
                    diff.user
                ));
            }
        }
        Ok(plans)
    }
}
