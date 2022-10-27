use std::path::PathBuf;

use dirs::home_dir;
use lazy_static::lazy_static;

lazy_static! {
    static ref PROJECT_DIR: PathBuf = PathBuf::from("src");
    static ref TAGS_CFG: PathBuf = PathBuf::from("tags.yaml");
    static ref JETTY_CFG: PathBuf = PathBuf::from("jetty_config.yaml");
    static ref CONNECTOR_CFG: PathBuf = PathBuf::from("connectors.yaml");
    static ref CONNECTOR_CFG_DIR: PathBuf = PathBuf::from(".jetty");
}

pub(crate) fn tags_cfg_path() -> PathBuf {
    PROJECT_DIR.join(TAGS_CFG.as_path())
}

pub(crate) fn connector_cfg_path() -> PathBuf {
    home_dir()
        .expect("getting home dir")
        .join(CONNECTOR_CFG_DIR.as_path())
        .join(CONNECTOR_CFG.as_path())
}

pub(crate) fn jetty_cfg_path() -> PathBuf {
    JETTY_CFG.clone()
}
