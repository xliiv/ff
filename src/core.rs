//! delivers functionlity of ff
use std;
use std::*;
use std::path::Path;
use std::path::PathBuf;
use std::os::unix::fs as unix_fs;
use std::result::Result;

use walkdir::{DirEntry, WalkDir};

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
    // notes:
    // asRef fix it? https://doc.rust-lang.org/std/fs/struct.File.html#method.open
    // add() uses text as std::path::Path
    // apply() uses text as std::path::Path
    let new = text.replace(old_value, new_value);
    String::from(new)
}

/// Saves defaults (like `sync_dir`, etc.) in config `config` for further use
pub fn init(sync_dir: &str, config: &Config) -> Result<(), String> {
    let abs_sync_dir = if std::path::Path::new(&sync_dir).is_absolute() {
        PathBuf::from(sync_dir)
    } else {
        let mut abs_dst = std::env::current_dir().map_err(|e| e.to_string())?;
        abs_dst.push(sync_dir);
        abs_dst
    };

    // next two lines can't be merged because of borrow error which i can't resolvec yet
    let abs_sync_dir = fs::canonicalize(&abs_sync_dir)
        .map_err(|e| format!("Can't canonicalize: {:?} ({})", &abs_sync_dir, e))?;
    let abs_sync_dir = abs_sync_dir
        .as_path()
        .to_str()
        .ok_or_else(|| format!("Can't convert to str: {:?}", &abs_sync_dir))?;

    let old_path = config.get("sync-dir")?;
    config.set("ignore-when-apply", ".git/,.hg/")?;
    config.set("sync-dir", abs_sync_dir)?;
    if let Some(p) = old_path {
        if p != abs_sync_dir {
            println!("Sync-dir overwritten (old value was: {:?})", p);
        };
    };
    println!("Set sync-dir to: {:?}", abs_sync_dir);
    Ok(())
}

/// Adds `file_path` to `sync-dir`
///
/// New path is calculated by replacing `home_dir` with `sync_dir` in `file_path`
pub fn add(file_path: &str, home_dir: &str, sync_dir: &str) -> Result<(), String> {
    let mut abs_dst = std::env::current_dir().map_err(|e| e.to_string())?;
    abs_dst.push(file_path);
    let abs_dst = match abs_dst.as_path().to_str() {
        None => return Err(format!("Can't get absolute dir for: {:?}", abs_dst)),
        Some(v) => swap_path_bases(v, home_dir, sync_dir),
    };

    let abs_dst_parent = Path::new(&abs_dst)
        .parent()
        .and_then(|p| p.to_str())
        .ok_or_else(|| format!("Can't get parent of: {:?}", &abs_dst))?;

    if let Err(e) = fs::create_dir_all(abs_dst_parent) {
        return Err(format!("Can't create sync-dir: {}", e.to_string()));
    }

    if let Err(e) = std::fs::rename(&file_path, &abs_dst) {
        return Err(format!("Can't move file to sync-dir ({})", e));
    }

    if let Err(e) = unix_fs::symlink(&abs_dst, &file_path) {
        println!(
            "Can't symlink moved file {} to {} ({})",
            abs_dst, file_path, e
        );
        println!("Trying revert file move..");
        if let Err(e) = std::fs::rename(&abs_dst, &file_path) {
            return Err(format!(
                "Can't revert file move ({}) - clean it MANUALLY ",
                e
            ));
        } else {
            return Err("File move reverted".to_owned());
        }
    }
    println!("added: {} (to: {})", file_path, abs_dst);
    Ok(())
}

/// Adds all files contained in `file_paths` to `sync-dir`
/// (see: `ff::core::add` for details)
pub fn add_files(file_paths: &[&str], home_dir: &str, sync_dir: &str) {
    for file_path in file_paths {
        if let Err(e) = add(file_path, home_dir, sync_dir) {
            println!("{}", e);
        }
    }
}

/// Removes `symlink` and replace it with its target
pub fn remove(symlinked: &str) -> Result<(), String> {
    let regular_file =
        fs::read_link(&symlinked).map_err(|e| format!("Can't read symlink: ({})", e))?;
    if let Err(e) = fs::rename(&regular_file, &symlinked) {
        return Err(format!(
            "Can't move file {:?} to {:?} ({})",
            &regular_file, &symlinked, e
        ));
    }
    println!("removed: {} (from: {:?})", symlinked, regular_file);
    Ok(())
}

/// Removes all files contained in `file_paths` from `sync-dir`
/// (see: `ff::core::remove` for details)
pub fn remove_files(file_paths: &[&str]) {
    for file_path in file_paths {
        if let Err(e) = remove(file_path) {
            println!("{}", e);
        }
    }
}

