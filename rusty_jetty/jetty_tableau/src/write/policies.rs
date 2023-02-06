//! Functionality for handling group diffs in tableau

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::{Context, Result};

use futures::future::BoxFuture;
use jetty_core::{access_graph::translate::diffs::policies, write::assets::PolicyState};
use reqwest::Request;

use crate::{
    coordinator::TableauAssetReference, nodes::IndividualPermission, rest::TableauAssetType,
    TableauConnector,
};

use super::{SequencedFutures, SequencedPlans};

impl TableauConnector {
    /// generate the plan for required changes.
    pub(crate) fn prepare_policies_plan(
        &self,
        policy_diffs: &Vec<policies::LocalDiff>,
    ) -> Result<SequencedPlans> {
        let mut plans = SequencedPlans::default();

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
                                .map(IndividualPermission::from_string)
                                .collect::<Vec<_>>(),
                        );
                    }
                    jetty_core::write::assets::diff::policies::DiffDetails::RemoveAgent {
                        remove,
                    } => plans.2.extend(self.build_delete_policy_requests(
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
                                .map(IndividualPermission::from_string)
                                .collect::<Vec<_>>(),
                        );
                        plans.2.extend(self.build_delete_policy_requests(
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
                    .unwrap_or(format!("<group_id name for new group: {group}>"));
                match details {
                    jetty_core::write::assets::diff::policies::DiffDetails::AddAgent { add } => {
                        group_adds.insert(
                            group_id.to_owned(),
                            add.privileges
                                .iter()
                                .map(IndividualPermission::from_string)
                                .collect::<Vec<_>>(),
                        );
                    }
                    jetty_core::write::assets::diff::policies::DiffDetails::RemoveAgent {
                        remove,
                    } => plans.2.extend(self.build_delete_policy_requests(
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
                                    .map(IndividualPermission::from_string)
                                    .collect::<Vec<_>>(),
                            );
                        }
                        if !remove.privileges.is_empty() {
                            plans.2.extend(self.build_delete_policy_requests(
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
                plans.2.push(self.build_add_policy_request(
                    asset_reference,
                    user_adds,
                    group_adds,
                )?);
            }
        }
        Ok(plans)
    }

    /// Generate the futures to be applied when there are changes (for Jetty Apply)
    /// This is very similar to prepare_policies_plan(), but requires extra care because it needs
    /// to handle the id's of newly created groups. That means that the actual creation of must requests
    /// must be deferred.
    pub(super) fn generate_policy_apply_futures<'a>(
        &'a self,
        policy_diffs: &'a Vec<policies::LocalDiff>,
        group_map: Arc<Mutex<HashMap<String, String>>>,
    ) -> Result<SequencedFutures> {
        let mut futures = SequencedFutures::default();

        for diff in policy_diffs {
            let asset_reference = self.coordinator.env.cual_id_map.get(&diff.asset).unwrap();

            let mut user_adds = HashMap::new();
            let mut group_adds = HashMap::new();

            for (user_id, details) in &diff.users {
                match details {
                    jetty_core::write::assets::diff::policies::DiffDetails::AddAgent { add } => {
                        user_adds.insert(
                            user_id.to_owned(),
                            add.privileges
                                .iter()
                                .map(IndividualPermission::from_string)
                                .collect::<Vec<_>>(),
                        );
                    }
                    jetty_core::write::assets::diff::policies::DiffDetails::RemoveAgent {
                        remove,
                    } => self
                        .build_delete_policy_request_futures(
                            remove,
                            asset_reference,
                            user_id,
                            "user",
                            group_map.clone(),
                        )?
                        .into_iter()
                        .for_each(|f| futures.2.push(f)),
                    jetty_core::write::assets::diff::policies::DiffDetails::ModifyAgent {
                        add,
                        remove,
                    } => {
                        user_adds.insert(
                            user_id.to_owned(),
                            add.privileges
                                .iter()
                                .map(IndividualPermission::from_string)
                                .collect::<Vec<_>>(),
                        );
                        self.build_delete_policy_request_futures(
                            remove,
                            asset_reference,
                            user_id,
                            "user",
                            group_map.clone(),
                        )?
                        .into_iter()
                        .for_each(|f| futures.2.push(f));
                    }
                }
            }
            for (group_name, details) in &diff.groups {
                match details {
                    jetty_core::write::assets::diff::policies::DiffDetails::AddAgent { add } => {
                        group_adds.insert(
                            group_name.to_owned(),
                            add.privileges
                                .iter()
                                .map(IndividualPermission::from_string)
                                .collect::<Vec<_>>(),
                        );
                    }
                    jetty_core::write::assets::diff::policies::DiffDetails::RemoveAgent {
                        remove,
                    } => self
                        .build_delete_policy_request_futures(
                            remove,
                            asset_reference,
                            group_name, // Use the group name here, as it'll be converted to an id downstream
                            "group",
                            group_map.clone(),
                        )?
                        .into_iter()
                        .for_each(|f| futures.2.push(f)),

                    jetty_core::write::assets::diff::policies::DiffDetails::ModifyAgent {
                        add,
                        remove,
                    } => {
                        if !add.privileges.is_empty() {
                            group_adds.insert(
                                group_name.to_owned(),
                                add.privileges
                                    .iter()
                                    .map(IndividualPermission::from_string)
                                    .collect::<Vec<_>>(),
                            );
                        }
                        if !remove.privileges.is_empty() {
                            self.build_delete_policy_request_futures(
                                remove,
                                asset_reference,
                                group_name, // Use the group name here, as it'll be converted to an id downstream
                                "group",
                                group_map.clone(),
                            )?
                            .into_iter()
                            .for_each(|f| futures.2.push(f));
                        }
                    }
                }
            }
            if !user_adds.is_empty() || !group_adds.is_empty() {
                futures
                    .1
                    .push(Box::pin(self.execute_add_policy_with_deferred_lookup(
                        asset_reference,
                        user_adds,
                        group_adds,
                        Arc::clone(&group_map),
                    )));
            }
        }
        Ok(futures)
    }

    /// Creates and executes a request to add policies after looking up the relevant group id.
    async fn execute_add_policy_with_deferred_lookup(
        &self,
        asset: &TableauAssetReference,
        user: HashMap<String, Vec<IndividualPermission>>,
        mut group: HashMap<String, Vec<IndividualPermission>>,
        group_map: Arc<Mutex<HashMap<String, String>>>,
    ) -> Result<()> {
        // convert group name to group id
        group = group
            .into_iter()
            .map(|(k, v)| -> Result<_> {
                Ok((
                    super::group_lookup_from_mutex(Arc::clone(&group_map), &k)?,
                    v,
                ))
            })
            .collect::<Result<HashMap<_, _>>>()?;
        self.execute_to_unit_result(self.build_add_policy_request(asset, user, group)?)
            .await
    }

    /// Build a request to add a policy
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

    /// Build futures that will delete privileges
    pub(crate) fn build_delete_policy_request_futures<'a>(
        &'a self,
        state: &PolicyState,
        asset: &TableauAssetReference,
        user_id_or_group_name: &'a String,
        grantee_type: &'a str,
        group_map: Arc<Mutex<HashMap<String, String>>>,
    ) -> Result<Vec<BoxFuture<Result<()>>>> {
        let mut res: Vec<BoxFuture<Result<()>>> = Vec::new();
        for privilege in &state.privileges {
            res.push(Box::pin(self.build_and_execute_delete_policy_request(
                privilege.to_owned(),
                asset.to_owned(),
                user_id_or_group_name.to_owned(),
                grantee_type.to_owned(),
                Arc::clone(&group_map),
            )));
        }

        Ok(res)
    }

    /// Look up the relevant grantee ids, generate a request and execute that request
    /// to remove privileges
    pub(crate) async fn build_and_execute_delete_policy_request(
        &self,
        privilege: String,
        asset: TableauAssetReference,
        user_id_or_group_name: String,
        grantee_type: String,
        group_map: Arc<Mutex<HashMap<String, String>>>,
    ) -> Result<()> {
        let mut grantee_id = user_id_or_group_name.to_owned();

        if grantee_type == *"group" {
            grantee_id = super::group_lookup_from_mutex(group_map, &grantee_id)?;
        }

        let request =
            self.generate_delete_privilege_request(&asset, &grantee_id, &grantee_type, &privilege)?;

        self.execute_to_unit_result(request).await
    }

    /// generate the actual request to delete a privilege
    fn generate_delete_privilege_request(
        &self,
        asset: &TableauAssetReference,
        grantee_id: &String,
        grantee_type: &str,
        privilege: &String,
    ) -> Result<Request, anyhow::Error> {
        let permission = IndividualPermission::from_string(privilege);

        let request = self
            .coordinator
            .rest_client
            .build_request(
                format!(
                    "{}/{}/permissions/{grantee_type}s/{grantee_id}/{}/{}",
                    asset.asset_type.as_category_str(),
                    &asset.id,
                    permission.capability,
                    permission.mode.to_string()
                ),
                None,
                reqwest::Method::DELETE,
            )?
            .build()
            .context("building request")?;
        Ok(request)
    }

    /// Build a request to remove privileges
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
            .map(|p| self.generate_delete_privilege_request(asset, grantee_id, grantee_type, p))
            .collect()
    }
}

/// Generate the body of the request to add privileges
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
        request_text += "          \"capabilities\": {\n";
        request_text += "            \"capability\": [";
        request_text += permissions
            .iter()
            .map(|permission| {
                format!(
                    "            {{ \"name\": \"{}\", \"mode\": \"{}\" }}\n",
                    permission.capability,
                    permission.mode.to_string()
                )
            })
            .collect::<Vec<_>>()
            .join(",")
            .as_str();

        request_text += "            ]\n";
        request_text += "          }\n";
        request_text += "}\n"
    }
    request_text += "]\n";
    request_text += "}\n";
    request_text += "}\n";

    serde_json::from_str(&request_text).context("building request body")
}
