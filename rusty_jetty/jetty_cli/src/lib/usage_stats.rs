//! Anonymous usage stats utils for tracking Jetty usage.
//!

use std::fs;

use firestore::{self, errors::FirestoreError, FirestoreDb};
use jetty_core::{jetty::JettyConfig, logging::debug};
use once_cell::sync::OnceCell;

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use time::{format_description::well_known::Iso8601, OffsetDateTime};
use uuid::Uuid;

const SCHEMA_VERSION: &str = "0.0.1";
const JETTY_VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) static FIRESTORE: OnceCell<Result<FirestoreDb, FirestoreError>> = OnceCell::new();

#[derive(Deserialize, Serialize, Debug)]
enum Platform {
    #[serde(rename = "windows")]
    Windows,
    #[serde(rename = "linux")]
    Linux,
    #[serde(rename = "mac")]
    Mac,
    #[serde(rename = "unknown")]
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

#[derive(Deserialize, Serialize, Debug)]
enum RuntimeEnvironment {
    #[serde(rename = "prod")]
    Prod,
    #[serde(rename = "dev")]
    Dev,
    #[serde(rename = "test")]
    Test,
}

impl RuntimeEnvironment {
    fn get() -> Self {
        if cfg!(debug_assertions) {
            RuntimeEnvironment::Dev
        } else if cfg!(test) {
            RuntimeEnvironment::Test
        } else {
            RuntimeEnvironment::Prod
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(transparent)]
pub(crate) struct JettyProjectId(String);

#[derive(Deserialize, Serialize, Debug)]
#[serde(transparent)]
pub(crate) struct JettyUserId(String);

impl JettyUserId {
    fn get() -> Result<Self> {
        // Get the user ID from the local file. Or create one and return it.
        let user_id_file = crate::project::user_id_file();
        let user_id = match fs::read_to_string(&user_id_file) {
            Ok(contents) => JettyUserId(contents),
            Err(e) => {
                match e.kind() {
                    std::io::ErrorKind::NotFound => {
                        debug!("User ID file not found: {:?}", &user_id_file);
                        // Create it.
                        let user_id = Uuid::new_v4().to_string();
                        fs::write(user_id_file, &user_id).expect("Writing user id file failed.");
                        JettyUserId(user_id)
                    }
                    _ => {
                        // Fail
                        debug!("{:?}", e);
                        bail!("Failed to read user id file.")
                    }
                }
            }
        };
        Ok(user_id)
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct Invocation {
    created: String,
    user_id: JettyUserId,
    project_id: Option<JettyProjectId>,
    jetty_version: String,
    schema_version: String,
    platform: Platform,
    event: UsageEvent,
    environment: RuntimeEnvironment,
}

impl Invocation {
    fn new(event: UsageEvent, jetty_config: &Option<JettyConfig>) -> Result<Self> {
        let user_id = JettyUserId::get().context("Getting user id")?;
        let project_id = jetty_config
            .as_ref()
            .map(|cfg| JettyProjectId(cfg.project_id.to_owned()));
        let created = OffsetDateTime::now_utc()
            .format(&Iso8601::DEFAULT)
            .unwrap_or_else(|_| Default::default());
        Ok(Invocation {
            user_id,
            project_id,
            created,
            jetty_version: JETTY_VERSION.to_owned(),
            schema_version: SCHEMA_VERSION.to_owned(),
            platform: Platform::get(),
            event,
            environment: RuntimeEnvironment::get(),
        })
    }

    async fn publish(&self) -> Result<()> {
        FIRESTORE
            .get()
            .ok_or(anyhow!("Firestore was not initialized"))?
            .as_ref()?
            .fluent()
            .insert()
            .into("jetty_telemetry")
            .generate_document_id()
            .object(self)
            .execute()
            .await
            .context("writing telemetry document")
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "name", content = "properties")]
/// An event representing a single invocation of Jetty.
pub enum UsageEvent {
    /// No args
    #[serde(rename = "invoked_default")]
    InvokedDefault,
    /// `jetty init`
    #[serde(rename = "invoked_init")]
    InvokedInit,
    /// `jetty fetch`
    #[serde(rename = "invoked_fetch")]
    InvokedFetch { connector_types: Vec<String> },
    /// `jetty explore`
    #[serde(rename = "invoked_explore")]
    InvokedExplore,
    /// `jetty help` or `jetty --help` or `jetty -h`
    #[serde(rename = "invoked_help")]
    InvokedHelp,
    /// Program panicked during execution.
    #[serde(rename = "panicked")]
    InvokedPanic,
}

/// Given an event, record its usage to Jetty anonymous usage stats.
pub async fn record_usage(event: UsageEvent, jetty_config: &Option<JettyConfig>) -> Result<()> {
    if let Some(cfg) = jetty_config {
        if !cfg.allow_anonymous_usage_statistics {
            // Collection is disabled.
            return Ok(());
        }
    }
    Invocation::new(event, jetty_config)
        .context("Creating anonymous usage statistics invocation.")?
        .publish()
        .await
}
