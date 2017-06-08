//! defines CLI for ff
use std;
use std::env;
use std::path::{Path, PathBuf};
use clap::{SubCommand, Arg};

use config::*;
use core::*;

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
        0 => {
            match std::env::current_dir() {
                Err(e) => {
                    println!("Can't get home dir ({})", e);
                    return;
                }
                Ok(v) => v,
            }
        }
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
            println!("Can't validate if {:?} is descendat of home dir ({})",
                     &_sync_dir,
                     e);
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
                        format!("Can't find 'sync-dir' value in config file: {}\n\
					Did you run: 'ff init' on your sync-dir?",
                                config.get_path())
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
    apply(sync_dir, sync_dir, home_dir)
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
						.help("Sets dir as sync-dir \
						(which means dot-files will be stored there and you sync that dir)")
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
            if let Some(matches) = matches.subcommand_matches("init") {
                action_init(matches.value_of("dir-path").unwrap_or(""), &config);
            }
        }
        Some("add") => {
            if let Some(matches) = matches.subcommand_matches("add") {
                let file_paths: Vec<_> = matches
                    .values_of("file-path")
                    .expect("Can't get file paths")
                    .collect();
                let space_dir = matches.value_of("space-dir").unwrap_or("");
                if let Err(e) = action_add(&file_paths, space_dir, &config) {
                    println!("{}", e);
                }
            }
        }
        Some("remove") => {
            if let Some(matches) = matches.subcommand_matches("remove") {
                let file_paths: Vec<_> = matches
                    .values_of("file-path")
                    .expect("Can't read file paths to remove")
                    .collect();
                action_remove(&file_paths);
            }
        }
        Some("apply") => {
            if let Some(matches) = matches.subcommand_matches("apply") {
                let space_dir = matches.value_of("space-dir").unwrap_or("");
                if let Err(e) = action_apply(space_dir, &config) {
                    println!("{}", e);
                }
            }
        }
        _ => {
            app.print_help().expect("Can't print help");
            ()
        }
    }
}
