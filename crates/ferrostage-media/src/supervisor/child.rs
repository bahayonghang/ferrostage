use super::*;
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    process::{Child, Command, Stdio},
    sync::{
        atomic::AtomicU64,
        mpsc::{self, SyncSender},
    },
};
const PLUGIN_SCANNER: &str = "libexec/gst-plugin-scanner.exe";
static TEMP_ID: AtomicU64 = AtomicU64::new(1);
pub(super) struct PreparedSource {
    pub path: PathBuf,
    directory: Option<PathBuf>,
}
impl PreparedSource {
    pub fn new(source: &Source) -> Result<Self, MediaError> {
        match source {
            Source::External(path) => {
                let path = path
                    .canonicalize()
                    .map_err(|_| MediaError::SourceUnavailable)?;
                let meta = fs::metadata(&path).map_err(|_| MediaError::SourceUnavailable)?;
                if !meta.is_file() {
                    return Err(MediaError::SourceUnavailable);
                }
                if meta.len() > crate::MAX_SOURCE_BYTES {
                    return Err(MediaError::Limit);
                }
                Ok(Self {
                    path,
                    directory: None,
                })
            }
            Source::Embedded { name, bytes } => {
                if bytes.len() as u64 > crate::MAX_SOURCE_BYTES {
                    return Err(MediaError::Limit);
                }
                let directory = std::env::temp_dir().join(format!(
                    "ferrostage-media-{}-{}",
                    std::process::id(),
                    TEMP_ID.fetch_add(1, Ordering::Relaxed)
                ));
                fs::create_dir(&directory).map_err(|_| MediaError::SourceUnavailable)?;
                let ext = std::path::Path::new(name)
                    .extension()
                    .and_then(|e| e.to_str())
                    .filter(|e| e.len() <= 10 && e.chars().all(|c| c.is_ascii_alphanumeric()))
                    .unwrap_or("bin");
                let source = Self {
                    path: directory.join(format!("payload.{ext}")),
                    directory: Some(directory),
                };
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&source.path)
                    .and_then(|mut f| f.write_all(bytes))
                    .map_err(|_| MediaError::SourceUnavailable)?;
                Ok(source)
            }
        }
    }
}
impl Drop for PreparedSource {
    fn drop(&mut self) {
        if let Some(dir) = &self.directory {
            if let Err(e) = fs::remove_file(&self.path) {
                tracing::debug!(%e,"media temporary payload cleanup");
            }
            if let Err(e) = fs::remove_dir(dir) {
                tracing::debug!(%e,"media temporary directory cleanup");
            }
        }
    }
}
#[derive(Default)]
pub(super) struct Output {
    pub frame: Option<(u64, u64, u64, ferrostage_core::BgraFrame)>,
    pub progress: Option<(u64, u64)>,
    pub ended: bool,
    error: Option<MediaError>,
    received: bool,
    eof: bool,
}
fn read_output(stdout: &mut impl Read, output: &Mutex<Output>) {
    let mut last_sequence = 0;
    loop {
        let packet = protocol::read(stdout);
        let Ok(mut out) = output.lock() else { break };
        match packet {
            Ok(Packet::Frame {
                sequence,
                position_ms,
                duration_ms,
                pixels,
            }) => {
                if sequence <= last_sequence {
                    out.error = Some(MediaError::Protocol);
                    break;
                }
                last_sequence = sequence;
                out.received = true;
                out.frame = Some((sequence, position_ms, duration_ms, pixels));
            }
            Ok(Packet::Status {
                position_ms,
                duration_ms,
                ..
            }) => {
                out.received = true;
                out.progress = Some((position_ms, duration_ms));
            }
            Ok(Packet::End) => {
                out.received = true;
                out.ended = true;
            }
            Ok(Packet::Error(error)) => {
                out.received = true;
                out.error = Some(error);
            }
            Ok(_) => {
                out.error = Some(MediaError::Protocol);
                break;
            }
            Err(_) => {
                out.eof = true;
                break;
            }
        }
    }
}
pub(super) struct Native {
    child: Child,
    input: SyncSender<Packet>,
    output: Arc<Mutex<Output>>,
    started: Instant,
    received: bool,
    ended: bool,
    threads: Vec<thread::JoinHandle<()>>,
}
impl Native {
    pub fn new(runtime: &Runtime, open: protocol::Open) -> Result<Self, MediaError> {
        if !runtime.helper.is_file()
            || include_str!("../runtime-files.txt")
                .lines()
                .any(|file| !runtime.directory.join(file).is_file())
        {
            return Err(MediaError::RuntimeUnavailable);
        }
        let mut command = Command::new(&runtime.helper);
        let paths = std::iter::once(runtime.directory.join("bin"))
            .chain(std::env::split_paths(
                &std::env::var_os("PATH").unwrap_or_default(),
            ))
            .collect::<Vec<_>>();
        command
            .env(
                "PATH",
                std::env::join_paths(paths).map_err(|_| MediaError::RuntimeUnavailable)?,
            )
            .env(
                "GST_PLUGIN_PATH_1_0",
                runtime.directory.join("lib/gstreamer-1.0"),
            )
            .env(
                "GST_PLUGIN_SYSTEM_PATH_1_0",
                runtime.directory.join("lib/gstreamer-1.0"),
            )
            .env(
                "GST_PLUGIN_SCANNER_1_0",
                runtime.directory.join(PLUGIN_SCANNER),
            )
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        let mut child = command
            .spawn()
            .map_err(|_| MediaError::RuntimeUnavailable)?;
        let Some(mut stdin) = child.stdin.take() else {
            return Err(MediaError::Process);
        };
        let Some(mut stdout) = child.stdout.take() else {
            return Err(MediaError::Process);
        };
        let Some(mut stderr) = child.stderr.take() else {
            return Err(MediaError::Process);
        };
        let output = Arc::new(Mutex::new(Output::default()));
        let reader = output.clone();
        let (input, commands) = mpsc::sync_channel(4);
        let threads = vec![
            thread::spawn(move || {
                while let Ok(packet) = commands.recv() {
                    if protocol::write(&mut stdin, &packet).is_err() {
                        break;
                    }
                }
            }),
            thread::spawn(move || read_output(&mut stdout, &reader)),
            thread::spawn(move || {
                let mut buffer = [0; 4096];
                let mut retained = Vec::new();
                loop {
                    match stderr.read(&mut buffer) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => {
                            let left = 4096usize.saturating_sub(retained.len());
                            retained.extend_from_slice(&buffer[..n.min(left)]);
                        }
                    }
                }
            }),
        ];
        let mut native = Self {
            child,
            input,
            output,
            started: Instant::now(),
            received: false,
            ended: false,
            threads,
        };
        native.command(Packet::Open(open))?;
        Ok(native)
    }
    pub fn command(&mut self, packet: Packet) -> Result<(), MediaError> {
        self.input.try_send(packet).map_err(|_| MediaError::Process)
    }
    pub fn poll(&mut self) -> Result<Output, MediaError> {
        let out = {
            let mut latest = self.output.lock().map_err(|_| MediaError::Process)?;
            std::mem::take(&mut *latest)
        };
        self.received |= out.received;
        self.ended |= out.ended;
        if let Some(error) = out.error {
            return Err(error);
        }
        if out.eof && !self.ended {
            return Err(if self.received {
                MediaError::Process
            } else {
                MediaError::RuntimeUnavailable
            });
        }
        if let Some(status) = self.child.try_wait().map_err(|_| MediaError::Process)?
            && !self.ended
        {
            return Err(if self.received || status.success() {
                MediaError::Process
            } else {
                MediaError::RuntimeUnavailable
            });
        }
        if !self.received && self.started.elapsed() > Duration::from_secs(10) {
            return Err(MediaError::Timeout);
        }
        Ok(Output {
            ended: self.ended,
            ..out
        })
    }
}
impl Drop for Native {
    fn drop(&mut self) {
        if self.input.try_send(Packet::Stop).is_err() {
            tracing::debug!("media stop pipe unavailable");
        }
        let deadline = Instant::now() + Duration::from_millis(300);
        loop {
            match self.child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(5)),
                _ => {
                    if let Err(e) = self.child.kill() {
                        tracing::debug!(%e,"media process kill");
                    }
                    if let Err(e) = self.child.wait() {
                        tracing::warn!(%e,"media process reap failed");
                    }
                    break;
                }
            }
        }
        // Only completed pipe readers are joined; Drop never waits on pipe IO.
        for handle in self.threads.drain(..) {
            if handle.is_finished() && handle.join().is_err() {
                tracing::warn!("media pipe reader panicked");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scanner_path_matches_selected_runtime_manifest() {
        assert!(
            include_str!("../runtime-files.txt")
                .lines()
                .any(|file| file == PLUGIN_SCANNER)
        );
    }
    #[test]
    fn child_frames_reject_duplicate_and_regressing_sequence() {
        for next in [2, 1] {
            let mut bytes = Vec::new();
            for sequence in [2, next] {
                protocol::write(
                    &mut bytes,
                    &Packet::Frame {
                        sequence,
                        position_ms: 0,
                        duration_ms: 100,
                        pixels: ferrostage_core::BgraFrame::new(1, 1, 4, vec![sequence as u8; 4])
                            .unwrap(),
                    },
                )
                .unwrap();
            }
            let output = Mutex::new(Output::default());
            read_output(&mut bytes.as_slice(), &output);
            let output = output.lock().unwrap();
            assert_eq!(output.error, Some(MediaError::Protocol));
            assert_eq!(output.frame.as_ref().unwrap().0, 2);
            assert_eq!(output.frame.as_ref().unwrap().3.pixels_bgra(), &[2; 4]);
        }
    }
    #[test]
    fn embedded_payload_is_owned_create_new_and_removed_after_release() {
        let source = PreparedSource::new(&Source::Embedded {
            name: "../../anything.gif".into(),
            bytes: Arc::from([1u8, 2, 3]),
        })
        .unwrap();
        let path = source.path.clone();
        let dir = path.parent().unwrap().to_path_buf();
        assert_eq!(path.file_name().unwrap(), "payload.gif");
        assert_eq!(fs::read(&path).unwrap(), [1, 2, 3]);
        drop(source);
        assert!(!path.exists());
        assert!(!dir.exists());
    }
}
