#[macro_use]

extern crate clap;
extern crate tempdir;
extern crate ini;
extern crate walkdir;

use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::Path;
use std::path::PathBuf;

use clap::{App, Arg, SubCommand};
use ini::Ini;
use walkdir::WalkDir;

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs::File;
    use std::fs;
    use std::io::prelude::*;
    use std::io;
    use std::os::unix::fs as unix_fs;
    use std::path::PathBuf;
    use tempdir::TempDir;

    use super::*;

    #[test]
    fn init_saves_dir_to_init_in_config_file() {
        let homedir = TempDir::new("user1").unwrap();
        let config_dir = TempDir::new_in(homedir.path(), ".ff").unwrap();
        let config_file = config_dir.path().join("config.ini");
        let dir_to_init = TempDir::new_in(homedir.path(), "dot-files").unwrap();
        let ok_content = format!(
            "tracking-dir={}\n", dir_to_init.path().to_str().unwrap()
        );

        env::set_current_dir(homedir.path()).unwrap();
        init(
            dir_to_init.path().to_str().unwrap(),
            config_file.to_str().unwrap(),
            homedir.path().to_str().unwrap(),
        );

        let mut f = File::open(config_file).unwrap();
        let mut config_file_src = String::new();
        f.read_to_string(&mut config_file_src).unwrap();
        assert_eq!(config_file_src, ok_content);
    }

    #[test]
    fn add_file_works_ok() {
        let homedir = TempDir::new("user1").unwrap();
        let tracking_dir = TempDir::new_in(homedir.path(), "dot-files").unwrap();
        let file_to_track = homedir.path().join(".vimrc");
        File::create(&file_to_track).unwrap();
        let tracked_file = tracking_dir.path().join(".vimrc");

        assert_eq!(tracked_file.exists(), false);
        env::set_current_dir(homedir.path()).unwrap();
        add(
            ".vimrc",
            &homedir.path().to_str().unwrap(),
            &tracking_dir.path().to_str().unwrap(),
        );
        assert_eq!(tracked_file.exists(), true);
        assert_eq!(
            fs::symlink_metadata(".vimrc").unwrap().file_type().is_symlink(),
            true
        );
    }

    #[test]
    fn add_many_files_works_ok() {
        let homedir = TempDir::new("user1").unwrap();
        let tracking_dir = TempDir::new_in(homedir.path(), "dot-files").unwrap();
        let mut files_to_track = Vec::new();
        let mut tracked_files = Vec::new();
        for filename in vec![".vimrc", ".bashrc"] {
            let filepath = homedir.path().join(filename);
            File::create(&filepath).unwrap();
            files_to_track.push(filepath);
            tracked_files.push(tracking_dir.path().join(&filename).to_owned());
        }

        // checks file are not tracked
        for file in &tracked_files {
            assert_eq!(file.exists(), false);
        }
        env::set_current_dir(homedir.path()).unwrap();
        add_files(
            &files_to_track.iter().map(|f| f.to_str().unwrap()).collect::<Vec<_>>(),
            homedir.path().to_str().unwrap(),
            tracking_dir.path().to_str().unwrap()
        );
        // checks file are tracked
        for (idx, path) in tracked_files.iter().enumerate() {
            assert_eq!(path.exists(), true);
            assert_eq!(
                fs::symlink_metadata(
                    files_to_track[idx].to_str().unwrap()
                ).unwrap().file_type().is_symlink(),
                true
            );
        }
    }

    #[test]
    fn added_file_is_removed_ok() {
        let homedir = TempDir::new("user1").unwrap();
        let sync_dir = TempDir::new_in(homedir.path(), "dot-files").unwrap();
        let sync_file = sync_dir.path().join(".vimrc");
        File::create(&sync_file).unwrap();
        let home_file = homedir.path().join(".vimrc");
        unix_fs::symlink(&sync_file, &home_file).unwrap();

        assert_eq!(
            fs::symlink_metadata(&home_file).unwrap().file_type().is_symlink(),
            true
        );
        env::set_current_dir(homedir.path()).unwrap();
        remove(
            &home_file.to_str().unwrap(),
            &homedir.path().to_str().unwrap(),
            &sync_dir.path().to_str().unwrap(),
        );
        assert_eq!(
            fs::symlink_metadata(&home_file).unwrap().file_type().is_file(),
            true
        );
    }

    #[test]
    fn added_files_are_removed_correctly() {
        let homedir = TempDir::new("user1").unwrap();
        let tracking_dir = TempDir::new_in(homedir.path(), "dot-files").unwrap();
        let mut tracked_files = Vec::new();
        let mut files_to_restore = Vec::new();
        for filename in vec![".vimrc", ".bashrc"] {
            let tracked_file = tracking_dir.path().join(&filename);
            let file_to_restore = homedir.path().join(&filename);

            File::create(&tracked_file).unwrap();
            unix_fs::symlink(
                &tracked_file.to_str().unwrap(),
                &file_to_restore.to_str().unwrap()
            ).unwrap();

            tracked_files.push(tracked_file);
            files_to_restore.push(file_to_restore);
        }

        for file in &files_to_restore {
            // checks that files are symlinks
            assert_eq!(
                fs::symlink_metadata(&file).unwrap().file_type().is_symlink(),
                true
            );
        }
        env::set_current_dir(homedir.path()).unwrap();
        remove_files(
            &files_to_restore.iter().map(|f| f.to_str().unwrap()).collect::<Vec<_>>(),
            homedir.path().to_str().unwrap(),
            tracking_dir.path().to_str().unwrap(),
        );
        for file in files_to_restore {
            // checks that files are regular files
            assert_eq!(
                fs::symlink_metadata(&file).unwrap().file_type().is_file(),
                true
            );
        }

    }

    #[test]
    fn apply_works_for_single_file() {
        let homedir = TempDir::new("user1").unwrap();
        let tracking_dir = TempDir::new_in(homedir.path(), "dot-files").unwrap();
        let tracked_file = tracking_dir.path().join(".vimrc");
        File::create(&tracked_file).unwrap();
        let user_file = homedir.path().join(".vimrc");
        File::create(&user_file).unwrap();

        assert_eq!(
            fs::metadata(&user_file).unwrap().file_type().is_file(), true
        );
        apply(
            &tracking_dir.path().to_str().unwrap(),
            &tracking_dir.path().to_str().unwrap(),
            &homedir.path().to_str().unwrap(),
        );
        assert_eq!(
            fs::symlink_metadata(&user_file).unwrap().file_type().is_symlink(),
            true
        );
    }
}


