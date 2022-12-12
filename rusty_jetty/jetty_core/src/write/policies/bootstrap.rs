//! Bootstrap policies from the generated graph into a yaml file

use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use serde::Serialize;

use crate::Jetty;

#[derive(Serialize, Debug)]
struct YamlPolicy {
    assets: String,
    agents: BTreeSet<String>,
    description: String,
    metadata: Option<BTreeMap<String, String>>,
    privileges: BTreeSet<String>,
}

impl Jetty {
    fn build_bootstrapped_policy_config(&self) -> Result<BTreeMap<String, YamlPolicy>> {
        let mut res = BTreeMap::new();

        // Get all the policies
        // Get the of the asset (there should be only one)
        // Get the agent (there should only be one)
        // Get the metadata
        // Get the privileges

        // Fold based on the name of the asset and the privileges -> If they are equal, create
        // a single entry

        // Pull in default policies
        // Get all the assets they touch

        // Prioritize overlapping default and standard policies

        // Use a description

        Ok(res)
    }
}
