//! Logging utilities for Jetty-wide output to stdout.
//!

// Re-exports for convenience
pub use tracing::metadata::LevelFilter;
pub use tracing::{debug, error, info, warn};
use tracing_subscriber::{filter, prelude::*};
use tracing_subscriber::{util::SubscriberInitExt, Layer};

/// Set up basic logging
pub fn setup(level: Option<LevelFilter>) {
    // The user can specify a log level via an env var
    // (such as for testing).
    let env = std::env::var("RUST_LOG").unwrap_or_else(|_| "info,tower_http=debug".into());
    let mut logging_layers = vec![tracing_subscriber::EnvFilter::new(env).boxed()];

    let layer = tracing_subscriber::fmt::layer()
        .with_filter(level.unwrap())
        .with_filter(filter::filter_fn(|metadata| {
            metadata.target().contains("jetty")
        }))
        .boxed();
    // The input level overrides any env vars.
    if let Some(level) = level {
        logging_layers.push(layer);
    } else {
        let layer = tracing_subscriber::fmt::layer()
            .with_filter(LevelFilter::INFO)
            .boxed();
        logging_layers.push(layer);
    }

    // Actually initialize all logging layers
    tracing_subscriber::registry()
        // .with(tracing_subscriber::fmt::layer())
        .with(logging_layers)
        .init();

    debug!("logging set up");
}
