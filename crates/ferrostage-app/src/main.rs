//! FerroStage process entry.
//!
//! `just dev` runs this binary. It parses a small CLI, loads settings, and can
//! validate a settings file without writing it. Opening the presenter window
//! is optional behind the `gpui` feature.

#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use ferrostage_settings::{LoadOutcome, SettingsStore, SettingsWarning, from_toml};
use ferrostage_ui::{GPUI_KIT_VERSION, ZED_GPUI_TAG};
use tracing::{info, warn};

#[derive(Debug, Parser)]
#[command(
    name = "ferrostage",
    version,
    about = "GPU-accelerated PDF and Beamer presentation stage"
)]
struct Cli {
    /// Settings file used at startup. Does not change `check-config`, which only reads its path.
    #[arg(long, global = true)]
    config: Option<PathBuf>,

    /// PDF opened read-only after the presenter window is ready.
    pdf: Option<PathBuf>,

    /// Exact PDFium library file. Defaults to pdfium.dll beside the executable.
    #[arg(long)]
    pdfium_library: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Parse a settings TOML file and print the resolved values. This does not write.
    CheckConfig {
        /// Path to a TOML settings file.
        path: PathBuf,
    },
}

fn runtime_log_filter(directives: Option<&str>) -> tracing_subscriber::EnvFilter {
    directives
        .and_then(|value| tracing_subscriber::EnvFilter::try_new(value).ok())
        .unwrap_or_else(|| tracing_subscriber::EnvFilter::new("warn"))
}

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_file(true)
        .with_line_number(true)
        .with_env_filter(runtime_log_filter(
            std::env::var("RUST_LOG").ok().as_deref(),
        ))
        .init();

    let cli = Cli::parse();
    match cli.command {
        None => start_app(cli.config, cli.pdf, cli.pdfium_library),
        Some(Command::CheckConfig { path }) => check_config(&path),
    }
}

fn start_app(
    config: Option<PathBuf>,
    pdf: Option<PathBuf>,
    pdfium_library: Option<PathBuf>,
) -> ExitCode {
    info!(
        zed = ZED_GPUI_TAG,
        gpui_kit = GPUI_KIT_VERSION,
        "starting FerroStage"
    );
    let store = match resolve_store(config) {
        Ok(store) => store,
        Err(message) => return fail_before_window(None, &message),
    };
    match store.open_for_startup() {
        Ok(session) => {
            info!(path = %store.path().display(), "loaded settings");
            print_warnings(&session.warnings);
            if let Some(error) = &session.save_error {
                eprintln!("ferrostage: settings were not saved: {error}");
            }
            #[cfg(feature = "gpui")]
            {
                native_shell(store, session, pdf, pdfium_library)
            }
            #[cfg(not(feature = "gpui"))]
            {
                let _inputs = (pdf, pdfium_library);
                println!(
                    "ferrostage {version} skeleton — presenter UI is not linked yet (GPUI pin {ZED_GPUI_TAG})",
                    version = env!("CARGO_PKG_VERSION"),
                );
                ExitCode::SUCCESS
            }
        }
        Err(error) => {
            let message = error.to_string();
            fail_before_window(Some(store.path()), &message)
        }
    }
}

// Pre-window failures stay visible when the release binary has no console.
// Append the same `ferrostage:` line to startup-error.log in the settings
// parent, or beside the executable when that directory cannot be used.
fn fail_before_window(settings_path: Option<&Path>, message: &str) -> ExitCode {
    let line = format!("ferrostage: {message}");
    eprintln!("{line}");
    if let Err(error) = append_startup_error(settings_path, &line) {
        warn!(error = %error, "failed to append startup-error.log");
    }
    ExitCode::from(1)
}

fn append_startup_error(settings_path: Option<&Path>, line: &str) -> io::Result<()> {
    let path = startup_log_directory(settings_path)?.join("startup-error.log");
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    writeln!(file, "{line}")?;
    Ok(())
}

fn startup_log_directory(settings_path: Option<&Path>) -> io::Result<PathBuf> {
    if let Some(directory) = settings_path.and_then(existing_parent_directory) {
        return Ok(directory);
    }
    let executable = std::env::current_exe()?;
    executable.parent().map(Path::to_path_buf).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "executable path has no parent directory",
        )
    })
}

