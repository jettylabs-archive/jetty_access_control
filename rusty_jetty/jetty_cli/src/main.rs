use anyhow::Result;
use jetty_core::snowflake;
use jetty_core::{fetch_credentials, Connector, JettyConfig};
use tokio;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let config = JettyConfig::new()?;
    println!("{:#?}", config);
    let creds = fetch_credentials()?;
    let snow = snowflake::Snowflake::new(&config.connectors[0], &creds["snow"])?;
    snow.check().await;

    Ok(())
}
