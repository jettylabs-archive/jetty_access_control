//! Functionality for handling group diffs in tableau

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::{bail, Context, Result};

use futures::future::BoxFuture;
use jetty_core::{
    access_graph::translate::diffs::default_policies, cual::Cual, logging::info,
    write::assets::PolicyState,
};
use reqwest::Request;
use serde_json::json;

use crate::{
    coordinator::TableauAssetReference, nodes::IndividualPermission, rest::TableauAssetType,
    TableauConnector,
};

use super::{SequencedFutures, SequencedPlans};

impl TableauConnector {
    /// Generate sequenced plan for changes (as part of `jetty plan` execution path)
    pub(crate) fn prepare_default_policies_plan(
        &self,
        policy_diffs: &Vec<default_policies::LocalDiff>,
    ) -> Result<SequencedPlans> {
        let mut plans = SequencedPlans::default();

        for diff in policy_diffs {
            let asset_reference = self.coordinator.env.cual_id_map.get(&diff.asset).unwrap();

            if asset_reference.asset_type != TableauAssetType::Project {
                bail!("problem generating plan for {}: default permissions can only be set at the project level", diff.asset.to_string());
            };

            let mut user_adds = HashMap::new();
            let mut group_adds = HashMap::new();

            try_wildcard_path_is_valid(&diff.path).context(format!(
                "problem generating plan for {}",
                &diff.asset.to_string()
            ))?;

            let asset_type = TableauAssetType::from_str(&diff.asset_type).context(format!(
                "problem generating plan for {}",
                &diff.asset.to_string()
            ))?;
            let asset_type = if asset_type == TableauAssetType::View {
                info!("default policies set for views are applied to workbooks (and then inherited by views); this may cause a policy conflict if the \
                workbook policy isn't also updated ({})", 
                &diff.asset.to_string());
                TableauAssetType::Workbook
            } else if asset_type == TableauAssetType::Workbook {
                info!("default policies set for workbooks are inherited by views; this may cause a policy conflict if the \
                view policy isn't also updated ({})", 
                &diff.asset.to_string());
                asset_type
            } else {
                asset_type
            };

            let mut set_tableau_content_permissions: Option<String> = None;

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

                        // catch changes to content permissions and check for errors
                        try_update_content_permissions(
                            &diff.asset,
                            &mut set_tableau_content_permissions,
                            add,
                            &asset_type,
                        )?;
                    }
                    jetty_core::write::assets::diff::policies::DiffDetails::RemoveAgent {
                        remove,
                    } => plans.1.extend(self.build_delete_default_policy_requests(
                        remove,
                        asset_reference,
                        user,
                        "user",
                        &asset_type,
                    )?),
                    jetty_core::write::assets::diff::policies::DiffDetails::ModifyAgent {
                        add,
                        remove,
                    } => {
                        // catch changes to content permissions and check for errors
                        try_update_content_permissions(
                            &diff.asset,
                            &mut set_tableau_content_permissions,
                            add,
                            &asset_type,
                        )?;

                        user_adds.insert(
                            user.to_owned(),
                            add.privileges
                                .iter()
                                .map(IndividualPermission::from_string)
                                .collect::<Vec<_>>(),
                        );
                        plans.1.extend(self.build_delete_default_policy_requests(
                            remove,
                            asset_reference,
                            user,
                            "user",
                            &asset_type,
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
                        // catch changes to content permissions and check for errors
                        try_update_content_permissions(
                            &diff.asset,
                            &mut set_tableau_content_permissions,
                            add,
                            &asset_type,
                        )?;
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
                    } => plans.1.extend(self.build_delete_default_policy_requests(
                        remove,
                        asset_reference,
                        &group_id,
                        "group",
                        &asset_type,
                    )?),
                    jetty_core::write::assets::diff::policies::DiffDetails::ModifyAgent {
                        add,
                        remove,
                    } => {
                        // catch changes to content permissions and check for errors
                        try_update_content_permissions(
                            &diff.asset,
                            &mut set_tableau_content_permissions,
                            add,
                            &asset_type,
                        )?;
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
                            plans.1.extend(self.build_delete_default_policy_requests(
                                remove,
                                asset_reference,
                                &group_id,
                                "group",
                                &asset_type,
                            )?);
                        }
                    }
                }
            }
            if !user_adds.is_empty() || !group_adds.is_empty() {
                plans.1.push(self.build_add_default_policy_request(
                    asset_reference,
                    user_adds,
                    group_adds,
                    &asset_type,
                )?);
            }

