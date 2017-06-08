//! relates to config management for `ff` like (`Cofnig` manager)
use std;
use std::path::Path;
use ini::Ini;


/// Config manager storing data in files
#[derive(Debug)]
pub struct Config {
	path: String,
}

#[allow(missing_docs)]
impl Config {
	/// Gives instance of `Config` stored at `path`
	///
	/// Config file is created (including necessery dirs.) if not exists
	pub fn new<T>(path: T) -> Result<Config, String> where T: AsRef<str> {
		let config = Config { path: path.as_ref().to_owned() };
		config.create_config_dir()?;
		Ok(config)
	}

	/// Returns path to file where config is stored
	pub fn get_path(&self) -> &str {
		&self.path
	}

	/// Creates config file (including missing dirs.) from `self.path`
	fn create_config_dir(&self) -> Result<(), String> {
		let path = Path::new(&self.path);
		let parent_dir = path.parent()
			.ok_or(format!("Can't get parent for {}", self.path))?;
		if parent_dir.exists() == false {
			std::fs::create_dir_all(&parent_dir)
				.map_err(|e| format!("Can't create config file dir: {:?} ({})", &parent_dir, e))?;
		}
		if !Path::new(&self.path).exists() {
			std::fs::File::create(path)
				.map_err(|e| format!("Can't create config file {:?} ({:?})", path, e))?;
		}
		Ok(())
	}

	/// Returns config value for `key` wrapped with `Option` and `Result`
	pub fn get(&self, key: &str) -> Result<Option<String>, String> {
		let conf = Ini::load_from_file(self.path.as_str())
			.map_err(|e| format!("Can't load config file: {} ({})", self.path, e))?;

		let v = conf.section(None::<String>)
			.and_then(|s| s.get(key).cloned())
			.or_else(|| None);
		Ok(v)
	}

	/// Sets and save `value` under `key` in file
	pub fn set(&self, key: &str, value: &str) -> Result<(), String> {
		let mut conf = Ini::new();
		conf.with_section(None::<String>).set(key, value);
		if let Err(e) = conf.write_to_file(self.path.as_str()) {
			return Err(format!("Can't save {}={} to {} ({})", &key, &value, self.path, e));
		}
		Ok(())
	}

}