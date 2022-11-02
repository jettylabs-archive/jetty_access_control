//! Logging utilities for Jetty-wide output to stdout.
//!

// Re-exports for convenience
pub use tracing::metadata::LevelFilter;
pub use tracing::{debug, error, info, warn};
use tracing_subscriber::prelude::*;
use tracing_subscriber::{util::SubscriberInitExt, Layer};

/// Set up basic logging.
///
/// The caller can specify a log level via `level`. If they don't, we
/// default to "info."
///
/// The `level` arg is overridden by any env var levels.
///
/// The user can specify a log level via the env var `RUST_LOG` (such as for testing).
/// If they don't, then we default to the level_filter defined above.
pub fn setup(level: Option<LevelFilter>) {
    let level_filter = level.unwrap_or(LevelFilter::INFO);

    let env = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| format!("{level_filter},tower_http=info,hyper=info,reqwest=info"));

    let logging_layers = vec![tracing_subscriber::fmt::layer()
        .with_filter(tracing_subscriber::EnvFilter::new(env))
        .boxed()];

    // Actually initialize the logger.
    tracing_subscriber::registry().with(logging_layers).init();

    debug!("logging set up");
}
