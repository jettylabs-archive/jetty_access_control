//! Jetty CLI – Python-Wrapped Version
//!

#![deny(missing_docs)]

#[tokio::main]
fn main() -> Result<()> {
    cli().await?
}
