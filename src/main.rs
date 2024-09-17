#[macro_use]
extern crate tracing;

mod config;

use anyhow::{Context, Result};
use crossbeam_channel::unbounded;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use tracing_oslog::OsLogger;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing_unwrap::ResultExt;

fn main() -> Result<()> {
	setup_logger()?;
	let (tx, rx) = unbounded();
	let mut watcher = RecommendedWatcher::new(tx, Config::default())
		.context("failed to setup fs event watcher")
		.unwrap_or_log();
	let folders = config::get_config()
		.context("failed to get configuration")
		.unwrap_or_log();
	for folder in folders {
		match watcher.watch(&folder, RecursiveMode::Recursive) {
			Ok(_) => {
				info!("watching {}", folder.display());
			}
			Err(err) => {
				error!("failed to watch {}: {err:?}", folder.display());
			}
		}
	}
	for received in rx {
		if let Some(event) = received.context("failed to receive event").ok_or_log() {
			on_event(event);
		}
	}
	Ok(())
}

#[instrument]
fn on_event(event: Event) {
	debug!("Change: {event:?}");
	if event.kind.is_create() {
		for path in event.paths {
			if !path.exists() {
				// why
				continue;
			}
			let did_unquarantine = unquarantine(&path)
				.with_context(|| format!("failed to unquarantine {}", path.display()))
				.ok_or_log()
				.unwrap_or(false);
			if did_unquarantine {
				info!("unquarantined {}", path.display());
			}
		}
	}
}

#[instrument]
fn setup_logger() -> Result<()> {
	let os_logger = OsLogger::new("moe.absolucy.autoremove-quarantine", "default");
	if cfg!(debug_assertions) {
		tracing_subscriber::registry()
			.with(tracing_subscriber::fmt::layer())
			.with(os_logger)
			.try_init()
			.context("failed to set global subscriber")
	} else {
		tracing_subscriber::registry()
			.with(os_logger)
			.try_init()
			.context("failed to set global subscriber")
	}
}

#[instrument]
fn unquarantine(path: &Path) -> Result<bool> {
	match xattr::get(path, "com.apple.quarantine")
		.context("failed to check for quarantine attribute")?
	{
		Some(_) => xattr::remove(path, "com.apple.quarantine")
			.context("failed to remove quarantine attribute")
			.map(|_| true),
		None => Ok(false),
	}
}
