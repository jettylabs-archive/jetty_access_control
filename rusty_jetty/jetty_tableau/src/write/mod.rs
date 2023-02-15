//! Functionality for the tableau write path

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::{anyhow, bail, Result};

use futures::future::BoxFuture;
use jetty_core::access_graph::translate::diffs::LocalConnectorDiffs;
use reqwest::Request;

use crate::TableauConnector;

mod default_policies;
mod groups;
mod policies;
mod users;

/// Struct containing a sequenced plans. The sequence is important to get the order
/// of operations right (don't add users to groups until they're created, for example)
#[derive(Default)]
pub(crate) struct SequencedPlans(
    pub(crate) Vec<reqwest::Request>,
    pub(crate) Vec<reqwest::Request>,
    pub(crate) Vec<reqwest::Request>,
);

impl SequencedPlans {
    /// extend prioritized plans, consuming other
    pub(crate) fn extend(&mut self, other: Self) {
        self.0.extend(other.0.into_iter());
        self.1.extend(other.1.into_iter());
        self.2.extend(other.2.into_iter());
    }

    /// Flatten planned requests to a vector of strings for 'jetty plan'
    pub(crate) fn flatten_to_string_vec(&self) -> Vec<String> {
        [&self.0, &self.1, &self.2]
            .into_iter()
            .flatten()
            .map(request_to_string)
            .collect::<Vec<_>>()
    }
}

/// Struct containing a sequenced futures. The sequence is important to get the order
/// of operations right (don't add users to groups until they're created, for example)
#[derive(Default)]
pub(crate) struct SequencedFutures<'a>(
    pub(crate) Vec<BoxFuture<'a, Result<()>>>,
    pub(crate) Vec<BoxFuture<'a, Result<()>>>,
    pub(crate) Vec<BoxFuture<'a, Result<()>>>,
);

impl<'a> SequencedFutures<'a> {
    /// extend prioritized futures, consuming other
    pub(crate) fn extend(&mut self, other: Self) {
        self.0.extend(other.0.into_iter());
        self.1.extend(other.1.into_iter());
        self.2.extend(other.2.into_iter());
    }
}

/// Convert a request to a string representation to display as part of the plan
fn request_to_string(req: &reqwest::Request) -> String {
    let mut res = format!("{} {}\n", req.method(), req.url().path(),);
    if let Some(b) = req.body() {
        let val: serde_json::Value =
            serde_json::from_slice(b.as_bytes().unwrap_or_default()).unwrap_or_default();
        res.push_str("body:\n");
        res.push_str(
            serde_json::to_string_pretty(&val)
                .unwrap_or_default()
                .as_str(),
        );
    };
    res
}

impl TableauConnector {
    /// generate the futures that will be executed with 'jetty apply'
    pub(super) fn generate_plan_futures<'a>(
        &'a self,
        diffs: &'a LocalConnectorDiffs,
    ) -> Result<SequencedFutures> {
        let group_map: HashMap<String, String> = self
            .coordinator
            .env
            .groups.values().map(|g| (g.name.to_owned(), g.id.to_owned()))
            .collect();

        let group_map_mutex = Arc::new(Mutex::new(group_map));

        let mut futures = SequencedFutures::default();

        let group_futures =
            self.generate_group_apply_futures(&diffs.groups, Arc::clone(&group_map_mutex))?;
        let user_futures =
            self.generate_user_apply_futures(&diffs.users, Arc::clone(&group_map_mutex))?;
        let policy_futures =
            self.generate_policy_apply_futures(&diffs.policies, Arc::clone(&group_map_mutex))?;
        let default_policy_futures = self.generate_default_policy_apply_futures(
            &diffs.default_policies,
            Arc::clone(&group_map_mutex),
        )?;

        futures.extend(group_futures);
        futures.extend(user_futures);
        futures.extend(policy_futures);
        futures.extend(default_policy_futures);

        Ok(futures)
    }

    /// Plan the requests that will be executed - for 'jetty plan'
    pub(super) fn generate_request_plan(
        &self,
        diffs: &LocalConnectorDiffs,
    ) -> Result<SequencedPlans> {
        let mut plans = SequencedPlans::default();

        let group_plans = self.prepare_groups_plan(&diffs.groups)?;
        let user_plans = self.prepare_users_plan(&diffs.users)?;
        let policy_plans = self.prepare_policies_plan(&diffs.policies)?;
        let default_policy_plans = self.prepare_default_policies_plan(&diffs.default_policies)?;

        plans.extend(group_plans);
        plans.extend(user_plans);
        plans.extend(policy_plans);
        plans.extend(default_policy_plans);

        Ok(plans)
    }

    /// Function to execute a request and return a unit response
    async fn execute_to_unit_result(&self, request: Request) -> Result<()> {
        let res = self.coordinator.rest_client.execute(request).await?;

        let status = res.status();
        if status.is_client_error() || status.is_server_error() {
            let url = res.url().to_owned();
            let error_detail = if let Some(details) = get_error_details_from_response(res).await {
                format!(": {details}")
            } else {
                String::new()
            };
            bail!("HTTP error ({status}) for url ({url}){error_detail}")
        } else {
            Ok(())
        }
    }
}

/// given an Arc Mutex containing a map of group names to ids, return the id for a given name
/// This was created to be called from async functions that need to fetch the id of a
/// group just in time
fn group_lookup_from_mutex(
    group_map: Arc<Mutex<HashMap<String, String>>>,
    group_name: &String,
) -> Result<String> {
    // get the group_id
    let temp_group_map = group_map.lock().unwrap();
    let group_id = temp_group_map
        .get(group_name)
        .ok_or_else(|| anyhow!("Unable to find group id for {}", group_name))?
        .to_owned();
    Ok(group_id)
}

async fn get_error_details_from_response(res: reqwest::Response) -> Option<String> {
    res.json::<serde_json::Value>()
        .await
        .ok()
        .and_then(|v| v.get("error").cloned())
        .and_then(|v| v.get("detail").cloned())
        .and_then(|v| v.as_str().map(|v| v.to_string()))
}
