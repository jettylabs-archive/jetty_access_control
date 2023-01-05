//! Watch for config changes and update schema if necessary

use std::{
    collections::{BTreeSet, HashMap},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{
    project,
    write::{
        self,
        groups::{self, GroupYaml},
        users::{self, UserYaml},
    },
    Jetty,
};

use anyhow::{bail, Result};
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
use tokio::sync::mpsc::{self, error::TrySendError, Sender};
use tracing::{error, info, warn};

/// Watch the user and group config files and update the yaml schema if there are changes
pub async fn watch_and_update(jetty: &Jetty) -> Result<()> {
    info!("starting file watcher");
    // Set up initial variables
    let group_config = Arc::new(Mutex::new(read_groups()));
    let paths = users::get_config_paths()?;
    let user_config = Arc::new(Mutex::new(users::parser::read_config_files(paths)?));

    update_schema(user_config.clone(), group_config.clone(), jetty)?;

    let (tx, mut rx) = mpsc::channel::<()>(1);

    tokio::spawn(watch_groups(
        project::groups_cfg_path_local(),
        group_config.clone(),
        tx.clone(),
    ));

    tokio::spawn(watch_users(
        project::users_cfg_root_path_local(),
        user_config.clone(),
        tx.clone(),
    ));

    drop(tx);

    while rx.recv().await.is_some() {
        info!("updating schema");
        update_schema(user_config.clone(), group_config.clone(), jetty)?;
        // This ensures that we don't end up writing the config more than once a second
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}

/// Update the yaml schema
fn update_schema(
    users: Arc<Mutex<HashMap<PathBuf, UserYaml>>>,
    groups: Arc<Mutex<BTreeSet<GroupYaml>>>,
    jetty: &Jetty,
) -> Result<()> {
    let user_config = users.lock().unwrap();
    let group_config = groups.lock().unwrap();

    let groups = groups::parser::get_all_group_names(&group_config);
    let users = users::parser::get_all_user_names(&user_config);

    let schema = write::config::generate_env_schema(&users, &groups, jetty)?;
    write::config::write_config_schema(&schema, ".")?;
    Ok(())
}

/// Watch the group config file for changes. Notify the calling function if changes are detected
async fn watch_groups<P: AsRef<Path>>(
    watch_path: P,
    group_config: Arc<Mutex<BTreeSet<GroupYaml>>>,
    update_channel: Sender<()>,
) -> Result<()> {
    let (tx, mut rx) = mpsc::channel(500);

    let mut debouncer = new_debouncer(
        Duration::from_millis(750),
        Some(Duration::from_millis(750)),
        move |res| {
            tx.blocking_send(res).unwrap();
        },
    )
    .unwrap();

    debouncer
        .watcher()
        .watch(watch_path.as_ref(), RecursiveMode::Recursive)?;

    while let Some(res) = rx.recv().await {
        match res {
            Ok(events) => {
                for event in events {
                    info!("change detected in {:?}", event.path);
                }
                *group_config.lock().unwrap() = read_groups();
                if let Err(TrySendError::Closed(_)) = update_channel.try_send(()) {
                    bail!("channel closed unexpectedly");
                }
            }
            Err(e) => warn!("watch error: {:?}", e),
        }
    }

    Ok(())
}

/// Watch the user config files for changes. Notify the calling function if changes are detected
async fn watch_users<P: AsRef<Path>>(
    watch_path: P,
    user_config: Arc<Mutex<HashMap<PathBuf, UserYaml>>>,
    update_channel: Sender<()>,
) -> Result<()> {
    let (tx, mut rx) = mpsc::channel(500);

    let mut debouncer = new_debouncer(
        Duration::from_millis(750),
        Some(Duration::from_millis(750)),
        move |res| {
            tx.blocking_send(res).unwrap();
        },
    )
    .unwrap();

    debouncer
        .watcher()
        .watch(watch_path.as_ref(), RecursiveMode::Recursive)?;

    while let Some(res) = rx.recv().await {
        match res {
            Ok(events) => {
                for event in events {
                    info!("change detected in {:?}", &event.path);
                    update_user_config(user_config.clone(), &event.path)
                }
                if let Err(TrySendError::Closed(_)) = update_channel.try_send(()) {
                    bail!("channel closed unexpectedly");
                }
            }
            Err(e) => warn!("watch error: {:?}", e),
        }
    }

    Ok(())
}

/// Update the user configuration if a file has changed
fn update_user_config(user_config: Arc<Mutex<HashMap<PathBuf, UserYaml>>>, path: &PathBuf) {
    // does the file exist?
    match path.exists() {
        true => {
            if path.is_file() {
                // read in the file, update the map
                if let Ok(config) = write::users::parser::read_config_file(path) {
                    user_config.lock().unwrap().insert(path.to_owned(), config);
                }
            }
        }
        false => {
            user_config.lock().unwrap().remove(path);
        }
    }
}

/// Read groups from the config file
fn read_groups() -> BTreeSet<GroupYaml> {
    groups::parser::read_config_file().unwrap_or_else(|e| {
        error!("unable to parse group config file: {e}");
        Default::default()
    })
}
