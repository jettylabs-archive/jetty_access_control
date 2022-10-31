use std::fmt::Debug;
use std::path::Path;

use anyhow::Result;
use jetty_core::logging::debug;
use tokio::fs::create_dir as create_fs_dir;
use tokio::fs::File;

pub(crate) async fn create_file<P: AsRef<Path> + Debug + Clone>(file_path: P) -> Result<File> {
    debug!("Creating {:?}", file_path);
    let res = File::create(file_path.clone())
        .await
        .map_err(anyhow::Error::from);
    if res.is_err() {
        debug!("Failed to create file {:?}. Continuing.", file_path);
    }
    res
}

/// Create a directory, ignoring if it already exists.
pub(crate) async fn create_dir_ignore_failure<P: AsRef<Path> + Debug + Clone>(dir_path: P) {
    if dir_path.as_ref().is_dir() {
        println!("Directory {dir_path:?} already exists. Continuing.");
    }
    let res = create_dir(&dir_path).await;
    match res {
        Ok(_) => (),
        Err(e) => debug!("Failed to create dir {:?}: {:?}", dir_path, e),
    }
}

pub(crate) async fn create_dir<P: AsRef<Path> + Debug + Clone>(dir_path: P) -> Result<()> {
    debug!("Creating dir {:?}", dir_path);
    create_fs_dir(dir_path.clone())
        .await
        .map_err(anyhow::Error::from)
}
