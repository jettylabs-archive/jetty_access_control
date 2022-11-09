//! Telemetry utils for tracking usage.
//!

use std::time::Instant;

use anyhow::Result;
use time::{format_description::well_known::Iso8601, OffsetDateTime};

const SCHEMA_VERSION: &str = "0.0.1";
const VERSION: &str = env!("CARGO_PKG_VERSION");

enum Platform {
    Windows,
    Linux,
    Mac,
    Unknown,
}

impl Platform {
    fn get() -> Self {
        if cfg!(target_os = "windows") {
            Platform::Windows
        } else if cfg!(target_os = "linux") {
            Platform::Linux
        } else if cfg!(target_os = "macos") {
            Platform::Mac
        } else {
            Platform::Unknown
        }
    }
}

struct JettyUserId(String);

impl JettyUserId {
    fn get() -> Self {
        // Get the user ID from the local file. Or create one and return it.
        todo!()
    }
}

struct Invocation {
    time: OffsetDateTime,
    user_id: JettyUserId,
    jetty_version: String,
    schema_version: String,
    platform: Platform,
}

impl Invocation {
    fn new() -> Self {
        Invocation {
            user_id: JettyUserId(String::new()),
            time: OffsetDateTime::now_utc(),
            jetty_version: get_jetty_version(),
            schema_version: SCHEMA_VERSION.to_owned(),
            platform: Platform::get(),
        }
    }

    fn publish(&self) -> Result<()> {
        self.time.format(&Iso8601::DEFAULT)?;
        Ok(())
    }
}

/// An event representing a single invocation of Jetty.
pub enum UsageEvent {
    /// No args
    Default,
    /// `jetty init`
    Init,
    /// `jetty fetch`
    Fetch,
    /// `jetty explore`
    Explore,
    /// `jetty help` or `jetty --help` or `jetty -h`
    Help,
    /// Program panicked during execution.
    Panic,
}

/// Given an event, record its usage to Jetty telemetry.
pub fn record_usage(event: UsageEvent) -> Result<()> {
    match event {
        UsageEvent::Default => todo!(),
        UsageEvent::Init => todo!(),
        UsageEvent::Fetch => todo!(),
        UsageEvent::Explore => todo!(),
        UsageEvent::Help => todo!(),
        UsageEvent::Panic => todo!(),
    }
}

/// Publish the given event to Firebase.
fn publish_event() {}

fn get_jetty_version() -> String {
    VERSION
}
