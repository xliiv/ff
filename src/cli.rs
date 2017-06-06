//! defines CLI for ff
use std;
use std::path::PathBuf;
use clap::*;

use config::*;
use core::*;

fn action_init(dir_path: &str) {
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

    if let Err(e) = init(
        dir_path,
        get_config_file_path().to_str().unwrap(),
    ) {
        println!("{}", e);
    }
}

fn action_add(file_paths: &Vec<&str>, space: &str) {
    add_files(
        file_paths,
        std::env::home_dir().unwrap().to_str().unwrap(),
        get_space_dir_path(&space).to_str().unwrap()
    );
}

fn action_remove(file_paths: &Vec<&str>) {
    remove_files(file_paths);
}

fn action_apply(space_dir: &str) {
    let mut sync_dir = PathBuf::from(get_sync_dir_path());
    if space_dir != "" {
        //TODO:: bug, unused var
        sync_dir = sync_dir.join(space_dir);
    }
    if let Err(e) = apply(
        &get_space_dir_path(space_dir).to_str().unwrap(),
        &get_space_dir_path(space_dir).to_str().unwrap(),
        std::env::home_dir().unwrap().to_str().unwrap(),
    ) {
        println!("{}", e);
    }
}

/// Defines and initialize command line dispatcher which run suitable actions
pub fn run_cli() {
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
                action_init(matches.value_of("dir-path").unwrap());
            }
        },
        Some("add") => {
            if let Some(ref matches) = matches.subcommand_matches("add") {
                let file_paths: Vec<_> = matches.values_of("file-path").unwrap().collect();
                let space_dir = matches.value_of("space-dir").unwrap_or("homedir");
                action_add(&file_paths, &space_dir);
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
                action_apply(space_dir);
            }
        },
        _ => {
            app.print_help().expect("Can't print help");
            ()
        },
    }
}