// TODO::
// keep trakcing-dir as ~/<path-tracking-dir> in cnofig file
// what about adding dir?
// bash autocompletion: https://kbknapp.github.io/clap-rs/clap/struct.App.html#examples-35
// errors handling!!
// println should be in action_* instead of add(..) or remove(..)
// add note that it works only in homedir


pub fn swap_path_bases(file_path: &str, src_base: &str, dst_base: &str) -> String {
    let new = file_path.replace(src_base, dst_base);
    return String::from(new);
}

pub fn init(dir_to_init: &str, config_path: &str, base_dir: &str) {
    let path = Path::new(config_path);
    if path.exists() == true {
        let old_path = get_tracking_dir_path();
        println!("Overwritting old path which was: {}", old_path);
    }
    if path.parent().unwrap().exists() == false {
        std::fs::create_dir(path.parent().unwrap());
    }

    let mut abs_dir = std::env::current_dir().unwrap();
    let _dir_to_init;
    if std::path::Path::new(dir_to_init).is_absolute() == true {
        _dir_to_init = std::path::Path::new(&dir_to_init);
    } else {
        abs_dir = abs_dir.join(&dir_to_init);
        _dir_to_init = std::path::Path::new(abs_dir.to_str().unwrap());
    }
    let mut conf = Ini::new();
    conf.with_section(None::<String>).set("tracking-dir", _dir_to_init.to_str().unwrap());
    conf.write_to_file(config_path).unwrap();
    println!("Set tracking-dir to: {}", _dir_to_init.display());
}

pub fn add(file_path: &str, src_base: &str, dst_base: &str) {
    let mut abs_src = std::env::current_dir().unwrap();
    abs_src.push(file_path);

    let abs_dst = swap_path_bases(
        abs_src.to_str().unwrap(), src_base, dst_base,
    );

    fs::create_dir_all(
        Path::new(&abs_dst).parent().unwrap().to_str().unwrap()
    );
    fs::copy(&file_path, &abs_dst);
    std::fs::remove_file(&file_path);
    unix_fs::symlink(&abs_dst, &file_path).unwrap();
    println!("added: {} (to: {})", file_path, abs_dst);
}

pub fn add_files(file_paths: &Vec<&str>, base_dir: &str, tracking_dir: &str) {
    for file_path in file_paths {
        add(file_path, base_dir, tracking_dir);
    };
}

pub fn remove(to_remove: &str, src_base: &str, dst_base: &str) {
    let mut abs_to_remove = std::env::current_dir().unwrap();
    abs_to_remove.push(to_remove);
    let sync_file = swap_path_bases(
        &abs_to_remove.to_str().unwrap(), &src_base, &dst_base,
    );
    std::fs::remove_file(&to_remove);
    match fs::copy(&sync_file, &to_remove) {
        Ok(x) => println!("Unlinked file: {} (from: {})", to_remove, sync_file),
        Err(x) => println!("ERROR unlinking file: {}", to_remove),
    }
}

