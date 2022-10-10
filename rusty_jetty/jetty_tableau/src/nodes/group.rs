use std::collections::{HashMap, HashSet};

use crate::nodes as tableau_nodes;
use crate::rest::{self, FetchJson};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use jetty_core::connectors::nodes as jetty_nodes;

/// Representation of a Group
#[derive(Clone, Default, Debug, Deserialize, Serialize, Hash)]
pub(crate) struct Group {
    pub id: String,
    pub name: String,
    /// Vec of user uids
    pub includes: Vec<tableau_nodes::User>,
}

impl Group {
    /// Update group membership
    pub(crate) async fn update_users(
        &mut self,
        tc: &rest::TableauRestClient,
        users: &HashMap<String, tableau_nodes::User>,
    ) -> Result<()> {
        let resp = tc
            .build_request(
                format!("groups/{}/users", self.id),
                None,
                reqwest::Method::GET,
            )
            .context("fetching group membership")?
            .fetch_json_response(Some(vec!["users".to_owned(), "user".to_owned()]))
            .await
            .context(format!("getting membership for group {}", self.name))?;

        let user_ids: Vec<super::IdField> =
            serde_json::from_value(resp).context("parsing group membership")?;
        let group_users = user_ids
            .iter()
            .map(|uid| {
                users.get(&uid.id).unwrap_or_else(|| {
                    panic!("user id {:?} not in tableau users {:?}", uid.id, users)
                })
            })
            .cloned()
            .collect();
        self.includes = group_users;
        Ok(())
    }
}

/// Convert JSON Value to a Group instance
impl Group {
    pub(crate) fn new(id: String, name: String, includes: Vec<tableau_nodes::User>) -> Self {
        Self { id, name, includes }
    }
}

impl From<Group> for jetty_nodes::Group {
    fn from(val: Group) -> Self {
        jetty_nodes::Group::new(
            val.name.to_owned(),
            HashMap::from([("tableau::id".to_owned(), val.id)]),
            // No nested groups in tableau
            HashSet::new(),
            val.includes.iter().map(|u| u.email.to_owned()).collect(),
            // No nested groups in tableau?
            HashSet::new(),
            // Handled in permissions/policies.
            HashSet::new(),
        )
    }
}

pub(crate) fn to_node(val: &serde_json::Value) -> Result<Group> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct GroupInfo {
        name: String,
        id: String,
    }
    let group_info: GroupInfo =
        serde_json::from_value(val.to_owned()).context("parsing group information")?;

    Ok(Group {
        id: group_info.id,
        name: group_info.name,
        includes: Vec::new(),
    })
}

/// Get basic group information. Excludes "includes" (group membership)
pub(crate) async fn get_basic_groups(
    tc: &rest::TableauRestClient,
) -> Result<HashMap<String, Group>> {
    let node = tc
        .build_request("groups".to_owned(), None, reqwest::Method::GET)
        .context("fetching groups")?
        .fetch_json_response(Some(vec!["groups".to_owned(), "group".to_owned()]))
        .await?;
    super::to_asset_map(tc, node, &to_node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};

    #[tokio::test]
    async fn test_fetching_groups_works() -> Result<()> {
        let tc = crate::connector_setup()
            .await
            .context("running tableau connector setup")?;
        let groups = get_basic_groups(&tc.coordinator.rest_client).await?;
        for (_k, v) in groups {
            println!("{:#?}", v);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_fetching_groups_with_users_works() -> Result<()> {
        let tc = crate::connector_setup()
            .await
            .context("running tableau connector setup")?;
        let mut groups = get_basic_groups(&tc.coordinator.rest_client).await?;
        for (_k, v) in &mut groups {
            v.update_users(
                &tc.coordinator.rest_client,
                &HashMap::from([(
                    "u".to_owned(),
                    tableau_nodes::User::new(
                        "id".to_owned(),
                        "name".to_owned(),
                        "email".to_owned(),
                        "eauid".to_owned(),
                        "full name".to_owned(),
                        Default::default(),
                    ),
                )]),
            )
            .await?;
            println!("{:#?}", v);
        }
        Ok(())
    }

    #[test]
    #[allow(unused_must_use)]
    fn test_jetty_group_from_group_works() {
        let g = Group::new(
            "id".to_owned(),
            "name".to_owned(),
            vec![tableau_nodes::User::new(
                "id".to_owned(),
                "name".to_owned(),
                "email".to_owned(),
                "eauid".to_owned(),
                "full name".to_owned(),
                Default::default(),
            )],
        );
        jetty_nodes::Group::from(g);
    }

    #[test]
    #[allow(unused_must_use)]
    fn test_group_into_jetty_group_works() {
        let g = Group::new(
            "id".to_owned(),
            "name".to_owned(),
            vec![tableau_nodes::User::new(
                "id".to_owned(),
                "name".to_owned(),
                "email".to_owned(),
                "eauid".to_owned(),
                "full name".to_owned(),
                Default::default(),
            )],
        );
        Into::<jetty_nodes::Group>::into(g);
    }

    #[test]
    fn test_group_with_users_into_jetty_group_gets_email() {
        let email = "email@email.email";
        let g = Group::new(
            "id".to_owned(),
            "name".to_owned(),
            vec![tableau_nodes::User::new(
                "id".to_owned(),
                "name".to_owned(),
                email.to_owned(),
                "eauid".to_owned(),
                "full name".to_owned(),
                Default::default(),
            )],
        );
        let a: jetty_nodes::Group = g.into();
        assert_eq!(a.includes_users, HashSet::from([email.to_owned()]));
    }
}
