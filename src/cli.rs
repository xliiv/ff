//! defines CLI for ff
use std::env;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std;

use fui::Fui;
use fui::feeders::DirItems;
use fui::fields::{Autocomplete, Multiselect};
use fui::form::FormView;
use fui::utils::cwd;
use fui::validators;

use config::*;
use core::*;

fn get_fui(config: Config) -> Fui<'static, 'static> {
    let config = Rc::new(config);
    let config_init = Rc::clone(&config);
    let config_add = Rc::clone(&config);
    let config_apply = Rc::clone(&config);
    Fui::new()
        .action(
            "init",
            "select dir where dot-files will be stored",
            FormView::new().field(
                Autocomplete::new("dir-path", DirItems::dirs())
                    .help("Path to dir where dot-files will be stored")
                    .initial(cwd())
                    .validator(validators::Required),
            ),
            move |v| {
                action_init(
                    v["dir-path"].as_str().expect("can't get dir-path"),
                    &config_init,
                );
            },
        )
        .action(
            "add",
            "adds home-dir files to sync-dir",
            FormView::new()
                .field(
                    Multiselect::new("file-path", DirItems::new())
                        .help("Path to file which should be tracked")
                        .validator(validators::Required)
                        .validator(validators::FileExists),
                )
                .field(
                    Autocomplete::new("sync-subdir", DirItems::dirs())
                        .help("Path to dir where tracked file are stored")
                        .initial("homedir")
                        .validator(validators::Required),
                ),
            move |v| {
                let file_paths = v.get("file-path")
                    .unwrap()
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| x.as_str().unwrap())
                    .collect::<Vec<&str>>();
                if let Err(e) =
                    action_add(&file_paths, v["sync-subdir"].as_str().unwrap(), &config_add)
                {
                    println!("{}", e);
                }
            },
        )
        .action(
            "remove",
            "removes home-dir files from sync-dir",
            FormView::new().field(
                Multiselect::new("file-path", DirItems::new())
                    .help("Path to home-dir file which should be removed from sync-dir")
                    .validator(validators::Required)
                    .validator(validators::FileExists),
            ),
            move |v| {
                let file_paths = v.get("file-path")
                    .unwrap()
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| x.as_str().unwrap())
                    .collect::<Vec<&str>>();
                action_remove(&file_paths)
            },
        )
        .action(
            "apply",
            "replaces home-dir's files with aliases from sync-dir",
            FormView::new().field(
                Autocomplete::new("sync-subdir", DirItems::dirs())
                    .help("Path to sync-subdir where tracked files are stored")
                    .initial("homedir")
                    .validator(validators::Required)
                    .validator(validators::DirExists),
            ),
            move |v| {
                let space_dir = v["sync-subdir"].as_str().unwrap();
                if let Err(e) = action_apply(space_dir, &config_apply) {
                    println!("{}", e);
                }
            },
        )
        .name(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .author(crate_authors!())
}

/// Returns path to config file
pub fn get_config_file_path() -> PathBuf {
    let mut conf_file = env::home_dir().expect("Can't find home dir");
    conf_file.push(".ff");
    conf_file.push("config.ini");
    conf_file
}

fn home_dir_contained<T: AsRef<Path>>(dir: T) -> Result<bool, String> {
    let home_dir = std::env::home_dir().ok_or("Can't find home dir")?;
    Ok(dir.as_ref().starts_with(&home_dir))
}

fn action_init(sync_dir: &str, config: &Config) {
    let _sync_dir = match sync_dir.len() {
        0 => match std::env::current_dir() {
            Err(e) => {
                println!("Can't get home dir ({})", e);
                return;
            }
            Ok(v) => v,
        },
        _ => Path::new(sync_dir).to_path_buf(),
    };
    let _sync_dir = match std::fs::canonicalize(&_sync_dir) {
        Err(e) => {
            println!("Can't canonicalize: {:?} ({})", &_sync_dir, e);
            return;
        }
        Ok(v) => v,
    };
    let is_home_dir = match home_dir_contained(&_sync_dir) {
        Err(e) => {
            println!(
                "Can't validate if {:?} is descendat of home dir ({})",
                &_sync_dir, e
            );
            return;
        }
        Ok(v) => v,
    };
    if !is_home_dir {
        println!("Sync dir should be descendant of home dir");
        return;
    }

    // TODO1: tmp conversion until code after action_ is moved to PathBuf
    let _sync_dir = match _sync_dir.as_path().to_str() {
        None => {
            println!("Can't convert {:?} to str", _sync_dir);
            return;
        }
        Some(v) => v,
    };
    if let Err(e) = init(_sync_dir, config) {
        println!("{}", e);
    }
}

fn action_add(file_paths: &[&str], space_dir: &str, config: &Config) -> Result<(), String> {
    let home_dir = env::home_dir().ok_or("Can't get home dir")?;
    let home_dir = home_dir.to_str().ok_or("Can't convert home dir to str")?;
    let sync_dir = with_space_dir(space_dir, config)?;
    let sync_dir = sync_dir.to_str().ok_or("Can't convert home dir to str")?;
    add_files(file_paths, home_dir, sync_dir);
    Ok(())
}

fn with_space_dir(space_dir: &str, config: &Config) -> Result<PathBuf, String> {
    let sync_dir = config
        .get("sync-dir")
        .map_err(|e| format!("Can't read sync-dir from {} ({})", config.get_path(), e))?
        .ok_or_else(|| {
            format!(
                "Can't find 'sync-dir' value in config file: {}\n\
                 Did you run: 'ff init' on your sync-dir?",
                config.get_path()
            )
        })?;
    let mut sync_dir = PathBuf::from(sync_dir);
    if space_dir != "" {
        sync_dir = sync_dir.join(space_dir);
    }
    Ok(sync_dir)
}

fn action_remove(file_paths: &[&str]) {
    remove_files(file_paths);
}

fn action_apply(space_dir: &str, config: &Config) -> Result<(), String> {
    let sync_dir = with_space_dir(space_dir, config)?;
    let sync_dir = sync_dir
        .to_str()
        .ok_or("Can't convert sync-dir with space to str")?;
    let home_dir = std::env::home_dir().ok_or("Can't get home dir")?;
    let home_dir = home_dir
        .to_str()
        .ok_or_else(|| "Can't convert home dir to str".to_owned())?;
    let to_ignore = config.get("ignore-when-apply").map_err(|e| {
        format!(
            "Can't get ignore-when-apply from {} ({})",
            config.get_path(),
            e
        )
    })?;
    let to_ignore = match to_ignore.as_ref() {
        None => vec![],
        Some(v) => v.split(',').collect(),
    };
    apply(sync_dir, sync_dir, home_dir, &to_ignore)
}

/// Defines and initialize command line dispatcher which run suitable actions
pub fn run_cli() {
    let conf_path = get_config_file_path();
    let conf_path = match conf_path.to_str() {
        None => {
            println!("Can't convert config path");
            return;
        }
        Some(v) => v,
    };
    let config = match Config::new(conf_path) {
        Err(e) => {
            println!("Can't initialize config file {}: ({})", conf_path, e);
            return;
        }
        Ok(v) => v,
    };

    get_fui(config).run();
}
