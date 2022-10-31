//! Jetty CLI
//!

#![deny(missing_docs)]

use std::{path::PathBuf, sync::Arc, time::Instant};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};

use jetty_core::{
    access_graph::AccessGraph,
    connectors::ConnectorClient,
    fetch_credentials,
    jetty::ConnectorNamespace,
    logging::{self, debug, info, warn, LevelFilter},
    Connector, Jetty,
};
use jetty_lib::project;

#[tokio::main]
async fn main() -> Result<()> {
    cli().await?
}
