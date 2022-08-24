use anyhow::Result;
use jetty_core::{fetch_credentials, Connector, Jetty};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let jetty = Jetty::new()?;
    println!("{:#?}", jetty.config);
    let creds = fetch_credentials()?;
    let snow = jetty_snowflake::Snowflake::new(&jetty.config.connectors[0], &creds["snow"])?;
    println!("checking for connection...");
    println!("working? {}", snow.check().await);

    println!("{:#?}", snow.get_roles().await?);
    let users = snow.get_users().await.unwrap();
    println!("{:#?}", users);

    Ok(())
}
