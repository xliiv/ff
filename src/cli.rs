//! defines CLI for ff
use std;
use std::env;
use std::path::{Path, PathBuf};
use clap::{SubCommand, Arg};

use config::*;
use core::*;

/// Returns path to config file
pub fn get_config_file_path() -> PathBuf {
	let mut config_dir = env::home_dir().unwrap();
	config_dir.push(".ff");
	config_dir.push("config.ini");
	return config_dir
}

fn home_dir_contained<T: AsRef<Path>>(dir: T) -> Result<bool, String> {
    let home_dir = std::env::home_dir()
        .ok_or("Can't find home dir")?;
    Ok(dir.as_ref().starts_with(&home_dir))
}

fn action_init(sync_dir: &str, config: Config) {
    let _sync_dir = match sync_dir.len() {
        0 => match std::env::current_dir() {
            Err(e) => {
                println!("Can't get home dir ({})", e);
                return
            },
            Ok(v) => v,
        },
        _ => Path::new(sync_dir).to_path_buf(),
    };
    let _sync_dir = match std::fs::canonicalize(&_sync_dir) {
        Err(e) => {
            println!("Can't canonicalize: {:?} ({})", &_sync_dir, e);
            return;
        },
        Ok(v) => v,
    };
    let is_home_dir = match home_dir_contained(&_sync_dir) {
        Err(e) => {
            println!("Can't validate if {:?} is descendat of home dir ({})", &_sync_dir, e);
            return 
        },
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
            return
        },
        Some(v) => v,
    };
    if let Err(e) = init(_sync_dir, config) {
        println!("{}", e);
    }
}

fn action_add(file_paths: &Vec<&str>, space_dir: &str, config: Config) {
    let sync_dir = match with_space_dir(space_dir, config) {
        Err(e) => {
            println!("{}", e);
            return
        }
        Ok(v) => v,
    };
    add_files(
        file_paths,
        std::env::home_dir().unwrap().to_str().unwrap(),
        &sync_dir.to_str().unwrap(),
    );
}

fn with_space_dir(space_dir: &str, config: Config) -> Result<PathBuf, String> {
    let sync_dir = config.get("sync-dir")
        .map_err(|e| format!("Can't read sync-dir from {} ({})", config.get_path(), e))?
        .ok_or(
            format!("Can't find 'sync-dir' value in config file: {}
                    Did you run: 'ff init' on your sync-dir?", config.get_path())
        )?;
    let mut sync_dir = PathBuf::from(sync_dir);
    if space_dir != "" {
        sync_dir = sync_dir.join(space_dir);
    }
    Ok(sync_dir)
}

fn action_remove(file_paths: &Vec<&str>) {
    remove_files(file_paths);
}

fn action_apply(space_dir: &str, config: Config) {
    let sync_dir = match with_space_dir(space_dir, config) {
        Err(e) => {
            println!("{}", e);
            return
        }
        Ok(v) => v,
    };
    if let Err(e) = apply(
        &sync_dir.to_str().unwrap(),
        &sync_dir.to_str().unwrap(),
        std::env::home_dir().unwrap().to_str().unwrap(),
    ) {
        println!("{}", e);
    }
}

/// Defines and initialize command line dispatcher which run suitable actions
pub fn run_cli() {
    let config = Config::new(get_config_file_path().to_str().unwrap()).unwrap();
    let mut app = app_from_crate!()
        .version(crate_version!())
        .about("\n\
            `ff` helps you manage dot files by:\n\n\
            - symlinking files from your homedir to synchronized dir\n\
            - symlinking files from synchronized dir to homedir\n\
        ")
        .subcommand(
            SubCommand::with_name("init")
                .about("Sets dir as sync-dir")
                .arg(Arg::with_name("dir-path")
                        .required(false)
                        .help("Sets dir as sync-dir (which means dot-files will be stored there and you sync that dir)")
                ),
        )
        .subcommand(
            SubCommand::with_name("add")
                .about("Adds files to synchronized dir")
                .args(&[
                      Arg::with_name("file-path")
                          .multiple(true).required(true),
                      Arg::with_name("space-dir")
                          .short("d")
                          .takes_value(true)
                          .help("Specifies dir in which file is added (default: 'homedir')")
                          .required(false),
                ])
        )
        .subcommand(
            SubCommand::with_name("remove")
                .about("Removes files from synchronized dir")
                .arg(Arg::with_name("file-path")
                        .multiple(true).required(true)
                )
        )
        .subcommand(
            SubCommand::with_name("apply")
                .about("Replaces home dir's files with aliases from synchronized dir")
                .arg(
                    Arg::with_name("space-dir")
                )
        );

    let matches = app.clone().get_matches();

    // apply matching
    match matches.subcommand_name() {
        Some("init") => {
            if let Some(ref matches) = matches.subcommand_matches("init") {
                action_init(matches.value_of("dir-path").unwrap_or(""), config);
            }
        },
        Some("add") => {
            if let Some(ref matches) = matches.subcommand_matches("add") {
                let file_paths: Vec<_> = matches.values_of("file-path").unwrap().collect();
                // TODO: space-dir should be optional
                let space_dir = matches.value_of("space-dir").unwrap_or("homedir");
                action_add(&file_paths, &space_dir, config);
            }
        },
        Some("remove") => {
            if let Some(ref matches) = matches.subcommand_matches("remove") {
                let file_paths: Vec<_> = matches.values_of("file-path").unwrap().collect();
                action_remove(&file_paths);
            }
        },
        Some("apply") => {
            if let Some(ref matches) = matches.subcommand_matches("apply") {
                //TODO:: default should be provided by clap
                let space_dir = matches.value_of("space_dir").unwrap_or("homedir");
                action_apply(space_dir, config);
            }
        },
        _ => {
            app.print_help().expect("Can't print help");
            ()
        },
    }
}
