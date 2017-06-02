//! delivers functionlity of ff
use std;
use std::*;
use std::path::PathBuf;
use std::os::unix::fs as unix_fs;
use std::path::Path;

use ini::Ini;
use walkdir::WalkDir;

use config::*;


/// Replaces `old_value` with `new_value` in `text`
///
/// ```
/// let text = "/home/joe/.bashrc";
/// let old_value = "/home/joe";
/// let new_value = "/home/joe/dot-files";
/// let result = swap_path_bases(&text, &old_value, &new_value)
/// assert_eq!(result, "/home/joe/dot-files/.bashrc");
/// ```
///
fn swap_path_bases(text: &str, old_value: &str, new_value: &str) -> String {
    let new = text.replace(old_value, new_value);
    return String::from(new);
}


/// Returns joined `space_name` with absolute `sync-dir` path
pub fn get_space_dir_path(space_name: &str) -> PathBuf {
    let mut dst_with_space = PathBuf::from(get_sync_dir_path());
    if space_name != "" {
        dst_with_space.push(space_name);
    }
    return dst_with_space;
}


/// Saves `sync_dir` in config at `config_path` for further use
pub fn init(sync_dir: &str, config_path: &str) {
    let path = Path::new(config_path);
    if path.exists() == true {
        let old_path = get_sync_dir_path();
        println!("Overwritting old path which was: {}", old_path);
    }
    if path.parent().unwrap().exists() == false {
        std::fs::create_dir(path.parent().unwrap()).unwrap();
    }

    let mut abs_dir = std::env::current_dir().unwrap();
    let _dir_to_init;
    if std::path::Path::new(sync_dir).is_absolute() == true {
        _dir_to_init = std::path::Path::new(&sync_dir);
    } else {
        abs_dir = abs_dir.join(&sync_dir);
        _dir_to_init = std::path::Path::new(abs_dir.to_str().unwrap());
    }
    // TODO:: this ini should be arg as `config` or something
    let mut conf = Ini::new();
    conf.with_section(None::<String>).set("tracking-dir", _dir_to_init.to_str().unwrap());
    conf.write_to_file(config_path).unwrap();
    println!("Set tracking-dir to: {}", _dir_to_init.display());
}

/// Adds `file_path` to `sync-dir`
///
/// New path is calculated by replacing `home_dir` with `sync_dir` in `file_path`
pub fn add(file_path: &str, home_dir: &str, sync_dir: &str) {
    let mut abs_src = std::env::current_dir().unwrap();
    abs_src.push(file_path);

    let abs_dst = swap_path_bases(
        abs_src.to_str().unwrap(), home_dir, sync_dir,
    );

    fs::create_dir_all(
        Path::new(&abs_dst).parent().unwrap().to_str().unwrap()
    ).unwrap();
    fs::copy(&file_path, &abs_dst).unwrap();
    std::fs::remove_file(&file_path).unwrap();
    unix_fs::symlink(&abs_dst, &file_path).unwrap();
    println!("added: {} (to: {})", file_path, abs_dst);
}

/// Adds all files contained in `file_paths` to `sync-dir`
/// (see: `ff::core::add` for details)
pub fn add_files(file_paths: &Vec<&str>, home_dir: &str, sync_dir: &str) {
    for file_path in file_paths {
        add(file_path, home_dir, sync_dir);
    };
}

/// Removes `symlink` and replace it with its target
pub fn remove(symlink: &str) {
    let linked_file;
    match fs::read_link(symlink) {
        Ok(path) => linked_file = path,
        Err(err) => {
            println!("File '{}' NOT unlinked ({:?})", symlink, err);
            return
        }
    }
    std::fs::remove_file(&symlink).unwrap();
    match fs::copy(&linked_file, &symlink) {
        Ok(_x) => println!(
            "Unlinked file: {} (from: {:?})", symlink, linked_file
        ),
        Err(_x) => println!("ERROR unlinking file: {}", symlink),
    }
    std::fs::remove_file(&linked_file).unwrap();
}

/// Removes all files contained in `file_paths` from `sync-dir`
/// (see: `ff::core::remove` for details)
pub fn remove_files(file_paths: &Vec<&str>) {
    for file_path in file_paths {
        remove(file_path);
    }
}

/// Uses all files contained in `to_walk` to be target of symlinks defined by
/// replacing `sync_dir` in `home_dir`, for example:
/// 
/// Assuming that there is a `.bashrc` file in `/home/joe/sync-dir` dir
///
/// `apply("/home/joe/sync-dir", "/home/joe/sync-dir", "/home/joe")`
///
/// will result as replacing:
///
/// `/home/joe/.bashrc`
///
/// with symlink to:
///
/// `/home/joe/sync-dir/.bashrc`
///
pub fn apply(to_walk: &str, sync_dir: &str, home_dir: &str) {
    for content_item in WalkDir::new(Path::new(&to_walk)) {
        let content_item = content_item.unwrap();

        if std::fs::metadata(content_item.path()).unwrap().is_file() == true {
            println!("CI: {:?}", content_item);
            let user_file = swap_path_bases(
                &content_item.path().to_str().unwrap(),
                &sync_dir,
                &home_dir,
            );

            fs::create_dir_all(
                Path::new(&user_file).parent().unwrap().to_str().unwrap()
            ).unwrap();
            std::fs::remove_file(&user_file).unwrap();
            unix_fs::symlink(&content_item.path(), &user_file).unwrap();
            println!("symlinked: {} -> {}", user_file, content_item.path().display());
        }

    }
}


#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::fs;
    use std::io::prelude::*;
    use std::path::Path;
    use std::os::unix::fs as unix_fs;
    use tempdir::TempDir;

    use super::*;

    #[test]
    fn init_saves_dir_to_init_in_config_file() {
        let homedir = TempDir::new("user1").unwrap();
        let config_dir = TempDir::new_in(homedir.path(), ".ff").unwrap();
        let config_file = config_dir.path().join("config.ini");
        let sync_dir = TempDir::new_in(homedir.path(), "dot-files").unwrap();
        let ok_content = format!(
            "tracking-dir={}\n", sync_dir.path().to_str().unwrap()
        );

        init(
            sync_dir.path().to_str().unwrap(),
            config_file.to_str().unwrap(),
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
        add(
            file_to_track.to_str().unwrap(),
            &homedir.path().to_str().unwrap(),
            &tracking_dir.path().to_str().unwrap(),
        );
        assert_eq!(tracked_file.exists(), true);
        assert_eq!(
            fs::symlink_metadata(
                file_to_track.to_str().unwrap()
            ).unwrap().file_type().is_symlink(),
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
        remove(&home_file.to_str().unwrap());
        assert_eq!(
            fs::symlink_metadata(&home_file).unwrap().file_type().is_file(),
            true
        );
        assert_eq!(
            Path::new(&sync_file).exists(), false
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
        remove_files(
            &files_to_restore.iter().map(|f| f.to_str().unwrap()).collect::<Vec<_>>(),
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