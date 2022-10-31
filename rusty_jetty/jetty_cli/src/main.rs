//! Jetty CLI
//!

#![deny(missing_docs)]

use anyhow::Result;

use jetty_lib::cli;

#[tokio::main]
async fn main() -> Result<()> {
    cli().await
}