pub fn remove_files(file_paths: &Vec<&str>, base_dir: &str, tracking_dir: &str) {
    for file_path in file_paths {
        remove(file_path, base_dir, tracking_dir);
    }
}

pub fn apply(root_dir: &str, src_base: &str, dst_base: &str) {
    for entry in WalkDir::new(Path::new(&root_dir)) {
        let entry = entry.unwrap();

        if std::fs::metadata(entry.path()).unwrap().is_file() == true {
            let user_file = swap_path_bases(
                &entry.path().to_str().unwrap(),
                &src_base,
                &dst_base,
            );

            fs::create_dir_all(
                Path::new(&user_file).parent().unwrap().to_str().unwrap()
            );
            std::fs::remove_file(&user_file);
            unix_fs::symlink(&entry.path(), &user_file).unwrap();
            println!("symlinked: {} -> {}", user_file, entry.path().display());
        }

    }
}

fn get_config_file_path() -> PathBuf {
    let mut config_dir = std::env::home_dir().unwrap();
    config_dir.push(".ff");
    config_dir.push("config.ini");
    return config_dir
}

fn get_tracking_dir_path() -> String {
    let conf = Ini::load_from_file(
        get_config_file_path().to_str().unwrap(),
    ).unwrap();
    let section = conf.section(None::<String>).unwrap();
    return section.get("tracking-dir").unwrap().to_owned();
}

fn get_space_dir_path(space_name: &str) -> PathBuf {
    let mut dst_with_space = PathBuf::from(get_tracking_dir_path());
    if space_name != "" {
        dst_with_space.push(space_name);
    }
    return dst_with_space;
}

fn action_init(dir_path: &str) {
    init(
        dir_path,
        get_config_file_path().to_str().unwrap(),
        std::env::home_dir().unwrap().to_str().unwrap(),
    );
}

fn action_add(file_paths: &Vec<&str>, space: &str) {
    add_files(
        file_paths,
        std::env::home_dir().unwrap().to_str().unwrap(),
        get_space_dir_path(&space).to_str().unwrap()
    );
}

fn action_remove(file_paths: &Vec<&str>, space: &str) {
    remove_files(
        file_paths,
        std::env::home_dir().unwrap().to_str().unwrap(),
        get_space_dir_path(&space).to_str().unwrap(),
    );
}

fn action_apply(space_dir: &str) {
    let mut tracking_dir = PathBuf::from(get_tracking_dir_path());
    if space_dir != "" {
        // TODO:: validate space_dir existance?
        tracking_dir = tracking_dir.join(space_dir);
    }
    apply(
        &get_space_dir_path(space_dir).to_str().unwrap(),
        &get_space_dir_path(space_dir).to_str().unwrap(),
        std::env::home_dir().unwrap().to_str().unwrap(),
    );
}

fn main() {
    // matches defined
    let matches = App::new("ff")
        .version(crate_version!())
        .about("\n\
            `ff` helps you manage dot files by:\n\n\
            - linking files from your homedir to synchronized dir\n\
            - linking files from synchronized dir to homedir\n\
        ")
        //.author("xliiv (tymoteusz.jankowski@gmail.com)")
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
                .arg(Arg::with_name("file-path")
                        .multiple(true)
                        .required(true)
                ),
        )
        .subcommand(
            SubCommand::with_name("remove")
                .about("Removes files from synchronized dir")
                .arg(Arg::with_name("file-path")
                        .multiple(true)
                        .required(true)
                )
        )
        .subcommand(
            SubCommand::with_name("apply")
                .about("Replaces home dir's files with aliases from synchronized dir")
                .arg(
                    Arg::with_name("space-dir")
                )
        )
        .get_matches();

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
                let space_dir = matches.value_of("space_dir").unwrap_or("homedir");
                action_add(&file_paths, &space_dir);
            }
        },
        Some("remove") => {
            if let Some(ref matches) = matches.subcommand_matches("remove") {
                let file_paths: Vec<_> = matches.values_of("file-path").unwrap().collect();
                let space_dir = matches.value_of("space_dir").unwrap_or("homedir");
                action_remove(&file_paths, space_dir);
            }
        },
        Some("apply") => {
            if let Some(ref matches) = matches.subcommand_matches("apply") {
                let space_dir = matches.value_of("space_dir").unwrap_or("homedir");
                action_apply(space_dir);
            }
        },
        None => println!("No subcommand was given"),
        _ => println!("see help for available commands"),
    }
}
