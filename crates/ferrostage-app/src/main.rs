//! FerroStage process entry.
//!
//! `just dev` runs this binary. It parses a small CLI and can validate a
//! settings file. Opening the presenter window waits on the GPUI task.

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use ferrostage_settings::{Settings, from_toml};
use ferrostage_ui::{GPUI_KIT_VERSION, ZED_GPUI_TAG};
use tracing::info;

#[derive(Debug, Parser)]
#[command(
    name = "ferrostage",
    version,
    about = "GPU-accelerated PDF and Beamer presentation stage"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Parse a settings TOML file and print the resolved values.
    CheckConfig {
        /// Path to a TOML settings file.
        path: PathBuf,
    },
}

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    match Cli::parse().command {
        None => {
            info!(zed = ZED_GPUI_TAG, gpui_kit = GPUI_KIT_VERSION, "skeleton");
            println!(
                "ferrostage {version} skeleton — presenter UI is not linked yet (GPUI pin {ZED_GPUI_TAG})",
                version = env!("CARGO_PKG_VERSION"),
            );
            ExitCode::SUCCESS
        }
        Some(Command::CheckConfig { path }) => match check_config(&path) {
            Ok(settings) => {
                println!("{settings:#?}");
                ExitCode::SUCCESS
            }
            Err(message) => {
                eprintln!("ferrostage: {message}");
                ExitCode::from(1)
            }
        },
    }
}

fn check_config(path: &std::path::Path) -> Result<Settings, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    from_toml(&text).map_err(|error| error.to_string())
}