            if let Some(content_permissions) = set_tableau_content_permissions {
                plans.1.push(
                    self.generate_content_permissions_request(
                        asset_reference,
                        &content_permissions,
                    )?,
                )
            };
        }
        Ok(plans)
    }

    /// Generate the actual request futures for changes to be applied as part of `jetty apply`
    /// This is very similar to prepare_default_policies_plan(), but requires extra care because it needs
    /// to handle the id's of newly created groups. That means that the actual creation of must requests
    /// must be deferred.
    pub(super) fn generate_default_policy_apply_futures<'a>(
        &'a self,
        policy_diffs: &'a Vec<default_policies::LocalDiff>,
        group_map: Arc<Mutex<HashMap<String, String>>>,
    ) -> Result<SequencedFutures> {
        let mut futures = SequencedFutures::default();

        for diff in policy_diffs {
            let asset_reference = self.coordinator.env.cual_id_map.get(&diff.asset).unwrap();

            if asset_reference.asset_type != TableauAssetType::Project {
                bail!("problem generating plan for {}: default permissions can only be set at the project level", diff.asset.to_string());
            };

            let mut user_adds = HashMap::new();
            let mut group_adds = HashMap::new();

            try_wildcard_path_is_valid(&diff.path).context(format!(
                "problem generating plan for {}",
                &diff.asset.to_string()
            ))?;

            let asset_type = TableauAssetType::from_str(&diff.asset_type).context(format!(
                "problem generating plan for {}",
                &diff.asset.to_string()
            ))?;
            let asset_type = if asset_type == TableauAssetType::View {
                info!("default policies set for views are applied to workbooks (and then inherited by views); this may cause a policy conflict if the \
                workbook policy isn't also updated");
                TableauAssetType::Workbook
            } else if asset_type == TableauAssetType::Workbook {
                info!("default policies set for workbooks are inherited by views; this may cause a policy conflict if the \
                view policy isn't also updated ({})", 
                &diff.asset.to_string());
                asset_type
            } else {
                asset_type
            };

            let mut set_tableau_content_permissions: Option<String> = None;

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

                        // catch changes to content permissions and check for errors
                        try_update_content_permissions(
                            &diff.asset,
                            &mut set_tableau_content_permissions,
                            add,
                            &asset_type,
                        )?;
                    }
                    jetty_core::write::assets::diff::policies::DiffDetails::RemoveAgent {
                        remove,
                    } => self
                        .build_delete_default_policy_request_futures(
                            remove,
                            asset_reference,
                            user_id,
                            "user",
                            &asset_type,
                            group_map.clone(),
                        )?
                        .into_iter()
                        .for_each(|f| futures.1.push(f)),

                    jetty_core::write::assets::diff::policies::DiffDetails::ModifyAgent {
                        add,
                        remove,
                    } => {
                        // catch changes to content permissions and check for errors
                        try_update_content_permissions(
                            &diff.asset,
                            &mut set_tableau_content_permissions,
                            add,
                            &asset_type,
                        )?;

                        user_adds.insert(
                            user_id.to_owned(),
                            add.privileges
                                .iter()
                                .map(IndividualPermission::from_string)
                                .collect::<Vec<_>>(),
                        );
                        self.build_delete_default_policy_request_futures(
                            remove,
                            asset_reference,
                            user_id,
                            "user",
                            &asset_type,
                            group_map.clone(),
                        )?
                        .into_iter()
                        .for_each(|f| futures.1.push(f));
                    }
                }
            }
            for (group_name, details) in &diff.groups {
                match details {
                    jetty_core::write::assets::diff::policies::DiffDetails::AddAgent { add } => {
                        // catch changes to content permissions and check for errors
                        try_update_content_permissions(
                            &diff.asset,
                            &mut set_tableau_content_permissions,
                            add,
                            &asset_type,
                        )?;
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
                        .build_delete_default_policy_request_futures(
                            remove,
                            asset_reference,
                            group_name,
                            "group",
                            &asset_type,
                            group_map.clone(),
                        )?
                        .into_iter()
                        .for_each(|f| futures.1.push(f)),
                    jetty_core::write::assets::diff::policies::DiffDetails::ModifyAgent {
                        add,
                        remove,
                    } => {
                        // catch changes to content permissions and check for errors
                        try_update_content_permissions(
                            &diff.asset,
                            &mut set_tableau_content_permissions,
                            add,
                            &asset_type,
                        )?;
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
                            self.build_delete_default_policy_request_futures(
                                remove,
                                asset_reference,
                                group_name,
                                "group",
                                &asset_type,
                                group_map.clone(),
                            )?
                            .into_iter()
                            .for_each(|f| futures.1.push(f));
                        }
                    }
                }
            }
            if !user_adds.is_empty() || !group_adds.is_empty() {
                futures.1.push(Box::pin(
                    self.execute_add_default_policy_with_deferred_lookup(
                        asset_reference,
                        user_adds,
                        group_adds,
                        asset_type.to_owned(),
                        group_map.clone(),
                    ),
                ));
            }

            if let Some(content_permissions) = set_tableau_content_permissions {
                futures.1.push(Box::pin(self.execute_to_unit_result(
                    self.generate_content_permissions_request(
                        asset_reference,
                        &content_permissions,
                    )?,
                )));
            };
        }
        Ok(futures)
    }

    /// Creates and executes a request to add to defualt policies after looking up the relevant group id.
    async fn execute_add_default_policy_with_deferred_lookup(
        &self,
        asset: &TableauAssetReference,
        user: HashMap<String, Vec<IndividualPermission>>,
        mut group: HashMap<String, Vec<IndividualPermission>>,
        applied_to_asset_type: TableauAssetType,
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
        self.execute_to_unit_result(self.build_add_default_policy_request(
            asset,
            user,
            group,
            &applied_to_asset_type,
        )?)
        .await
    }

    /// Build requests that will delete privileges
    fn build_delete_default_policy_requests(
        &self,
        state: &PolicyState,
        asset: &TableauAssetReference,
        grantee_id: &String,
        grantee_type: &str,
        applied_to_asset_type: &TableauAssetType,
    ) -> Result<Vec<Request>> {
        if applied_to_asset_type == &TableauAssetType::Project {
            return self.build_delete_policy_requests(state, asset, grantee_id, grantee_type);
        }
        state
            .privileges
            .iter()
            .map(|p| {
                self.generate_delete_default_privilege_request(
                    p,
                    asset,
                    grantee_id,
                    grantee_type,
                    applied_to_asset_type,
                )
            })
            .collect()
    }

    /// generate the actual request to delete a default privilege
    fn generate_delete_default_privilege_request(
        &self,
        privilege: &String,
        asset: &TableauAssetReference,
        grantee_id: &String,
        grantee_type: &str,
        applied_to_asset_type: &TableauAssetType,
    ) -> Result<Request> {
        let permission = IndividualPermission::from_string(privilege);
        self.coordinator
            .rest_client
            .build_request(
                format!(
                    "projects/{}/default-permissions/{}/{grantee_type}s/{grantee_id}/{}/{}",
                    &asset.id,
                    applied_to_asset_type.as_category_str(),
                    &permission.capability,
                    &permission.mode.to_string()
                ),
                None,
                reqwest::Method::DELETE,
            )?
            .build()
            .context("building request")
    }

    /// build request to update content permissions
    fn generate_content_permissions_request(
        &self,
        asset: &TableauAssetReference,
        content_permissions: &String,
    ) -> Result<Request> {
        self.coordinator
            .rest_client
            .build_request(
                format!("projects/{}", asset.id,),
                Some(json!( {
                    "project": {
                        "contentPermissions": content_permissions
                    }
                })),
                reqwest::Method::PUT,
            )?
            .build()
            .context("building request")
    }

    /// Build a request to update default permissions
    fn build_add_default_policy_request(
        &self,
        asset: &TableauAssetReference,
        user: HashMap<String, Vec<IndividualPermission>>,
        group: HashMap<String, Vec<IndividualPermission>>,
        applied_to_asset_type: &TableauAssetType,
    ) -> Result<reqwest::Request> {
        // project default policies are just regular policies
        if applied_to_asset_type == &TableauAssetType::Project {
            return self.build_add_policy_request(asset, user, group);
        }
        let req_body = generate_add_default_privileges_request_body(
            asset,
            user,
            group,
            applied_to_asset_type,
        )?;
        self.coordinator
            .rest_client
            .build_request(
                format!(
                    "projects/{}/default-permissions/{}",
                    asset.id,
                    applied_to_asset_type.as_category_str()
                ),
                Some(req_body),
                reqwest::Method::PUT,
            )?
            .build()
            .context("building request")
    }

    /// Build futures that will delete privileges
    pub(crate) fn build_delete_default_policy_request_futures<'a>(
        &'a self,
        state: &PolicyState,
        asset: &TableauAssetReference,
        user_id_or_group_name: &'a String,
        grantee_type: &'a str,
        applied_to_asset_type: &TableauAssetType,
        group_map: Arc<Mutex<HashMap<String, String>>>,
    ) -> Result<Vec<BoxFuture<Result<()>>>> {
        // If it's a project, just generate the same request for a non-default policy
        if applied_to_asset_type == &TableauAssetType::Project {
            return self.build_delete_policy_request_futures(
                state,
                asset,
                user_id_or_group_name,
                grantee_type,
                group_map,
            );
        }

        let mut res: Vec<BoxFuture<Result<()>>> = Vec::new();
        for privilege in &state.privileges {
            res.push(Box::pin(
                self.build_and_execute_delete_default_policy_request(
                    privilege.to_owned(),
                    asset.to_owned(),
                    user_id_or_group_name.to_owned(),
                    grantee_type.to_owned(),
                    applied_to_asset_type.to_owned(),
                    Arc::clone(&group_map),
                ),
            ));
        }

        Ok(res)
    }

    /// Look up the relevant grantee ids, generate a request and execute that request
    /// to remove privileges.
    async fn build_and_execute_delete_default_policy_request(
        &self,
        privilege: String,
        asset: TableauAssetReference,
        user_id_or_group_name: String,
        grantee_type: String,
        applied_to_asset_type: TableauAssetType,
        group_map: Arc<Mutex<HashMap<String, String>>>,
    ) -> Result<()> {
        let mut grantee_id = user_id_or_group_name.to_owned();

        if grantee_type == *"group" {
            grantee_id = super::group_lookup_from_mutex(group_map, &grantee_id)?;
        }

        let request = self.generate_delete_default_privilege_request(
            &privilege,
            &asset,
            &grantee_id,
            &grantee_type,
            &applied_to_asset_type,
        )?;

        self.execute_to_unit_result(request).await
    }
}

