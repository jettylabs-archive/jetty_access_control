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
    /// extend prioritized plans, consuming other
    pub(crate) fn extend(&mut self, other: Self) {
        self.0.extend(other.0.into_iter().map(|r| r));
        self.1.extend(other.1.into_iter().map(|r| r));
        self.2.extend(other.2.into_iter().map(|r| r));
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

impl<'a> PrioritizedFutures<'a> {
    /// extend prioritized futures, consuming other
    pub(crate) fn extend(&mut self, other: Self) {
        self.0.extend(other.0.into_iter().map(|r| r));
        self.1.extend(other.1.into_iter().map(|r| r));
        self.2.extend(other.2.into_iter().map(|r| r));
    }
}

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
    ) -> Result<PrioritizedFutures> {
        let group_map: HashMap<String, String> = self
            .coordinator
            .env
            .groups
            .iter()
            .map(|(_name, g)| (g.name.to_owned(), g.id.to_owned()))
            .collect();

        let group_map_mutex = Arc::new(Mutex::new(group_map));

        let mut futures = PrioritizedFutures::default();

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

    pub(super) fn generate_request_plan(
        &self,
        diffs: &LocalConnectorDiffs,
    ) -> Result<PrioritizedPlans> {
        let mut plans = PrioritizedPlans::default();

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
        self.coordinator.rest_client.execute(request).await?;
        Ok(())
    }
}
