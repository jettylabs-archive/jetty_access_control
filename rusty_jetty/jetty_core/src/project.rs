//! Path utilities for project organization.
//!
//! The project structure currently looks like this:
//!
//! ```text
//! pwd
//!  └── {project_name}
//!       ├── jetty_config.yaml
//!       ├── .data
//!       │    ├── jetty_graph
//!       │    └── {connector}
//!       │         └── {connector-specific data}
//!       ├── .vscode
//!       │    └── settings.json
//!       ├── .schemas
//!       │    ├── assets.json
//!       │    ├── groups.json
//!       │    ├── users.json
//!       │    └── config.json
//!       ├── .data
//!       │    ├── jetty_graph
//!       │    └── {connector}
//!       │         └── {connector-specific data}
//!       ├── groups
//!       │    └── groups.yaml
//!       ├── tags
//!       │    └── tags.yaml
//!       ├── users
//!       │    └── <name>.yaml
//!       └── assets
//!            └── <structure mirroring your infrastructure>
//! ```

use std::path::{Path, PathBuf};

use dirs::home_dir;
use lazy_static::lazy_static;

lazy_static! {
    static ref TAGS_DIR: PathBuf = PathBuf::from("tags");
    static ref DATA_DIR: PathBuf = PathBuf::from(".data");
    static ref TAGS_CFG: PathBuf = PathBuf::from("tags.yaml");
    static ref GROUPS_DIR: PathBuf = PathBuf::from("groups");
    static ref GROUPS_CFG: PathBuf = PathBuf::from("groups.yaml");
    static ref ASSETS_DIR: PathBuf = PathBuf::from("assets");
    static ref ASSETS_CFG: PathBuf = PathBuf::from("index.yaml");
    static ref USERS_DIR: PathBuf = PathBuf::from("users");
    static ref JETTY_CFG: PathBuf = PathBuf::from("jetty_config.yaml");
    static ref CONNECTOR_CFG: PathBuf = PathBuf::from("connectors.yaml");
    static ref PROFILE_CFG_DIR: PathBuf = PathBuf::from(".jetty");
    static ref JETTY_GRAPH: PathBuf = PathBuf::from("jetty_graph");
    static ref DEFAULT_KEY_DIR: PathBuf = PathBuf::from(".ssh");
    pub(crate) static ref DEFAULT_SCHEMA_DIR: PathBuf = PathBuf::from(".schema");
    pub(crate) static ref VSCODE_SETTINGS_PATH: PathBuf = PathBuf::from(".vscode/settings.json");
}

/// The path to tag configuration files
pub fn tags_cfg_path<P: AsRef<Path>>(project_path: P) -> PathBuf {
    project_path.as_ref().join(tags_cfg_path_local())
}

/// Local path for the tags config.
pub fn tags_cfg_path_local() -> PathBuf {
    TAGS_DIR.as_path().join(TAGS_CFG.as_path())
}

/// The path to group configuration files
pub fn groups_cfg_path<P: AsRef<Path>>(project_path: P) -> PathBuf {
    project_path.as_ref().join(groups_cfg_path_local())
}

/// Local path for the groups config.
pub fn groups_cfg_path_local() -> PathBuf {
    GROUPS_DIR.as_path().join(GROUPS_CFG.as_path())
}

/// Local path for the users config.
pub fn users_cfg_root_path_local() -> PathBuf {
    USERS_DIR.to_owned()
}

/// The path to assets configuration files
pub fn assets_cfg_root_path_local() -> PathBuf {
    ASSETS_DIR.to_owned()
}

/// The filename to use for the assets configuration files
pub fn assets_cfg_filename() -> PathBuf {
    ASSETS_CFG.to_owned()
}

/// Path for the connector config.
pub fn connector_cfg_path() -> PathBuf {
    home_dir()
        .expect("getting home dir")
        .join(PROFILE_CFG_DIR.as_path())
        .join(CONNECTOR_CFG.as_path())
}

/// Path for the jetty config.
pub fn jetty_cfg_path<P: AsRef<Path>>(project_path: P) -> PathBuf {
    project_path.as_ref().join(JETTY_CFG.as_path())
}

/// Local path for the jetty config.
pub fn jetty_cfg_path_local() -> PathBuf {
    JETTY_CFG.clone()
}

/// Path for the data directory
pub fn data_dir() -> PathBuf {
    DATA_DIR.clone()
}

/// Filename for the serialized access graph
pub fn graph_filename() -> PathBuf {
    JETTY_GRAPH.clone()
}

/// Path for the default key directory
pub fn default_keypair_dir_path() -> PathBuf {
    home_dir()
        .expect("getting home dir")
        .join(DEFAULT_KEY_DIR.as_path())
}

/// Path to the user_id file
pub fn user_id_file() -> PathBuf {
    home_dir()
        .expect("getting home dir")
        .join(PROFILE_CFG_DIR.as_path())
        .join("uid")
}
