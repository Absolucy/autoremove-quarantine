use anyhow::{Context, Result};
use std::path::PathBuf;

static DEFAULT_LAUNCHAGENT: &str = include_str!("moe.absolucy.autoremove-quarantine.plist");

pub(crate) fn try_self_install() -> Result<bool> {
	if !std::env::args().any(|arg| arg.trim().eq_ignore_ascii_case("install")) {
		return Ok(false);
	}
	let current_path = std::env::current_exe().context("failed to get current executable path")?;
	let current_dir =
		std::env::current_dir().context("failed to get current executable directory")?;
	let home_dir = std::env::var("HOME")
		.map(PathBuf::from)
		.context("failed to get home directory")?;
	assert!(home_dir.is_dir(), "home directory doesn't exist!");
	let target_path = if current_dir.ends_with("bin")
		|| current_dir.starts_with("/opt/homebrew")
		|| current_dir.starts_with("/usr/local")
	{
		current_path
	} else {
		let local_bin_dir = home_dir.join(".local/bin");
		if !local_bin_dir.exists() {
			std::fs::create_dir_all(&local_bin_dir)
				.with_context(|| format!("failed to create {}", local_bin_dir.display()))?;
		}
		let target_path = local_bin_dir.join("autoremove-quarantine");
		std::fs::copy(&current_path, &target_path)
			.with_context(|| format!("failed to copy to {}", target_path.display()))?;
		target_path
	};
	let launchd_plist = DEFAULT_LAUNCHAGENT.replace(
		"/usr/local/bin/autoremove-quarantine",
		target_path.to_string_lossy().as_ref(),
	);
	let launchagents_dir = home_dir.join("Library/LaunchAgents");
	if !launchagents_dir.exists() {
		std::fs::create_dir_all(&launchagents_dir)
			.with_context(|| format!("failed to create {}", launchagents_dir.display()))?;
	}
	let launchd_plist_path = launchagents_dir.join("moe.absolucy.autoremove-quarantine.plist");
	std::fs::write(&launchd_plist_path, launchd_plist).with_context(|| {
		format!(
			"failed to write launchagent plist to {}",
			launchd_plist_path.display()
		)
	})?;
	println!(
		"=== INSTALL DONE! ===\nRun the following command to start the service:\n\nlaunchctl load \
		 -w \"{}\"\n",
		launchd_plist_path.display()
	);
	Ok(true)
}
