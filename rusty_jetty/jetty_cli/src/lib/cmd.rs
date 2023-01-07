//! Commands for Jetty CLI
//!

use std::path::PathBuf;

use clap::{self, Parser, Subcommand, ValueEnum};

use jetty_core::logging::LevelFilter;
use serde::{Deserialize, Serialize};

/// Jetty CLI: Open-source data access control for modern teams
#[derive(Parser, Debug)]
#[clap(author, about, long_about = None, arg_required_else_help = true)]
pub(crate) struct JettyArgs {
    #[clap(subcommand)]
    pub(crate) command: JettyCommand,
    /// Specify the log level. Can be debug, info, warn, or error.
    #[clap(global = true, short = 'l', long)]
    pub(crate) log_level: Option<LevelFilter>,
}

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum JettyCommand {
    /// Launch a guided flow to create a Jetty project in a new directory
    New {
        /// Project name
        project_name: Option<String>,
        /// Initialize from an existing jetty_config.yaml file. This will create a new directory based on the name specified
        /// in the config file. For this to work properly, you must also have an appropriate ~/.jetty/connectors.yaml file in place.
        #[clap(short, long)]
        from: Option<PathBuf>,
        /// Overwrite project directory if it exists
        #[clap(short, long, value_parser, default_value = "false")]
        overwrite: bool,
    },
    /// Add connectors to an existing Jetty project via a similar flow to `jetty new`
    Add,
    /// Fetch and build out initial configuration files for a project
    Bootstrap {
        /// Don't fetch the current configuration before generating the configuration files
        #[clap(short, long, value_parser, default_value = "false")]
        no_fetch: bool,
        /// Overwrite files if they exists
        #[clap(short, long, value_parser, default_value = "false")]
        overwrite: bool,
    },
    /// Fetch metadata using an existing Jetty project
    Fetch {
        /// Visualize the graph in an SVG file
        #[clap(long, value_parser, default_value = "false", hide = true)]
        visualize: bool,
        /// Connectors to collect for
        #[clap(short, long, use_value_delimiter = true, value_delimiter = ',')]
        connectors: Option<Vec<String>>,
    },
    /// Watch config files for changes and update the YAML schema as needed to keep validation working properly. It's recommended that you run this while editing configuration files
    Dev,
    /// Rename a group or user across all the configuration files
    Rename {
        /// the type of node that is being modified
        #[arg(value_enum)]
        node_type: RemoveOrModifyNodeType,
        /// the name of the user or group that will be updated
        old: String,
        /// the new name of that user or group
        new: String,
    },
    /// Remove references to a group or user across all the configuration files
    Remove {
        /// the type of node that is being modified
        #[arg(value_enum)]
        node_type: RemoveOrModifyNodeType,
        /// the name of the user or group that is being removed
        name: String,
    },
    /// Diff the configuration and the current state of your environment
    Diff {
        /// Fetch the current configurations before generating the diff
        #[clap(short, long, value_parser, default_value = "false")]
        fetch: bool,
    },
    /// Plan the changes needed to update the environment based on the diff
    Plan {
        /// Fetch the current configurations before generating the diff
        #[clap(short, long, value_parser, default_value = "false")]
        fetch: bool,
    },
    /// Update the environment with the planned changes
    Apply {
        /// Don't fetch the current configurations before applying the changes
        #[clap(short, long, value_parser, default_value = "false")]
        no_fetch: bool,
    },
    /// Launch the exploration web UI
    Explore {
        /// Fetch the current configuration before launching the UI
        #[clap(short, long, value_parser, default_value = "false")]
        fetch: bool,

        /// Select the ip and port to bind the server to (e.g. 127.0.0.1:3000)
        #[clap(short, long, value_parser)]
        bind: Option<String>,
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
}

/// The type of node being modified in the rename or remove commands
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum, Serialize, Deserialize)]
pub enum RemoveOrModifyNodeType {
    /// Modify a group
    Group,
    /// Modify a user
    User,
}
