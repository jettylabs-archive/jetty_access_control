//! Logging utilities for Jetty-wide output to stdout.
//!

// Re-exports for convenience
pub use tracing::metadata::LevelFilter;
pub use tracing::{debug, error, info, warn};
use tracing_subscriber::prelude::*;
use tracing_subscriber::{reload, util::SubscriberInitExt};

/// Set up basic logging.
///
/// The caller can specify a log level via `level`. If they don't, we
/// default to "info."
///
/// The `level` arg is overridden by any env var levels.
///
/// The user can specify a log level via the env var `RUST_LOG` (such as for testing).
/// If they don't, then we default to the level_filter defined above.
pub fn setup(
    level: Option<LevelFilter>,
) -> reload::Handle<tracing_subscriber::EnvFilter, tracing_subscriber::Registry> {
    let level_filter = level.unwrap_or(LevelFilter::INFO);

    let env = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| format!("{level_filter},tower_http=info,hyper=info,reqwest=info"));

    let (filter, reload_handle) = reload::Layer::new(tracing_subscriber::EnvFilter::new(env));
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    debug!("logging set up");
    reload_handle
}

/// Update global logging level.
///
/// The caller can specify a log level via `level`. If they don't, we
/// default to "info."
///
/// The `level` arg is overridden by any env var levels.
///
/// The user can specify a log level via the env var `RUST_LOG` (such as for testing).
/// If they don't, then we default to the level_filter defined above.
pub fn update_filter_level(
    reload_handle: reload::Handle<tracing_subscriber::EnvFilter, tracing_subscriber::Registry>,
    level: Option<LevelFilter>,
) {
    let level_filter = level.unwrap_or(LevelFilter::INFO);

    let env = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| format!("{level_filter},tower_http=info,hyper=info,reqwest=info"));

    let res = reload_handle.modify(|filter| *filter = tracing_subscriber::EnvFilter::new(&env));

    match res {
        Ok(_) => debug!("logging filter set to: {}", &env),
        Err(e) => error!("failed to update logging level: {}", e),
    }
}
