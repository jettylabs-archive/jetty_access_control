//! Functionality for handling group diffs in tableau

use std::collections::HashMap;

use anyhow::{anyhow, bail, Context, Result};

use jetty_core::{
    access_graph::translate::diffs::default_policies, cual::Cual, write::assets::PolicyState,
};
use reqwest::Request;
use serde_json::json;

use crate::{
    coordinator::TableauAssetReference, nodes::IndividualPermission, rest::TableauAssetType,
    TableauConnector,
};

use super::PrioritizedPlans;

impl TableauConnector {
    pub(crate) fn prepare_default_policies_plan(
        &self,
        policy_diffs: &Vec<default_policies::LocalDiff>,
    ) -> Result<PrioritizedPlans> {
        let mut plans = PrioritizedPlans::default();

        let base_url = format![
            "https://{}/api/{}/sites/{}",
            self.coordinator.rest_client.get_server_name(),
            self.coordinator.rest_client.get_api_version(),
            self.coordinator.rest_client.get_site_id()?,
        ];

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

            let mut set_tableau_content_permissions: Option<String> = None;

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

                        // catch changes to content permissions and check for errors
                        try_update_content_permissions(
                            &diff.asset,
                            &mut set_tableau_content_permissions,
                            add,
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
                        )?;

                        user_adds.insert(
                            user.to_owned(),
                            add.privileges
                                .iter()
                                .map(|p| IndividualPermission::from_string(p))
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
                    .unwrap_or(format!("<group_id name for new group: {}>", group));
                match details {
                    jetty_core::write::assets::diff::policies::DiffDetails::AddAgent { add } => {
                        // catch changes to content permissions and check for errors
                        try_update_content_permissions(
                            &diff.asset,
                            &mut set_tableau_content_permissions,
                            add,
                        )?;
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
                        )?;
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
                let permission = IndividualPermission::from_string(p);
                self.coordinator
                    .rest_client
                    .build_request(
                        format!(
                            "projects/{}/default-permissions/{}/{grantee_type}/{grantee_id}/{}/{}",
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
            })
            .collect()
    }

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
}

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
) -> Result<()> {
    match state.metadata.get("Tableau Content Permissions") {
        Some(p) => {
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
        None => (),
    }
    Ok(())
}
