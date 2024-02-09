pub mod kiara;
pub mod ring;

use crate::kiara::Kiara;
use clap::Parser;
use color_eyre::eyre::{bail, Result};
use manifest_dir_macros::{directory_relative_path, file_relative_path};
use stardust_xr_fusion::{client::Client, items::ItemUI};
use std::env::set_var;
use tokio::process::Command;
use tracing_subscriber::EnvFilter;

#[derive(Debug, clap::Parser)]
pub struct Cli {
	#[arg(long = "", num_args(0..))]
	niri_launch: Vec<String>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
	tracing_subscriber::fmt()
		.compact()
		.with_env_filter(EnvFilter::from_env("LOG_LEVEL"))
		.init();
	let args = Cli::parse();
	let (client, event_loop) = Client::connect_with_async_loop().await?;
	client.set_base_prefixes(&[directory_relative_path!("res")]);

	let kiara = client.wrap_root(Kiara::new())?;
	let _item_ui_wrapped = ItemUI::register(&client)?.wrap_raw(kiara)?;
	let env = client.get_connection_environment()?.await?;
	for (key, value) in env {
		set_var(key, value);
	}
	let mut niri_open = Command::new("niri")
		.args(&["-c", file_relative_path!("src/niri_config.kdl"), "--"])
		.args(&args.niri_launch)
		.spawn()?;

	let result = tokio::select! {
		n = niri_open.wait() => {
			println!("Niri stopped. Output status: {:?}", n);
			Ok(())
		},
		_ = tokio::signal::ctrl_c() => {
			niri_open.kill().await?;
			Ok(())
		},
		_ = event_loop => {
			niri_open.kill().await?;
			bail!("Server crashed")
		}
	};
	result
}
