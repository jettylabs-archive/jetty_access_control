//! Commands for Jetty CLI
//!

use std::path::PathBuf;

use clap::{self, Parser, Subcommand};

use jetty_core::logging::LevelFilter;

use crate::usage_stats::UsageEvent;

/// Jetty CLI: Open-source data access control for modern teams
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None, arg_required_else_help = true)]
pub(crate) struct JettyArgs {
    #[clap(subcommand)]
    pub(crate) command: JettyCommand,
    #[clap(global = true, short = 'v', long)]
    pub(crate) log_level: Option<LevelFilter>,
}

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum JettyCommand {
    /// Initialize a Jetty project.
    Init {
        /// Project name
        project_name: Option<String>,
        /// Initialize from an existing config (as a shortcut).
        #[clap(short, long, hide = true)]
        from: Option<PathBuf>,
        /// Overwrite project directory if it exists
        #[clap(short, long, value_parser, default_value = "false")]
        overwrite: bool,
    },
    Fetch {
        /// Visualize the graph in an SVG file.
        #[clap(long, value_parser, default_value = "false")]
        visualize: bool,
        /// Connectors to collect for.
        #[clap(short, long, use_value_delimiter = true, value_delimiter = ',')]
        connectors: Option<Vec<String>>,
    },
    Explore {
        #[clap(short, long, value_parser, default_value = "false")]
        fetch: bool,

        /// Select the ip and port to bind the server to (e.g. 127.0.0.1:3000)
        #[clap(short, long, value_parser)]
        bind: Option<String>,
    },
}
