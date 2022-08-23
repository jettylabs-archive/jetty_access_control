use anyhow::Result;
use jetty_core::snowflake;
use jetty_core::{fetch_credentials, Connector, Jetty};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let jetty = Jetty::new()?;
    println!("{:#?}", jetty.config);
    let creds = fetch_credentials()?;
    let snow = snowflake::Snowflake::new(&jetty.config.connectors[0], &creds["snow"])?;
    println!("checking for connection...");
    snow.check().await;

    Ok(())
}
