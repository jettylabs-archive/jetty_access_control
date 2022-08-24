//! Snowflake Connector
//!
//! Everything needed for connection and interaction with Snowflake.&
//!
//! ```
//! use jetty_core::snowflake::Snowflake;
//! use jetty_core::connectors::Connector;
//! use jetty_core::jetty::{ConnectorConfig, CredentialsBlob};
//!
//! fn main(){
//!     let config = ConnectorConfig::default();
//!     let credentials = CredentialsBlob::default();
//!     let snow = Snowflake::new(&config, &credentials);
//! }
//! ```

mod consts;
mod grant;
mod role;
mod snowflake;
mod snowflake_query;
mod user;

pub use grant::Grant;
pub use role::Role;
pub use snowflake::*;
pub use user::User;
