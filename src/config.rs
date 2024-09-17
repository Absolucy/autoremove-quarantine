use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::path::{Path, PathBuf};

static DEFAULT_CONFIG: &str = include_str!("watch.list.default");

#[instrument]
pub fn get_config() -> Result<Vec<PathBuf>> {
	let config_dir = ProjectDirs::from("moe", "absolucy", "autoremove-quarantine")
		.map(|dirs| dirs.config_dir().to_path_buf())
		.context("failed to get app config directory")?;
	let config_file = config_dir.join("watch.list");
	debug!("config directory: {}", config_dir.display());
	debug!("config file: {}", config_file.display());
	if !config_dir.is_dir() {
		info!("config directory doesn't exist, creating it");
		std::fs::create_dir_all(&config_dir).with_context(|| {
			format!(
				"failed to create config directory '{}'",
				config_dir.display()
			)
		})?;
	}
	if !config_file.exists() {
		info!("config file doesn't exist, writing default");
		std::fs::write(&config_file, DEFAULT_CONFIG).with_context(|| {
			format!(
				"failed to write default config to '{}'",
				config_file.display()
			)
		})?;
	}
	parse_config_file(&config_file)
}

#[instrument]
fn parse_config_file(path: &Path) -> Result<Vec<PathBuf>> {
	let home_dir = std::env::var("HOME")
		.map(PathBuf::from)
		.context("failed to get HOME env var")?;
	let contents = std::fs::read_to_string(path).context("failed to read file")?;
	Ok(contents
		.lines()
		.map(|line| line.trim())
		.filter(|line| !line.is_empty() && !line.starts_with('#'))
		.map(PathBuf::from)
		.map(|path| {
			if path.is_relative() {
				home_dir.join(path)
			} else {
				path
			}
		})
		.collect())
}
