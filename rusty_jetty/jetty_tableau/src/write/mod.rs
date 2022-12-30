//! Functionality for the tableau write path

use std::{
    collections::HashMap,
    pin::Pin,
    sync::{Arc, Mutex},
};

use anyhow::{anyhow, Context, Result};

use futures::{future::BoxFuture, Future};
use jetty_core::access_graph::translate::diffs::LocalConnectorDiffs;
use reqwest::Request;
use serde_json::json;

use crate::{rest, TableauConnector};

mod default_policies;
mod groups;
mod policies;
mod users;

#[derive(Default)]
pub(crate) struct PrioritizedPlans(
    pub(crate) Vec<reqwest::Request>,
    pub(crate) Vec<reqwest::Request>,
    pub(crate) Vec<reqwest::Request>,
);

impl PrioritizedPlans {
    pub(crate) fn extend(&mut self, other: &Self) {
        self.0
            .extend(other.0.iter().map(|r| r.try_clone().unwrap()));
        self.1
            .extend(other.1.iter().map(|r| r.try_clone().unwrap()));
        self.2
            .extend(other.2.iter().map(|r| r.try_clone().unwrap()));
    }
    pub(crate) fn flatten_to_string_vec(&self) -> Vec<String> {
        [&self.0, &self.1, &self.2]
            .into_iter()
            .flatten()
            .map(|r| request_to_string(r))
            .collect::<Vec<_>>()
    }
}

#[derive(Default)]
pub(crate) struct PrioritizedFutures<'a>(
    pub(crate) Vec<BoxFuture<'a, Result<()>>>,
    pub(crate) Vec<BoxFuture<'a, Result<()>>>,
    pub(crate) Vec<BoxFuture<'a, Result<()>>>,
);

fn request_to_string(req: &reqwest::Request) -> String {
    let mut res = format!("{} {}\n", req.method(), req.url().path(),);
    match req.body() {
        Some(b) => {
            let val: serde_json::Value =
                serde_json::from_slice(b.as_bytes().unwrap_or_default()).unwrap_or_default();
            res.push_str("body:\n");
            res.push_str(
                serde_json::to_string_pretty(&val)
                    .unwrap_or_default()
                    .as_str(),
            );
        }
        None => (),
    };
    res
}

impl TableauConnector {
    pub(super) fn generate_plan_futures<'a>(
        &'a self,
        diffs: &'a LocalConnectorDiffs,
    ) -> Result<Vec<Vec<Pin<Box<dyn Future<Output = Result<()>> + '_ + Send>>>>> {
        todo!();
        // let mut batch1: Vec<BoxFuture<_>> = Vec::new();
        // let mut batch2: Vec<BoxFuture<_>> = Vec::new();

        // let group_map: HashMap<String, String> = self
        //     .coordinator
        //     .env
        //     .groups
        //     .iter()
        //     .map(|(_name, g)| (g.name.to_owned(), g.id.to_owned()))
        //     .collect();

        // let group_map_mutex = Arc::new(Mutex::new(group_map));

        // // Starting with groups
        // let group_diffs = &diffs.groups;
        // for diff in group_diffs {
        //     match &diff.details {
        //         groups::LocalDiffDetails::AddGroup { members } => {
        //             // start by creating the group
        //             batch1.push(Box::pin(self.create_group_and_add_to_env(
        //                 &diff.group_name,
        //                 group_map_mutex.clone(),
        //             )));
        //             for user in &members.users {
        //                 batch2.push(Box::pin(self.add_user_to_group(
        //                     user,
        //                     &diff.group_name,
        //                     Arc::clone(&group_map_mutex),
        //                 )))
        //             }
        //         }
        //         groups::LocalDiffDetails::RemoveGroup => {
        //             // get the group_id
        //             let temp_group_map = group_map_mutex.lock().unwrap();
        //             let group_id = temp_group_map
        //                 .get(&diff.group_name)
        //                 .ok_or(anyhow!("Unable to find group id for {}", &diff.group_name))?;

        //             let req = self.coordinator.rest_client.build_request(
        //                 format!("groups/{group_id}"),
        //                 None,
        //                 reqwest::Method::DELETE,
        //             )?;
        //             batch1.push(Box::pin(request_builder_to_unit_result(req)))
        //         }
        //         groups::LocalDiffDetails::ModifyGroup { add, remove } => {
        //             // Add users
        //             for user in &add.users {
        //                 batch2.push(Box::pin(self.add_user_to_group(
        //                     user,
        //                     &diff.group_name,
        //                     group_map_mutex.clone(),
        //                 )))
        //             }
        //             // Remove users
        //             // get the group_id
        //             let temp_group_map = group_map_mutex.lock().unwrap();
        //             let group_id = temp_group_map
        //                 .get(&diff.group_name)
        //                 .ok_or(anyhow!("Unable to find group id for {}", &diff.group_name))?;

        //             for user in &remove.users {
        //                 let req = self.coordinator.rest_client.build_request(
        //                     format!("groups/{group_id}/users/{user}"),
        //                     None,
        //                     reqwest::Method::DELETE,
        //                 )?;
        //                 batch1.push(Box::pin(request_builder_to_unit_result(req)))
        //             }
        //         }
        //     }
        // }
        // Ok(vec![batch1, batch2])
    }

    pub(super) fn generate_request_plan(
        &self,
        diffs: &LocalConnectorDiffs,
    ) -> Result<PrioritizedPlans> {
        let mut plans = PrioritizedPlans::default();

        let group_plans = self.prepare_groups_plan(&diffs.groups)?;
        let user_plans = self.prepare_users_plan(&diffs.users)?;
        let policy_plans = self.prepare_policies_plan(&diffs.policies)?;
        let default_policy_plans = self.prepare_default_policies_plan(&diffs.default_policies)?;

        plans.extend(&group_plans);
        plans.extend(&user_plans);
        plans.extend(&policy_plans);
        plans.extend(&default_policy_plans);

        Ok(plans)
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
        let req_body = json!({"user": {"id": user}});
        self.coordinator
            .rest_client
            .build_request(
                format!("groups/{group_id}/users"),
                Some(req_body),
                reqwest::Method::POST,
            )?
            .send()
            .await?;

        Ok(())
    }

    /// Function to execute a request and return a unit response
    async fn execute_to_unit_result(&self, request: Request) -> Result<()> {
        let x = self.coordinator.rest_client.execute(request).await?;
        Ok(())
    }
}
