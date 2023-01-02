//! Diffing for user configurations <-> Env

mod identity;
mod membership;

use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fmt::Display,
};

use colored::Colorize;
pub use identity::{get_identity_diffs, update_graph};
pub use membership::get_membership_diffs;

use crate::{access_graph::NodeName, jetty::ConnectorNamespace, write::SplitByConnector};

use self::{
    identity::{IdentityDiff, IdentityDiffDetails},
    membership::{MembershipDiff, MembershipDiffDetails},
};

/// Complete diffs for users
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct CombinedUserDiff {
    pub(crate) user: NodeName,
    identity: Option<IdentityDiffDetails>,
    pub(crate) group_membership: Option<MembershipDiffDetails>,
}

impl SplitByConnector for CombinedUserDiff {
    fn split_by_connector(&self) -> HashMap<ConnectorNamespace, Box<Self>> {
        let mut res = HashMap::new();

        let mut add_map: HashMap<ConnectorNamespace, BTreeSet<NodeName>> = HashMap::new();
        let mut remove_map: HashMap<ConnectorNamespace, BTreeSet<NodeName>> = HashMap::new();

        match &self.group_membership {
            Some(membership_details) => {
                add_map = membership_details.add.iter().fold(add_map, |mut acc, v| {
                    acc.entry(
                        v.get_group_origin()
                            .expect("must be a list of groups")
                            .to_owned(),
                    )
                    .and_modify(|groups| {
                        groups.insert(v.to_owned());
                    })
                    .or_insert_with(|| [v.to_owned()].into());
                    acc
                });

                remove_map = membership_details
                    .remove
                    .iter()
                    .fold(remove_map, |mut acc, v| {
                        acc.entry(
                            v.get_group_origin()
                                .expect("must be a list of groups")
                                .to_owned(),
                        )
                        .and_modify(|groups| {
                            groups.insert(v.to_owned());
                        })
                        .or_insert_with(|| [v.to_owned()].into());
                        acc
                    });
            }
            None => return Default::default(),
        };

        let mut keys: HashSet<_> = add_map.keys().collect();
        keys.extend(remove_map.keys().collect::<HashSet<_>>());

        for key in keys {
            res.insert(
                key.to_owned(),
                Box::new(CombinedUserDiff {
                    user: self.user.to_owned(),
                    identity: None,
                    group_membership: Some(MembershipDiffDetails {
                        add: add_map.get(key).cloned().unwrap_or_default(),
                        remove: remove_map.get(key).cloned().unwrap_or_default(),
                    }),
                }),
            );
        }

        res
    }
}

impl Display for CombinedUserDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut text = "".to_owned();

        // Need to show if the user is new, deleted, or modified
        if let Some(identity_details) = &self.identity {
            match identity_details {
                IdentityDiffDetails::Add { add } => {
                    text += format!("{}{}\n", "+ user: ".green(), self.user.to_string().green())
                        .as_str();
                    text += "  identity:\n";
                    for (conn, local_name) in add {
                        text +=
                            format!("{}", format!("  + {conn}: {local_name}\n").green()).as_str();
                    }
                }
                IdentityDiffDetails::Remove { remove } => {
                    text += format!("{}", format!("- user: {}\n", self.user).red()).as_str();
                    text += "  identity:\n";
                    for (conn, local_name) in remove {
                        text += &format!("{}", format!("    - {conn}: {local_name}\n").red());
                    }
                }
                IdentityDiffDetails::Modify { add, remove } => {
                    text += format!(
                        "{}{}\n",
                        "~ user: ".yellow(),
                        self.user.to_string().yellow()
                    )
                    .as_str();
                    text += "  identity:\n";
                    for (conn, local_name) in add {
                        text +=
                            format!("{}", format!("    + {conn}: {local_name}\n").green()).as_str();
                    }
                    for (conn, local_name) in remove {
                        text +=
                            format!("{}", format!("    - {conn}: {local_name}\n").red()).as_str();
                    }
                }
            }
        }
        // if identity is None, groups must me Some
        else {
            text += format!(
                "{}{}\n",
                "~ user: ".yellow(),
                self.user.to_string().yellow()
            )
            .as_str();
        }

        if let Some(group_details) = &self.group_membership {
            text += "  groups:\n";
            for group in &group_details.add {
                text += format!("{}", format!("    + {group}\n").green()).as_str();
            }
            for group in &group_details.remove {
                text += format!("{}", format!("    - {group}\n").red()).as_str();
            }
        }

        write!(f, "{text}")
    }
}

/// Given the the identity and membership diffs, combine them into user-level diffs
pub fn combine_diffs(
    identity_diffs: &HashSet<IdentityDiff>,
    membership_diffs: &HashSet<MembershipDiff>,
) -> BTreeSet<CombinedUserDiff> {
    // create a set of all the keys from both
    let all_diff_users = get_all_diff_users(identity_diffs, membership_diffs);
    let identity_map: HashMap<_, _> = identity_diffs
        .iter()
        .map(|d| (&d.user, &d.details))
        .collect();
    let membership_map: HashMap<_, _> = membership_diffs
        .iter()
        .map(|d| (&d.user, &d.details))
        .collect();
    // iterate over them to create the new diffs
    all_diff_users
        .into_iter()
        .map(|u| CombinedUserDiff {
            user: u.to_owned(),
            group_membership: membership_map.get(&u).cloned().cloned(),
            identity: identity_map.get(&u).cloned().cloned(),
        })
        .collect()
}

/// Get all the users that have any changes at all
fn get_all_diff_users(
    identity_diffs: &HashSet<IdentityDiff>,
    membership_diffs: &HashSet<MembershipDiff>,
) -> HashSet<NodeName> {
    identity_diffs
        .iter()
        .map(|diff| diff.user.to_owned())
        .collect::<HashSet<_>>()
        .union(
            &membership_diffs
                .iter()
                .map(|diff| diff.user.to_owned())
                .collect::<HashSet<_>>(),
        )
        .cloned()
        .collect()
}
