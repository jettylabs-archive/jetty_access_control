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

use crate::access_graph::NodeName;

use self::{
    identity::{IdentityDiff, IdentityDiffDetails},
    membership::{MembershipDiff, MembershipDiffDetails},
};

/// Complete diffs for users
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CombinedUserDiff {
    user: NodeName,
    identity: Option<IdentityDiffDetails>,
    group_membership: Option<MembershipDiffDetails>,
}

impl Display for CombinedUserDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut text = "".to_owned();

        // Need to show if the user is new, deleted, or modified
        if let Some(identity_details) = &self.identity {
            match identity_details {
                IdentityDiffDetails::AddUser { add } => {
                    text += format!("{}{}\n", "+ user: ".green(), self.user.to_string().green())
                        .as_str();
                    text += "  identity:\n";
                    for (conn, local_name) in add {
                        text +=
                            format!("{}", format!("  + {conn}: {local_name}\n").green()).as_str();
                    }
                }
                IdentityDiffDetails::RemoveUser { remove } => {
                    text += format!("{}", format!("- user: {}\n", self.user.to_string()).red())
                        .as_str();
                    text += "  identity:\n";
                    for (conn, local_name) in remove {
                        text += &format!("{}", format!("    - {conn}: {local_name}\n").red());
                    }
                }
                IdentityDiffDetails::ModifyUser { add, remove } => {
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