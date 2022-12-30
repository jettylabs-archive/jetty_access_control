//! Functionality for handling group diffs in tableau

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::{anyhow, Context, Result};

use jetty_core::{
    access_graph::translate::diffs::{policies, users},
    write::assets::PolicyState,
};
use reqwest::Request;

use crate::{
    coordinator::TableauAssetReference, nodes::IndividualPermission, rest::TableauAssetType,
    TableauConnector,
};

use super::{PrioritizedFutures, PrioritizedPlans};

impl TableauConnector {
    pub(crate) fn prepare_policies_plan(
        &self,
        policy_diffs: &Vec<policies::LocalDiff>,
    ) -> Result<PrioritizedPlans> {
        let mut plans = PrioritizedPlans::default();

        for diff in policy_diffs {
            let asset_reference = self.coordinator.env.cual_id_map.get(&diff.asset).unwrap();

            let mut user_adds = HashMap::new();
            let mut group_adds = HashMap::new();

            for (user, details) in &diff.users {
                match details {
                    jetty_core::write::assets::diff::policies::DiffDetails::AddAgent { add } => {
                        user_adds.insert(
                            user.to_owned(),
                            add.privileges
                                .iter()
                                .map(|p| IndividualPermission::from_string(p))
                                .collect::<Vec<_>>(),
                        );
                    }
                    jetty_core::write::assets::diff::policies::DiffDetails::RemoveAgent {
                        remove,
                    } => plans.1.extend(self.build_delete_policy_requests(
                        remove,
                        asset_reference,
                        user,
                        "user",
                    )?),
                    jetty_core::write::assets::diff::policies::DiffDetails::ModifyAgent {
                        add,
                        remove,
                    } => {
                        user_adds.insert(
                            user.to_owned(),
                            add.privileges
                                .iter()
                                .map(|p| IndividualPermission::from_string(p))
                                .collect::<Vec<_>>(),
                        );
                        plans.1.extend(self.build_delete_policy_requests(
                            remove,
                            asset_reference,
                            user,
                            "user",
                        )?);
                    }
                }
            }
            for (group, details) in &diff.groups {
                let group_id = self
                    .coordinator
                    .env
                    .get_group_id_by_name(group)
                    .unwrap_or(format!("<group_id name for new group: {}>", group));
                match details {
                    jetty_core::write::assets::diff::policies::DiffDetails::AddAgent { add } => {
                        group_adds.insert(
                            group_id.to_owned(),
                            add.privileges
                                .iter()
                                .map(|p| IndividualPermission::from_string(p))
                                .collect::<Vec<_>>(),
                        );
                    }
                    jetty_core::write::assets::diff::policies::DiffDetails::RemoveAgent {
                        remove,
                    } => plans.1.extend(self.build_delete_policy_requests(
                        remove,
                        asset_reference,
                        &group_id,
                        "group",
                    )?),
                    jetty_core::write::assets::diff::policies::DiffDetails::ModifyAgent {
                        add,
                        remove,
                    } => {
                        if !add.privileges.is_empty() {
                            group_adds.insert(
                                group_id.to_owned(),
                                add.privileges
                                    .iter()
                                    .map(|p| IndividualPermission::from_string(p))
                                    .collect::<Vec<_>>(),
                            );
                        }
                        if !remove.privileges.is_empty() {
                            plans.1.extend(self.build_delete_policy_requests(
                                remove,
                                asset_reference,
                                &group_id,
                                "group",
                            )?);
                        }
                    }
                }
            }
            if !user_adds.is_empty() || !group_adds.is_empty() {
                plans.1.push(self.build_add_policy_request(
                    asset_reference,
                    user_adds,
                    group_adds,
                )?);
            }
        }
        Ok(plans)
    }

