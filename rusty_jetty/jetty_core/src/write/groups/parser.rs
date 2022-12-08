use std::collections::BTreeMap;

use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;
use yaml_peg::{parse, repr::RcRepr};

use crate::{
    jetty::ConnectorNamespace,
    write::parser_common::{get_optional_bool, get_optional_string, indicated_msg},
};

use super::{ConnectorName, GroupConfig, GroupMembers, MemberGroup, MemberUser};

/// Parse a string tag config into a map of tag configuration objects
pub(crate) fn parse_groups(config: &String) -> Result<BTreeMap<String, GroupConfig>> {
    // Parse into Node representation and selecting the first element in the Vec
    let root = &parse::<RcRepr>(config).context("unable to parse tags file - invalid yaml")?[0];

    // Return an empty BTreeMap if there are no tags
    if root.is_null() {
        return Ok(Default::default());
    }
    let mapped_root = root
        .as_map()
        .map_err(|_| anyhow!("improperly formatted groups file: should be a dictionary of group names and configurations \
        - please refer to the documentation for more information"))?;

    let mut groups = BTreeMap::new();

    // iterate over each item in the map
    for (k, v) in mapped_root {
        let group_pos = k.pos();
        let group_name = k
            .as_str()
            .map_err(|_| {
                anyhow!(
                    "group name is not a string: {}",
                    indicated_msg(config.as_bytes(), k.pos(), 2)
                )
            })?
            .to_owned();

        // get the names map
        let connector_names_node = v.get("names");
        // first map checks to see if the names map exists
        let connector_names = match connector_names_node {
            Ok(names) => {
                // now check to see if it's a map
                let names = names.as_map().map_err(|_| anyhow!("improperly formatted groups file: names field for {group_name} should be a map of namespace: string \
                - please refer to the documentation for more information"))?;
                let mut result_vec = Vec::new();
                // names should be listed as a map of string -> string
                for (k, v) in names {
                    result_vec.push(ConnectorName {
                        connector: ConnectorNamespace(k.as_str().map_err(|_| anyhow!("improperly formatted groups file: names field for {group_name} should be a map of namespace: string \
                        - please refer to the documentation for more information"))?.to_owned()),
                        alias: v.as_str().map_err(|_| anyhow!("improperly formatted groups file: names field for {group_name} should be a map of namespace: string \
                        - please refer to the documentation for more information"))?.to_owned(),
                        pos: k.pos(),
                    })
                }
                Some(result_vec)
            }
            Err(_) => None,
        };

        // get the members map
        let members_node = v.get("members");
        let members = match members_node {
            Ok(ok_members) => {
                // Get the users
                let users = ok_members.get("users").ok();
                let users = match users {
                    Some(u) => {
                        if let Ok(Ok(user_vec)) = u.as_seq().map(|seq| {
                            seq.iter()
                                .map(|v| {
                                    v.as_str().map(|s| MemberUser {
                                        name: s.to_owned(),
                                        pos: v.pos(),
                                    })
                                })
                                .collect::<Result<Vec<_>, u64>>()
                        }) {
                            Some(user_vec)
                        } else {
                            bail!("improperly formatted groups file: members.users field in {group_name} should be a sequence of strings")
                        }
                    }
                    None => None,
                };
                let member_groups = ok_members.get("groups").ok();
                let member_groups = match member_groups {
                    Some(u) => {
                        if let Ok(Ok(user_vec)) = u.as_seq().map(|seq| {
                            seq.iter()
                                .map(|v| {
                                    v.as_str().map(|s| MemberGroup {
                                        name: s.to_owned(),
                                        pos: v.pos(),
                                    })
                                })
                                .collect::<Result<Vec<_>, u64>>()
                        }) {
                            Some(user_vec)
                        } else {
                            bail!("improperly formatted groups file: members.groups field in {group_name} should be a sequence of strings")
                        }
                    }
                    None => None,
                };
                GroupMembers {
                    groups: member_groups,
                    users,
                }
            }
            Err(_) => {
                bail!("improperly formatted groups file: members field must exist for {group_name}")
            }
        };

        groups.insert(
            group_name.clone(),
            GroupConfig {
                name: group_name,
                connector_names,
                members,
                pos: group_pos,
            },
        );
    }

    Ok(groups)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_groups_works() -> Result<()> {
        let group_config = r#"
All Analysts:
    names:
        snow: ANALYSTS
        tab: cheese puffs
    members:
        groups:
            - Sales Analysts
            - Product Analysts
            - Data Engineering
        users:
            - isaac.hales@gmail.com
            - jk@get-jetty.com

Sales Analysts:
    members:
        users:
            - mark@thefacebook.com
            - elliot@allsafe.com

"#;
        dbg!(parse_groups(&group_config.to_owned()));
        Ok(())
    }
}