fn existing_parent_directory(path: &Path) -> Option<PathBuf> {
    let parent = path.parent()?;
    let parent = if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    };
    parent.is_dir().then(|| parent.to_path_buf())
}

#[cfg(feature = "gpui")]
fn native_shell(
    store: SettingsStore,
    session: ferrostage_settings::SettingsSession,
    pdf: Option<PathBuf>,
    library: Option<PathBuf>,
) -> ExitCode {
    let library_path = match library {
        Some(path) => path,
        None => match std::env::current_exe() {
            Ok(path) => path.with_file_name("pdfium.dll"),
            Err(error) => {
                eprintln!("ferrostage: cannot resolve executable path: {error}");
                return ExitCode::from(1);
            }
        },
    };
    let renderer = ferrostage_render::RenderHandle::spawn(
        ferrostage_pdf::RuntimeConfig {
            library_path,
            font_directories: Vec::new(),
        },
        ferrostage_render::CacheBudgets {
            l2_bytes: session.settings.cache.l2_cpu_mib as usize * 1024 * 1024,
            l3_bytes: session.settings.cache.l3_thumb_mib as usize * 1024 * 1024,
        },
    );
    let initial_monitors = ferrostage_platform::enumerate_monitors();
    let (sink, topology) = std::sync::mpsc::channel();
    let watcher = ferrostage_platform::watch_topology(sink);
    ferrostage_ui::run(ferrostage_ui::ShellInputs {
        settings: session,
        store,
        renderer,
        topology,
        watcher,
        initial_monitors,
        initial_pdf: pdf,
    });
    ExitCode::SUCCESS
}

fn resolve_store(config: Option<PathBuf>) -> Result<SettingsStore, String> {
    let path = match config {
        Some(path) => path,
        None => SettingsStore::default_path().map_err(|error| error.to_string())?,
    };
    Ok(SettingsStore::at(path))
}

fn check_config(path: &Path) -> ExitCode {
    match read_config(path) {
        Ok(outcome) => {
            print_warnings(&outcome.warnings);
            println!("{:#?}", outcome.settings);
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("ferrostage: {message}");
            ExitCode::from(1)
        }
    }
}

fn read_config(path: &Path) -> Result<LoadOutcome, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    from_toml(&text).map_err(|error| error.to_string())
}

fn print_warnings(warnings: &[SettingsWarning]) {
    for warning in warnings {
        eprintln!("warning: {}: {}", warning.key, warning.reason);
    }
}

#[cfg(test)]
mod logging_tests {
    use super::runtime_log_filter;
    use std::{
        io::Write,
        sync::{Arc, Mutex},
    };

    #[derive(Clone)]
    struct Buffer(Arc<Mutex<Vec<u8>>>);
    impl Write for Buffer {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.lock().expect("log lock").extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    fn capture(filter: Option<&str>) -> String {
        let bytes = Arc::new(Mutex::new(Vec::new()));
        let writer = Buffer(bytes.clone());
        let subscriber = tracing_subscriber::fmt()
            .without_time()
            .with_ansi(false)
            .with_env_filter(runtime_log_filter(filter))
            .with_writer(move || writer.clone())
            .finish();
        tracing::subscriber::with_default(subscriber, || {
            tracing::debug!(target: "dependency", "diagnostic-detail");
            tracing::info!("routine-startup");
            tracing::warn!("visible-warning");
            tracing::error!("visible-error");
        });
        String::from_utf8(bytes.lock().expect("log lock").clone()).expect("UTF-8 log")
    }

    #[test]
    fn runtime_is_quiet_by_default_and_keeps_diagnostic_override() {
        let quiet = capture(None);
        assert!(!quiet.contains("routine-startup") && !quiet.contains("diagnostic-detail"));
        assert!(quiet.contains("visible-warning") && quiet.contains("visible-error"));
        let verbose = capture(Some("debug"));
        assert!(verbose.contains("routine-startup") && verbose.contains("diagnostic-detail"));
    }
}
