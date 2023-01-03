//! Functionality for handling group diffs in tableau

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::{anyhow, Context, Result};

use jetty_core::access_graph::translate::diffs::groups;
use serde_json::json;

use crate::{rest, TableauConnector};

use super::{SequencedFutures, SequencedPlans};

impl TableauConnector {
    /// prepare the plans for group changes (for jetty plan)
    pub(crate) fn prepare_groups_plan(
        &self,
        group_diffs: &Vec<groups::LocalDiff>,
    ) -> Result<SequencedPlans> {
        let mut plans = SequencedPlans::default();

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
                        .ok_or_else(|| {
                            anyhow!(
                                "can't delete group {}: group doesn't exist",
                                &diff.group_name
                            )
                        })?;

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

    /// Generate futures for the requests to be executed as part of `jetty apply`
    pub(super) fn generate_group_apply_futures<'a>(
        &'a self,
        group_diffs: &'a Vec<groups::LocalDiff>,
        group_map: Arc<Mutex<HashMap<String, String>>>,
    ) -> Result<SequencedFutures> {
        let mut futures = SequencedFutures::default();

        for diff in group_diffs {
            match &diff.details {
                groups::LocalDiffDetails::AddGroup { member_of } => {
                    if !member_of.is_empty() {
                        panic!("tableau does not support nested groups")
                    }

                    // Request to create the group
                    futures.0.push(Box::pin(
                        self.create_group_and_add_to_env(&diff.group_name, Arc::clone(&group_map)),
                    ));
                }
                groups::LocalDiffDetails::RemoveGroup => {
                    futures
                        .0
                        .push(Box::pin(self.execute_delete_group_with_deferred_lookup(
                            &diff.group_name,
                            Arc::clone(&group_map),
                        )));
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
        Ok(futures)
    }

    /// Async function that deletes a group, deferring the lookup of the group id until
    /// the function is awaited
    async fn execute_delete_group_with_deferred_lookup(
        &self,
        group_name: &String,
        group_map: Arc<Mutex<HashMap<String, String>>>,
    ) -> Result<()> {
        let group_id = &super::group_lookup_from_mutex(group_map, group_name)?;
        self.execute_to_unit_result(self.build_delete_group_request(group_id)?)
            .await
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

    /// Async function to create a new group in tableau, and then add it to the group map for future lookup.
    async fn create_group_and_add_to_env(
        &self,
        group_name: &String,
        group_map: Arc<Mutex<HashMap<String, String>>>,
    ) -> Result<()> {
        let req = self.build_add_group_request(group_name)?;
        let resp = self
            .coordinator
            .rest_client
            .execute(req)
            .await?
            .json::<serde_json::Value>()
            .await?;

        let group_id = rest::get_json_from_path(&resp, &vec!["group".to_owned(), "id".to_owned()])?
            .as_str()
            .ok_or_else(|| anyhow!["unable to get new id for {group_name}"])?
            .to_string();

        // update the environment so that when users look for this group in the future, they are able to find it!
        let mut locked_group_map = group_map.lock().unwrap();
        locked_group_map.insert(group_name.to_owned(), group_id);

        Ok(())
    }
}
