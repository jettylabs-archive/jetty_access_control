//! Commands for Jetty CLI
//!

use std::path::PathBuf;

use clap::{self, Parser, Subcommand, ValueEnum};

use jetty_core::logging::LevelFilter;
use serde::{Deserialize, Serialize};

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
    /// Create a Jetty project in a new directory
    New {
        /// Project name
        project_name: Option<String>,
        /// Initialize from an existing config (as a shortcut).
        #[clap(short, long, hide = true)]
        from: Option<PathBuf>,
        /// Overwrite project directory if it exists
        #[clap(short, long, value_parser, default_value = "false")]
        overwrite: bool,
    },
    /// Fetch metadata using an existing Jetty project
    Fetch {
        /// Visualize the graph in an SVG file.
        #[clap(long, value_parser, default_value = "false")]
        visualize: bool,
        /// Connectors to collect for.
        #[clap(short, long, use_value_delimiter = true, value_delimiter = ',')]
        connectors: Option<Vec<String>>,
    },
    /// Launch the permissions exploration UI
    Explore {
        /// Fetch the current configuration before launching the UI
        #[clap(short, long, value_parser, default_value = "false")]
        fetch: bool,

        /// Select the ip and port to bind the server to (e.g. 127.0.0.1:3000)
        #[clap(short, long, value_parser)]
        bind: Option<String>,
    },
    /// Add connectors to an existing Jetty project
    Add,
    /// Build out initial configuration files for a project
    Bootstrap {
        /// Don't fetch the current configuration before generating the configuration files
        #[clap(short, long, value_parser, default_value = "false")]
        no_fetch: bool,
        /// Overwrite files if they exists
        #[clap(short, long, value_parser, default_value = "false")]
        overwrite: bool,
    },

    /// Diff the configuration with the current state of your infrastructure
    Diff {
        /// Fetch the current configurations before generating the diff
        #[clap(short, long, value_parser, default_value = "false")]
        fetch: bool,
    },

    /// Plan the changes needed to update the infra based on the diff
    Plan {
        /// Fetch the current configurations before generating the diff
        #[clap(short, long, value_parser, default_value = "false")]
        fetch: bool,
    },

    /// Apply the planned changes
    Apply {
        /// Don't fetch the current configurations before applying the changes
        #[clap(short, long, value_parser, default_value = "false")]
        no_fetch: bool,
    },

    /// Get the dot representation of a subgraph
    Subgraph {
        /// the node_id to start with. Get this from the url of the explore web UI
        #[clap(short, long, value_parser)]
        id: String,
        /// The depth of the subgraph to collect
        #[clap(short, long, value_parser, default_value = "1")]
        depth: usize,
    },
    /// Remove references to a group or user from the configuration
    Remove {
        /// the type of node that is being modified
        #[arg(value_enum)]
        node_type: RemoveOrModifyNodeType,
        /// the name of the user or group that is being removed
        name: String,
    },
    /// Rename a group or user in the configuration configuration
    Rename {
        /// the type of node that is being modified
        #[arg(value_enum)]
        node_type: RemoveOrModifyNodeType,
        /// the name of the user or group that will be updated
        old: String,
        /// the new name of that user or group
        new: String,
    },
}

/// The type of node being modified in the rename or remove commands
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum, Serialize, Deserialize)]
pub enum RemoveOrModifyNodeType {
    /// Modify a group
    Group,
    /// Modify a user
    User,
}
