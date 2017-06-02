//! helpers related to managing config in ff
use std::env;
use std::path::PathBuf;
use ini::Ini;

/// Returns path to config file
pub fn get_config_file_path() -> PathBuf {
	let mut config_dir = env::home_dir().unwrap();
	config_dir.push(".ff");
	config_dir.push("config.ini");
	return config_dir
}

/// Returns dir under sync from config file
pub fn get_sync_dir_path() -> String {
	let conf = Ini::load_from_file(
		get_config_file_path().to_str().unwrap(),
	).unwrap();
	let section = conf.section(None::<String>).unwrap();
	return section.get("tracking-dir").unwrap().to_owned();
}