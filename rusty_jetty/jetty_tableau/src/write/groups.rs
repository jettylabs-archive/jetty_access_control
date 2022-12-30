//! Functionality for handling group diffs in tableau

use anyhow::{anyhow, Result};

use jetty_core::access_graph::translate::diffs::groups;

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
                    // Request to create the group

                    plans.0.push(format!(
                        r#"POST {base_url}groups
        body:
          {{
            "group": {{
              "name": {},
            }}
          }}"#,
                        diff.group_name
                    ));
                    if !member_of.is_empty() {
                        panic!("tableau does not support nested groups")
                    }
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

                    plans.0.push(format!(
                        "DELETE {base_url}groups/{group_id}\n## {group_id} is the id for {}\n",
                        diff.group_name
                    ));
                }
                groups::LocalDiffDetails::ModifyGroup {
                    add_member_of,
                    remove_member_of,
                } => {
                    // This only modifies group hierarcy, which Tableau doesn't have.
                    if !add_member_of.is_empty() || !remove_member_of.is_empty() {
                        panic!("tableau does not support nested groups")
                    }
                }
            }
        }
        Ok(plans)
    }
}
