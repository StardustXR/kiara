pub mod kiara;
pub mod ring;

use crate::kiara::Kiara;
use clap::Parser;
use color_eyre::eyre::{bail, Result};
use stardust_xr_fusion::{client::Client, items::ItemUI};
use std::env::set_var;
use tokio::process::Command;
use tracing_subscriber::EnvFilter;

use lazy_static::lazy_static;

lazy_static! {
	static ref RING_GLB: Vec<u8> = include_bytes!("../res/kiara/ring.glb").to_vec();
	static ref KIARA_BLEND: Vec<u8> = include_bytes!("../assets/kiara.blend").to_vec();
	static ref KIARA_BLEND1: Vec<u8> = include_bytes!("../assets/kiara.blend1").to_vec();
	static ref NIRI_CONFIG_KDL: Vec<u8> = include_bytes!("../src/niri_config.kdl").to_vec();
}

#[derive(Debug, clap::Parser)]
pub struct Cli {
	#[arg(long = "", num_args(0..))]
	niri_launch: Vec<String>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
	// Make temporary directory for resources and copy resources to it
	let temp_dir = tempfile::tempdir().unwrap();
	let temp_dir_path = temp_dir.path().to_str().unwrap();
	std::fs::create_dir_all(format!("{}/res/kiara", temp_dir_path)).expect("Failed to create res/kiara directory");
	std::fs::create_dir_all(format!("{}/assets", temp_dir_path)).expect("Failed to create assets directory");
	std::fs::create_dir_all(format!("{}/src", temp_dir_path)).expect("Failed to create src directory");
	std::fs::write(format!("{}/res/kiara/ring.glb", temp_dir_path), &*RING_GLB).expect("Failed to write ring.glb");
	std::fs::write(format!("{}/assets/kiara.blend", temp_dir_path), &*KIARA_BLEND).expect("Failed to write kiara.blend");
	std::fs::write(format!("{}/assets/kiara.blend1", temp_dir_path), &*KIARA_BLEND1).expect("Failed to write kiara.blend1");
	std::fs::write(format!("{}/src/niri_config.kdl", temp_dir_path), &*NIRI_CONFIG_KDL).expect("Failed to write niri_config.kdl");

	tracing_subscriber::fmt()
		.compact()
		.with_env_filter(EnvFilter::from_env("LOG_LEVEL"))
		.init();
	let args = Cli::parse();
	let (client, event_loop) = Client::connect_with_async_loop().await?;
	client.set_base_prefixes(&[format!("{}/res", temp_dir_path).as_str()]);

	let kiara = client.wrap_root(Kiara::new())?;
	let _item_ui_wrapped = ItemUI::register(&client)?.wrap_raw(kiara)?;
	let env = client.get_connection_environment()?.await?;
	for (key, value) in env {
		set_var(key, value);
	}
	let mut niri_open = Command::new("niri")
		.args(&["-c", format!("{}/src/niri_config.kdl", temp_dir_path).as_str(), "--"])
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