/// Calls `symlink_file` on each files contained in `to_walk`
pub fn apply(
    to_walk: &str,
    sync_dir: &str,
    home_dir: &str,
    to_ignore: &[&str],
) -> Result<(), String> {
    'dir_item: for item_result in WalkDir::new(Path::new(&to_walk)) {
        let sync_file = match item_result {
            Err(e) => {
                println!("SKIPPING ({})", e);
                continue;
            }
            Ok(v) => v,
        };

        let rel_sync_file = sync_file
            .path()
            .to_str()
            .ok_or_else(|| format!("Can't get str for: {:?}", &sync_file))?
            .replace(sync_dir, "");
        let rel_sync_file = rel_sync_file.trim_matches('/');
        for ignore in to_ignore {
            if rel_sync_file.starts_with(ignore) {
                continue 'dir_item;
            }
        }

        if let Err(e) = symlink_file(&sync_file, sync_dir, home_dir) {
            println!("SKIPPING: {}", e);
        }
    }
    Ok(())
}

/// Symlinks `sync_file` to its counterpart in homedir
/// Homedir path is calculated by replacing `sync_dir` in `home_dir`
///
/// For example:
///
/// Assuming that there is a `.bashrc` file in `/home/joe/sync-dir` dir
///
/// `symlink_file(
///    &WalkDir::DirEntry("/home/joe/sync-dir/.bashrc"), "/home/joe/sync-dir", "/home/joe"
/// )`
///
/// will result as replacing:
///
/// `/home/joe/.bashrc`
///
/// with symlink to:
///
/// `/home/joe/sync-dir/.bashrc`
///
pub fn symlink_file(sync_file: &DirEntry, sync_dir: &str, home_dir: &str) -> Result<(), String> {
    let content_item_data = std::fs::metadata(sync_file.path())
        .map_err(|e| format!("Can't get file data {:?} ({})", &sync_file, e))?;
    if !content_item_data.is_file() {
        return Ok(());
    }
    let src_path = sync_file
        .path()
        .to_str()
        .ok_or_else(|| format!("Can't convert src file: {:?}", &sync_file))?;
    let user_file = swap_path_bases(src_path, sync_dir, home_dir);
    let user_file = Path::new(&user_file);
    let user_file_dir = user_file
        .parent()
        .and_then(|p| p.to_str())
        .ok_or_else(|| format!("Can't get parent dir for file: {:?}", &user_file))?;
    if let Err(e) = fs::create_dir_all(user_file_dir) {
        return Err(format!("Can't create dir: {} ({})", user_file_dir, e));
    }
    if user_file.exists() {
        if let Err(e) = std::fs::remove_file(&user_file) {
            return Err(format!("Can't remove {:?} ({})", &user_file, e));
        }
    }
    if let Err(e) = unix_fs::symlink(&sync_file.path(), &user_file) {
        return Err(format!(
            "Can't symlink {:?} to {:?} ({})",
            sync_file, user_file, e
        ));
    }
    println!("symlinked: {:?} -> {:?}", user_file, sync_file.path());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::fs;
    use std::io::prelude::*;
    use std::path::Path;
    use std::os::unix::fs as unix_fs;
    use tempdir::TempDir;

    #[test]
    fn init_saves_dir_to_init_in_config_file() {
        let homedir = TempDir::new("user1").unwrap();
        let config_dir = TempDir::new_in(homedir.path(), ".ff").unwrap();
        let config_file = config_dir.path().join("config.ini");
        let sync_dir = TempDir::new_in(homedir.path(), "dot-files").unwrap();
        let config = Config::new(config_file.as_path().to_str().unwrap()).unwrap();

        let result = init(sync_dir.path().to_str().unwrap(), &config).unwrap();

        assert_eq!(result, ());
        let mut f = File::open(config_file).unwrap();
        let mut config_file_src = String::new();
        f.read_to_string(&mut config_file_src).unwrap();
        assert_eq!(
            config_file_src.contains("ignore-when-apply=.git/,.hg/\n"),
            true
        );
        assert_eq!(
            config_file_src.contains(&format!("sync-dir={}\n", sync_dir.path().to_str().unwrap())),
            true
        );
    }

    #[test]
    fn add_file_works_ok() {
        let homedir = TempDir::new("user1").unwrap();
        let sync_dir = TempDir::new_in(homedir.path(), "dot-files").unwrap();
        let file_to_sync = homedir.path().join(".vimrc");
        File::create(&file_to_sync).unwrap();
        let synced_file = sync_dir.path().join(".vimrc");
        assert_eq!(synced_file.exists(), false);

        add(
            file_to_sync.to_str().unwrap(),
            &homedir.path().to_str().unwrap(),
            &sync_dir.path().to_str().unwrap(),
        ).unwrap();

        assert_eq!(synced_file.exists(), true);
        assert_eq!(
            fs::symlink_metadata(file_to_sync.to_str().unwrap())
                .unwrap()
                .file_type()
                .is_symlink(),
            true
        );
    }

    #[test]
    fn add_many_files_works_ok() {
        let homedir = TempDir::new("user1").unwrap();
        let sync_dir = TempDir::new_in(homedir.path(), "dot-files").unwrap();
        let mut files_to_sync = Vec::new();
        let mut synced_files = Vec::new();
        for filename in vec![".vimrc", ".bashrc"] {
            let filepath = homedir.path().join(filename);
            File::create(&filepath).unwrap();
            files_to_sync.push(filepath);
            synced_files.push(sync_dir.path().join(&filename).to_owned());
        }
        // checks file are not synced
        for file in &synced_files {
            assert_eq!(file.exists(), false);
        }

        add_files(
            &files_to_sync
                .iter()
                .map(|f| f.to_str().unwrap())
                .collect::<Vec<_>>(),
            homedir.path().to_str().unwrap(),
            sync_dir.path().to_str().unwrap(),
        );

        // checks file are synced
        for (idx, path) in synced_files.iter().enumerate() {
            assert_eq!(path.exists(), true);
            assert_eq!(
                fs::symlink_metadata(files_to_sync[idx].to_str().unwrap())
                    .unwrap()
                    .file_type()
                    .is_symlink(),
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
            fs::symlink_metadata(&home_file)
                .unwrap()
                .file_type()
                .is_symlink(),
            true
        );

        let result = remove(&home_file.to_str().unwrap()).unwrap();

        assert_eq!(result, ());
        assert_eq!(
            fs::symlink_metadata(&home_file)
                .unwrap()
                .file_type()
                .is_file(),
            true
        );
        assert_eq!(Path::new(&sync_file).exists(), false);
    }

    #[test]
    fn added_files_are_removed_correctly() {
        let homedir = TempDir::new("user1").unwrap();
        let sync_dir = TempDir::new_in(homedir.path(), "dot-files").unwrap();
        let mut synced_files = Vec::new();
        let mut files_to_restore = Vec::new();
        for filename in vec![".vimrc", ".bashrc"] {
            let synced_file = sync_dir.path().join(&filename);
            let file_to_restore = homedir.path().join(&filename);

            File::create(&synced_file).unwrap();
            unix_fs::symlink(
                &synced_file.to_str().unwrap(),
                &file_to_restore.to_str().unwrap(),
            ).unwrap();

            synced_files.push(synced_file);
            files_to_restore.push(file_to_restore);
        }
        for file in &files_to_restore {
            // checks that files are symlinks
            assert_eq!(
                fs::symlink_metadata(&file)
                    .unwrap()
                    .file_type()
                    .is_symlink(),
                true
            );
        }

        remove_files(&files_to_restore
            .iter()
            .map(|f| f.to_str().unwrap())
            .collect::<Vec<_>>());

        for file in files_to_restore {
            // checks that files are regular files
            assert_eq!(
                fs::symlink_metadata(&file).unwrap().file_type().is_file(),
                true
            );
        }
    }

    #[test]
    fn apply_works_for_single_file_which_missing() {
        let homedir = TempDir::new("user1").unwrap();
        let sync_dir = TempDir::new_in(homedir.path(), "dot-files").unwrap();
        let synced_file = sync_dir.path().join(".vimrc");
        File::create(&synced_file).unwrap();
        let user_file = homedir.path().join(".vimrc");
        assert_eq!(user_file.exists(), false);

        let result = apply(
            &sync_dir.path().to_str().unwrap(),
            &sync_dir.path().to_str().unwrap(),
            &homedir.path().to_str().unwrap(),
            &vec![],
        ).unwrap();

        assert_eq!(result, ());
        assert_eq!(
            fs::symlink_metadata(&user_file)
                .unwrap()
                .file_type()
                .is_symlink(),
            true
        );
    }

    #[test]
    fn apply_works_for_single_file_which_exists() {
        let homedir = TempDir::new("user1").unwrap();
        let sync_dir = TempDir::new_in(homedir.path(), "dot-files").unwrap();
        let synced_file = sync_dir.path().join(".vimrc");
        File::create(&synced_file).unwrap();
        let user_file = homedir.path().join(".vimrc");
        File::create(&user_file).unwrap();
        assert_eq!(
            fs::metadata(&user_file).unwrap().file_type().is_file(),
            true
        );

        let result = apply(
            &sync_dir.path().to_str().unwrap(),
            &sync_dir.path().to_str().unwrap(),
            &homedir.path().to_str().unwrap(),
            &vec![],
        ).unwrap();

        assert_eq!(result, ());
        assert_eq!(
            fs::symlink_metadata(&user_file)
                .unwrap()
                .file_type()
                .is_symlink(),
            true
        );
    }
}
