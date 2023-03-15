//!
//! Access to Jetty
//!
//! Provides all utilities for accessing Jetty connectors and the Jetty Access
//! Graph.
#![deny(missing_docs)]

pub use connectors::Connector;
pub use jetty::fetch_credentials;
pub use jetty::Jetty;

pub mod access_graph;
pub mod connectors;
pub mod cual;
pub mod jetty;
pub mod logging;
pub mod permissions;
pub mod project;
pub mod write;

#[macro_export]
/// Time the code inside the macro. Write the elapsed time to debug logs.
/// Derived from https://notes.iveselov.info/programming/time_it-a-case-study-in-rust-macros
macro_rules! log_runtime {
    ($context:literal, $($tt:tt)+) => {
        {
            debug!("{}: starting", $context);
            let timer = std::time::Instant::now();
            let x =
            $(
                $tt
            )+;
            debug!("{}: {:?}", $context, timer.elapsed());
            x
        }
    }
}

#[allow(unused_macros)]
#[macro_export]
/// Time the code inside the macro. Write the elapsed time to std out.
/// Derived from https://notes.iveselov.info/programming/time_it-a-case-study-in-rust-macros
macro_rules! print_runtime {
    ($context:literal, $($tt:tt)+) => {
        {
            println!("{}: starting", $context);
            let timer = std::time::Instant::now();
            let x =
            $(
                $tt
            )+;
            println!("{}: {:?}", $context, timer.elapsed());
            x
        }
    }
}
