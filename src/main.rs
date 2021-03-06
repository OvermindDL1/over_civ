use anyhow::Context;
use over_civ::{core::ClientType, prelude::*};
use std::path::PathBuf;
use structopt::StructOpt;
use tracing::log::LevelFilter;
use unic_langid::LanguageIdentifier;

#[derive(StructOpt, Debug)]
#[structopt()]
pub struct CLIOpts {
	/// Logging level override to bypass the logging config, can be: off, error, warn, info, debug, trace
	#[structopt(short, long, parse(try_from_str))]
	log_level: Option<LevelFilter>,

	/// Path to a directory to store configuration files
	#[structopt(short, long, parse(from_str))]
	config_dir: Option<PathBuf>,

	/// Which Client to select
	#[structopt(long)]
	client: Option<ClientType>,

	/// Do not include server code, this will allow you to join a server but not play locally
	#[structopt(long)]
	no_server: bool,

	/// Load game configuration file, this will generate a new file then exit if it doesn't exit so
	/// as to allow it to be filled out manually before actually loading it.
	#[cfg(feature = "server")]
	#[structopt(long)]
	load_game: Option<PathBuf>,

	/// Override the in-game language via the specified language code
	#[structopt(long)]
	language: Option<LanguageIdentifier>,
}

fn main() -> anyhow::Result<()> {
	#[allow(unused_assignments)]
	let default_client_type = {
		let mut preferred_client_type = ClientType::Logger;
		#[cfg(feature = "client_tui")]
		{
			preferred_client_type = ClientType::TUI;
		}
		#[cfg(feature = "client_wgpu")]
		if cfg!(target_os = "linux") || std::env::var("DISPLAY").is_ok() {
			preferred_client_type = ClientType::WGPU;
		}
		preferred_client_type
	};

	let opts = CLIOpts::from_args();

	let client_type = opts.client.unwrap_or(default_client_type);

	let mut engine = Engine::new(opts.config_dir.unwrap_or(PathBuf::from("./config")))?;
	engine.override_logging_level(opts.log_level);
	#[cfg(feature = "server")]
	engine.load_game_configuration(opts.load_game.or_else(|| {
		if client_type == ClientType::Logger {
			tracing::warn!("Logger-only client selected but no server file was set to be loaded, defaulting to `saves/server`");
			Some(PathBuf::new().join("saves").join("server"))
		} else {None}
	}));
	engine.set_include_server(!opts.no_server);
	engine.set_client_type(client_type);
	engine.run().context("Failed to run the engine")
}
