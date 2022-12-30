//! Functionality for handling group diffs in tableau

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::{anyhow, Context, Result};

use jetty_core::access_graph::translate::diffs::users;
use serde_json::json;

use crate::TableauConnector;

use super::{PrioritizedFutures, PrioritizedPlans};

impl TableauConnector {
    pub(crate) fn prepare_users_plan(
        &self,
        user_diffs: &Vec<users::LocalDiff>,
    ) -> Result<PrioritizedPlans> {
        let mut plans = PrioritizedPlans::default();

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

    async fn generate_user_apply_futures<'a>(
        &'a self,
        user_diffs: &'a Vec<users::LocalDiff>,
        group_map: Arc<Mutex<HashMap<String, String>>>,
    ) -> Result<PrioritizedFutures> {
        let mut futures = PrioritizedFutures::default();

        for diff in user_diffs {
            for group in &diff.group_membership.add {
                // get the group_id
                let temp_group_map = group_map.lock().unwrap();
                let group_id = temp_group_map
                    .get(group)
                    .ok_or(anyhow!("Unable to find group id for {}", group))?;
                futures.1.push(Box::pin(self.execute_to_unit_result(
                    self.build_add_user_request(&group_id, &diff.user)?,
                )));
            }
            for group in &diff.group_membership.remove {
                // get the group_id
                let temp_group_map = group_map.lock().unwrap();
                let group_id = temp_group_map
                    .get(group)
                    .ok_or(anyhow!("Unable to find group id for {}", group))?;
                futures.1.push(Box::pin(self.execute_to_unit_result(
                    self.build_remove_user_request(&group_id, &diff.user)?,
                )));
            }
        }

        Ok(futures)
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

    /// Function to add users to a group, deferring the group lookup until it's needed. This
    /// allows it to work for new groups
    async fn add_user_to_group(
        &self,
        user: &String,
        group_name: &String,
        group_map: Arc<Mutex<HashMap<String, String>>>,
    ) -> Result<()> {
        // get the group_id
        let mut group_id = "".to_owned();
        {
            let temp_group_map = group_map.lock().unwrap();
            group_id = temp_group_map
                .get(group_name)
                .ok_or(anyhow!("Unable to find group id for {}", group_name))?
                .to_owned();
        }

        // Add the user
        self.execute_to_unit_result(self.build_add_user_request(&group_id, user)?)
            .await
    }
}