/// Generate the request body needed to generate and add default permissions
fn generate_add_default_privileges_request_body(
    asset: &TableauAssetReference,
    user: HashMap<String, Vec<IndividualPermission>>,
    group: HashMap<String, Vec<IndividualPermission>>,
    applied_to_asset_type: &TableauAssetType,
) -> Result<serde_json::Value> {
    // project default policies are just regular policies
    if applied_to_asset_type == &TableauAssetType::Project {
        return super::policies::generate_add_privileges_request_body(asset, user, group);
    }

    let mut request_text = "{
    \"permissions\": {\n"
        .to_owned();
    request_text += "      \"granteeCapabilities\": [\n";
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
        request_text += "          ]\n";
        request_text += "        }\n"
    }
    for (group_id, permissions) in group.iter() {
        request_text += "        {\n";
        request_text += format!(
            "          \"group\": {{ \"id\": \"{}\" }},\n",
            group_id.to_owned()
        )
        .as_str();
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
        request_text += "        }\n"
    }
    request_text += "      ]\n";
    request_text += "    }\n";
    request_text += "  }\n";

    serde_json::from_str(&request_text).context("building request body")
}

/// Ensure that the connector-managed wildcard is legal
fn try_wildcard_path_is_valid(wildcard_path: &String) -> Result<()> {
    // right now, we only support unbounded wildcards as they best align with
    // tableau's default permissions
    match wildcard_path.trim_start_matches('/').trim_end_matches('/') == "**" {
        true => Ok(()),
        false => bail!(
            "illegal path for connector-managed default policy: got {wildcard_path}, expected /**"
        ),
    }
}

/// Ensure that the Tableau Content Permissiosn are updated and consistent across an asset
fn try_update_content_permissions(
    cual: &Cual,
    content_permissions: &mut Option<String>,
    state: &PolicyState,
    applied_to_asset_type: &TableauAssetType,
) -> Result<()> {
    // Only check for the metadata on projects
    if applied_to_asset_type == &TableauAssetType::Project {
        if let Some(p) = state.metadata.get("Tableau Content Permissions") {
            if content_permissions
                .to_owned()
                .and_then(|existing_value| {
                    if &existing_value == p {
                        None
                    } else {
                        Some(false)
                    }
                })
                .is_none()
            {
                *content_permissions = Some(p.to_owned());
            } else {
                bail!("problem generating plan for {}: Tableau Content Permissions must match for all default policies originating from a given asset", cual.to_string());
            };
        }
    }
    Ok(())
}
