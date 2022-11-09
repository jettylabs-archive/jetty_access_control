// //! Telemetry utils for tracking usage.
// //!

// use firebase_rs::Firebase;
// use lazy_static::lazy_static;

// use anyhow::Result;
// use serde::{Deserialize, Serialize};
// use time::{format_description::well_known::Iso8601, OffsetDateTime};

// const SCHEMA_VERSION: &str = "0.0.1";
// const JETTY_VERSION: &str = env!("CARGO_PKG_VERSION");

// lazy_static! {
//     static ref FIREBASE: Firebase =
//         // Firebase::new("https://jetty-cli-telemetry-default-rtdb.firebaseio.com/").unwrap();
//         Firebase::new("https://jetty-cli-telemetry.firebaseapp.com").unwrap();
// }

// #[derive(Deserialize, Serialize, Debug)]
// enum Platform {
//     Windows,
//     Linux,
//     Mac,
//     Unknown,
// }

// impl Platform {
//     fn get() -> Self {
//         if cfg!(target_os = "windows") {
//             Platform::Windows
//         } else if cfg!(target_os = "linux") {
//             Platform::Linux
//         } else if cfg!(target_os = "macos") {
//             Platform::Mac
//         } else {
//             Platform::Unknown
//         }
//     }
// }

// #[derive(Deserialize, Serialize, Debug)]
// #[serde(transparent)]
// struct JettyUserId(String);

// impl JettyUserId {
//     fn get() -> Self {
//         // Get the user ID from the local file. Or create one and return it.
//         todo!()
//     }
// }

// #[derive(Deserialize, Serialize, Debug)]
// struct Invocation {
//     time: Option<String>,
//     user_id: JettyUserId,
//     jetty_version: String,
//     schema_version: String,
//     platform: Platform,
//     event:UsageEvent,
// }

// impl Invocation {
//     fn new(event:UsageEvent) -> Self {
//         Invocation {
//             user_id: JettyUserId(String::new()),
//             time: OffsetDateTime::now_utc().format(&Iso8601::DEFAULT).ok(),
//             jetty_version: JETTY_VERSION.to_owned(),
//             schema_version: SCHEMA_VERSION.to_owned(),
//             platform: Platform::get(),
//             event,
//         }
//     }

//     async fn publish(&self) -> Result<()> {
//         let telemetry_ref = FIREBASE.at("telemetry");
//         telemetry_ref.set(self).await;
//         Ok(())
//     }
// }



// /// An event representing a single invocation of Jetty.
// pub enum UsageEvent {
//     /// No args
//     Default,
//     /// `jetty init`
//     Init,
//     /// `jetty fetch`
//     Fetch,
//     /// `jetty explore`
//     Explore,
//     /// `jetty help` or `jetty --help` or `jetty -h`
//     Help,
//     /// Program panicked during execution.
//     Panic,
// }

// /// Given an event, record its usage to Jetty telemetry.
// pub fn record_usage(event: UsageEvent) -> Result<()> {
//     Invocation::new(event).publish()
// }

// /// Publish the given event to Firebase.
// fn publish_event() {}
