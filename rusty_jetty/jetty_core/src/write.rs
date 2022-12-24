//! Write user-configured groups and permissions back to the data stack.

pub mod assets;
pub mod groups;
pub mod new_groups;
mod parser_common;
pub(crate) mod tag_parser;
pub mod users;
mod utils;

use std::collections::HashMap;

pub use groups::get_group_diff;

use crate::jetty::ConnectorNamespace;

/// A collection of diffs
pub struct Diffs {
    /// All the group-level diffs
    pub groups: Vec<groups::Diff>,
}

impl Diffs {
    /// Split diffs into a HashMap of diffs, by connector
    pub fn split_by_connector(&self) -> HashMap<ConnectorNamespace, Diffs> {
        // go through each field of Diffs, starting with groups
        let group_maps = self.groups.iter().fold(HashMap::new(), |mut acc, x| {
            acc.entry(&x.connector)
                .and_modify(|vec_diffs: &mut Vec<groups::Diff>| vec_diffs.push(x.to_owned()))
                .or_insert(vec![x.to_owned()]);
            acc
        });

        let mut res: HashMap<ConnectorNamespace, Diffs> = HashMap::new();

        // now put the groups into the proper fields
        for (conn, diffs) in group_maps {
            res.entry(conn.clone())
                .and_modify(|the_diff| the_diff.groups = diffs.clone())
                .or_insert(Diffs { groups: diffs });
        }

        res
    }
}
