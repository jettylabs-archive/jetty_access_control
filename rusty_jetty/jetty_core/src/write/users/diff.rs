//! Diffing for user configurations <-> Env

use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;

use crate::Jetty;

use super::UserYaml;

fn diff_identities(validated_configs: &HashMap<PathBuf, UserYaml>, jetty: &Jetty) -> Result<()> {
    Ok(())
}
