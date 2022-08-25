use anyhow::Result;
use jetty_core::{fetch_credentials, Connector, Jetty};
use jetty_snowflake::{Role, User};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let jetty = Jetty::new()?;
    println!("{:#?}", jetty.config);
    let creds = fetch_credentials()?;
    let snow = jetty_snowflake::Snowflake::new(&jetty.config.connectors[0], &creds["snow"])?;
    println!("checking for connection...");
    println!("working? {}", snow.check().await);

    println!("{:#?}", snow.query_to_obj::<Role>("SHOW ROLES").await?);
    let users = snow.query_to_obj::<User>("SHOW USERS").await.unwrap();
    println!("{:#?}", users);

    Ok(())
}