    async fn generate_policy_apply_futures<'a>(
        &'a self,
        policy_diffs: &'a Vec<policies::LocalDiff>,
        group_map: Arc<Mutex<HashMap<String, String>>>,
    ) -> Result<PrioritizedFutures> {
        let mut futures = PrioritizedFutures::default();

        for diff in policy_diffs {
            let asset_reference = self.coordinator.env.cual_id_map.get(&diff.asset).unwrap();

            let mut user_adds = HashMap::new();
            let mut group_adds = HashMap::new();

            for (user, details) in &diff.users {
                match details {
                    jetty_core::write::assets::diff::policies::DiffDetails::AddAgent { add } => {
                        user_adds.insert(
                            user.to_owned(),
                            add.privileges
                                .iter()
                                .map(|p| IndividualPermission::from_string(p))
                                .collect::<Vec<_>>(),
                        );
                    }
                    jetty_core::write::assets::diff::policies::DiffDetails::RemoveAgent {
                        remove,
                    } => self
                        .build_delete_policy_requests(remove, asset_reference, user, "user")?
                        .into_iter()
                        .for_each(|req| futures.1.push(Box::pin(self.execute_to_unit_result(req)))),
                    jetty_core::write::assets::diff::policies::DiffDetails::ModifyAgent {
                        add,
                        remove,
                    } => {
                        user_adds.insert(
                            user.to_owned(),
                            add.privileges
                                .iter()
                                .map(|p| IndividualPermission::from_string(p))
                                .collect::<Vec<_>>(),
                        );
                        self.build_delete_policy_requests(remove, asset_reference, user, "user")?
                            .into_iter()
                            .for_each(|req| {
                                futures.1.push(Box::pin(self.execute_to_unit_result(req)))
                            });
                    }
                }
            }
            for (group, details) in &diff.groups {
                // get the group_id
                let temp_group_map = group_map.lock().unwrap();
                let group_id = temp_group_map
                    .get(group)
                    .ok_or(anyhow!("Unable to find group id for {}", group))?;
                match details {
                    jetty_core::write::assets::diff::policies::DiffDetails::AddAgent { add } => {
                        group_adds.insert(
                            group_id.to_owned(),
                            add.privileges
                                .iter()
                                .map(|p| IndividualPermission::from_string(p))
                                .collect::<Vec<_>>(),
                        );
                    }
                    jetty_core::write::assets::diff::policies::DiffDetails::RemoveAgent {
                        remove,
                    } => self
                        .build_delete_policy_requests(remove, asset_reference, &group_id, "group")?
                        .into_iter()
                        .for_each(|req| futures.1.push(Box::pin(self.execute_to_unit_result(req)))),

                    jetty_core::write::assets::diff::policies::DiffDetails::ModifyAgent {
                        add,
                        remove,
                    } => {
                        if !add.privileges.is_empty() {
                            group_adds.insert(
                                group_id.to_owned(),
                                add.privileges
                                    .iter()
                                    .map(|p| IndividualPermission::from_string(p))
                                    .collect::<Vec<_>>(),
                            );
                        }
                        if !remove.privileges.is_empty() {
                            self.build_delete_policy_requests(
                                remove,
                                asset_reference,
                                &group_id,
                                "group",
                            )?
                            .into_iter()
                            .for_each(|req| {
                                futures.1.push(Box::pin(self.execute_to_unit_result(req)))
                            });
                        }
                    }
                }
            }
            if !user_adds.is_empty() || !group_adds.is_empty() {
                futures.1.push(Box::pin(self.execute_to_unit_result(
                    self.build_add_policy_request(asset_reference, user_adds, group_adds)?,
                )));
            }
        }
        Ok(futures)
    }

    pub(crate) fn build_add_policy_request(
        &self,
        asset: &TableauAssetReference,
        user: HashMap<String, Vec<IndividualPermission>>,
        group: HashMap<String, Vec<IndividualPermission>>,
    ) -> Result<reqwest::Request> {
        // Add the user
        let req_body = generate_add_privileges_request_body(asset, user, group)?;
        self.coordinator
            .rest_client
            .build_request(
                format!(
                    "{}/{}/permissions",
                    asset.asset_type.as_category_str(),
                    asset.id
                ),
                Some(req_body),
                reqwest::Method::PUT,
            )?
            .build()
            .context("building request")
    }

    pub(crate) fn build_delete_policy_requests(
        &self,
        state: &PolicyState,
        asset: &TableauAssetReference,
        grantee_id: &String,
        grantee_type: &str,
    ) -> Result<Vec<Request>> {
        state
            .privileges
            .iter()
            .map(|p| {
                let permission = IndividualPermission::from_string(p);
                self.coordinator
                    .rest_client
                    .build_request(
                        format!(
                            "{}/{}/permissions/{grantee_type}/{grantee_id}/{}/{}",
                            asset.asset_type.as_category_str(),
                            &asset.id,
                            permission.capability,
                            permission.mode.to_string()
                        ),
                        None,
                        reqwest::Method::DELETE,
                    )?
                    .build()
                    .context("building request")
            })
            .collect()
    }
}

pub(super) fn generate_add_privileges_request_body(
    asset: &TableauAssetReference,
    user: HashMap<String, Vec<IndividualPermission>>,
    group: HashMap<String, Vec<IndividualPermission>>,
) -> Result<serde_json::Value> {
    let mut request_text = "".to_owned();
    request_text += "
  { \"permissions\": {\n";
    if !matches!(asset.asset_type, TableauAssetType::Project) {
        request_text += format!(
            "\"{}\": {{ \"id\": \"{}\" }},\n",
            asset.asset_type.as_str(),
            asset.id
        )
        .as_str();
    }
    request_text += "\"granteeCapabilities\": [\n";
    for (user_id, permissions) in user.iter() {
        request_text += "        {\n";
        request_text += format!(
            "          \"user\": {{ \"id\": \"{}\" }},\n",
            user_id.to_owned()
        )
        .as_str();
        request_text += "          \"capabilities\": [\n";
        for permission in permissions {
            request_text += format!(
                "            {{ \"capability\": {{ \"name\": \"{}\", \"mode\": \"{}\" }} }}\n",
                &permission.capability,
                &permission.mode.to_string()
            )
            .as_str()
        }
        request_text += "]\n";
        request_text += "}\n"
    }
    for (group_id, permissions) in group.iter() {
        request_text += "{";
        request_text += format!("\"group\": {{ \"id\": \"{}\" }},\n", group_id.to_owned()).as_str();
        request_text += "\"capabilities\": [\n";
        for permission in permissions {
            request_text += format!(
                "            {{ \"capability\": {{ \"name\": \"{}\", \"mode\": \"{}\" }} }}\n",
                &permission.capability,
                &permission.mode.to_string()
            )
            .as_str()
        }
        request_text += "\n";
        request_text += "}\n"
    }
    request_text += "]\n";
    request_text += "}\n";
    request_text += "}\n";

    serde_json::from_str(&request_text).context("building request body")
}
