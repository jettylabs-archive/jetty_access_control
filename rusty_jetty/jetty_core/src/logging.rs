//! Logging utilities for Jetty-wide output to stdout.
//!

use std::io::Write;

use env_logger::{fmt::Color, Builder, Env};
// Re-exports for convenience
pub use log::{debug, error, info, warn, LevelFilter};

/// Set up basic logging
pub fn setup(level: Option<LevelFilter>) {
    // The user can specify a log level via an env var
    // (such as for testing).
    let env = Env::default().filter_or("LOG_LEVEL", LevelFilter::Info.as_str());

    let mut log_builder = &mut Builder::from_env(env);
    // The input level overrides any env vars.
    if let Some(level) = level {
        log_builder = log_builder.filter_level(level);
    }
    log_builder
        .format(|buf, record| {
            let style = buf.style();
            let mut error_style = buf.style().clone();
            error_style
                .set_color(Color::Rgb(244, 113, 36))
                .set_bold(true);

            let timestamp = buf.timestamp();

            match record.level() {
                log::Level::Error => {
                    writeln!(
                        buf,
                        "Jetty ({}): {}",
                        timestamp,
                        error_style.value(record.args())
                    )
                }
                _ => writeln!(buf, "Jetty ({}): {}", timestamp, style.value(record.args())),
            }
        })
        .init();
    debug!("logging set up");
}
