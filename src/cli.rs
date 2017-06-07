//! defines CLI for ff
use std;
use std::env;
use std::path::PathBuf;
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

fn action_init(dir_path: &str, config: Config) {
    // TODO:: make it separate function
    // TODO:: dir_path must be descendant of home dir
    // 
	// let home_dir = match std::env::home_dir() {
	// 	None => return Err("unknown home dir path".to_owned()),
	// 	Some(v) => v,
	// };
	// if !Path::new(&sync_dir).starts_with(&home_dir) {
	// 	println!("{:?}, {:?}", &sync_dir, &home_dir);
	// 	return Err("sync_dir should be descendant of home dir".to_owned());
	// }

    if let Err(e) = init(dir_path, config) {
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
                .about("Sets dir as dir being synchronized")
                .arg(Arg::with_name("dir-path")
                        .required(true)
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
                action_init(matches.value_of("dir-path").unwrap(), config);
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
