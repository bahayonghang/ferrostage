//! Read-only external replacement watcher. Native events never authorize a write.
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::{
    fs,
    io::{Read, Seek, SeekFrom},
    path::Path,
    sync::mpsc::{self, Sender},
    thread::{self, JoinHandle},
    time::{Duration, Instant, SystemTime},
};

const POLL: Duration = Duration::from_millis(25);
const STABLE: Duration = Duration::from_millis(100);
const TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug)]
pub enum PdfWatchEvent {
    Invalidated(u64),
    Ready { attempt: u64, bytes: Vec<u8> },
    TimedOut(u64),
}

pub struct PdfWatcher {
    stop: Sender<()>,
    thread: Option<JoinHandle<()>>,
}
impl Drop for PdfWatcher {
    fn drop(&mut self) {
        self.stop.send(()).ok();
        if let Some(thread) = self.thread.take() {
            thread.join().ok();
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Stamp {
    len: u64,
    modified: SystemTime,
}
fn stamp(path: &Path) -> std::io::Result<Stamp> {
    let metadata = fs::metadata(path)?;
    if !metadata.is_file() {
        return Err(std::io::Error::other("PDF path is not a file"));
    }
    Ok(Stamp {
        len: metadata.len(),
        modified: metadata.modified()?,
    })
}

#[derive(Default)]
struct Stability {
    attempt: u64,
    started: Option<Duration>,
    changed: Duration,
    observed: Option<(Stamp, Duration)>,
}
impl Stability {
    fn changed(&mut self, now: Duration) -> u64 {
        self.attempt += 1;
        self.started.get_or_insert(now);
        self.changed = now;
        self.observed = None;
        self.attempt
    }
    fn timed_out(&mut self, now: Duration) -> bool {
        if self
            .started
            .is_some_and(|start| now.saturating_sub(start) >= TIMEOUT)
        {
            self.started = None;
            self.observed = None;
            return true;
        }
        false
    }
    fn ready(&mut self, now: Duration, debounce: Duration, current: Option<Stamp>) -> bool {
        if self.started.is_none() || now.saturating_sub(self.changed) < debounce {
            return false;
        }
        match (self.observed, current) {
            (Some((before, since)), Some(current)) if before == current => {
                now.saturating_sub(since) >= STABLE
            }
            (_, Some(current)) => {
                self.observed = Some((current, now));
                false
            }
            (_, None) => {
                self.observed = None;
                false
            }
        }
    }
}

fn matches(event: &Event, target: &Path) -> bool {
    matches!(
        event.kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    ) && event.paths.iter().any(|path| {
        #[cfg(target_os = "windows")]
        {
            path.to_string_lossy()
                .eq_ignore_ascii_case(&target.to_string_lossy())
        }
        #[cfg(not(target_os = "windows"))]
        {
            path == target
        }
    })
}

/// Watches the parent directory so replace/rename and delete/recreate survive.
/// Dropping the owner stops the loop and releases the native watcher.
pub fn watch_pdf(
    path: &Path,
    debounce: Duration,
    sink: Sender<PdfWatchEvent>,
) -> std::io::Result<PdfWatcher> {
    let parent = path
        .parent()
        .ok_or_else(|| std::io::Error::other("PDF has no parent"))?;
    let parent = if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    };
    let name = path
        .file_name()
        .ok_or_else(|| std::io::Error::other("PDF has no filename"))?;
    let parent = fs::canonicalize(parent)?;
    let target = parent.join(name);
    let (events, source) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |event| {
        events.send(event).ok();
    })
    .map_err(std::io::Error::other)?;
    watcher
        .watch(&parent, RecursiveMode::NonRecursive)
        .map_err(std::io::Error::other)?;
    let (stop, stopped) = mpsc::channel();
    let thread = thread::Builder::new()
        .name("ferrostage-reload".into())
        .spawn(move || {
            let _watcher = watcher;
            let clock = Instant::now();
            let mut state = Stability::default();
            loop {
                if stopped.try_recv().is_ok() {
                    break;
                }
                match source.recv_timeout(POLL) {
                    Ok(Ok(event)) if matches(&event, &target) => {
                        let attempt = state.changed(clock.elapsed());
                        if sink.send(PdfWatchEvent::Invalidated(attempt)).is_err() {
                            break;
                        }
                    }
                    Ok(Err(error)) => tracing::debug!(%error, "PDF watcher event failed"),
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    _ => {}
                }
                let now = clock.elapsed();
                if state.timed_out(now) {
                    tracing::debug!(
                        attempt = state.attempt,
                        "PDF replacement timed out; retaining active document"
                    );
                    if sink.send(PdfWatchEvent::TimedOut(state.attempt)).is_err() {
                        break;
                    }
                } else if state.started.is_some() && state.ready(now, debounce, stamp(&target).ok())
                {
                    if let Some(bytes) =
                        read_stable(&target, state.observed.map(|(stamp, _)| stamp))
                    {
                        state.started = None;
                        state.observed = None;
                        if sink
                            .send(PdfWatchEvent::Ready {
                                attempt: state.attempt,
                                bytes,
                            })
                            .is_err()
                        {
                            break;
                        }
                    } else {
                        state.observed = None;
                    }
                }
            }
        })?;
    Ok(PdfWatcher {
        stop,
        thread: Some(thread),
    })
}

fn read_stable(path: &Path, expected: Option<Stamp>) -> Option<Vec<u8>> {
    let expected = expected?;
    // Incomplete replacements can remain large and stable for seconds. Check
    // only their tail before allocating/reading the full document.
    let mut file = fs::File::open(path).ok()?;
    let tail_len = expected.len.min(1024) as usize;
    file.seek(SeekFrom::End(-(tail_len as i64))).ok()?;
    let mut tail = [0; 1024];
    file.read_exact(&mut tail[..tail_len]).ok()?;
    if !super::has_pdf_eof(&tail[..tail_len]) {
        return None;
    }
    drop(file);
    let bytes = fs::read(path).ok()?;
    (super::has_pdf_eof(&bytes)
        && bytes.len() as u64 == expected.len
        && stamp(path).ok() == Some(expected))
    .then_some(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;
    #[test]
    fn real_parent_watch_handles_chunks_rename_delete_recreate_and_ignores_siblings() {
        let dir = std::env::temp_dir().join(format!(
            "ferrostage-reload-{}-{:?}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&dir).unwrap();
        let path = dir.join("deck.pdf");
        fs::write(&path, b"%PDF-old\n%%EOF").unwrap();
        let (tx, rx) = mpsc::channel();
        let watcher = watch_pdf(&path, Duration::from_millis(30), tx).unwrap();
        fs::write(dir.join("unrelated.pdf"), b"%%EOF").unwrap();
        assert!(rx.recv_timeout(Duration::from_millis(180)).is_err());
        let mut file = fs::File::create(&path).unwrap();
        for _ in 0..3 {
            file.write_all(&vec![b'x'; 65536]).unwrap();
            file.flush().unwrap();
            thread::sleep(Duration::from_millis(50));
        }
        assert!(
            !rx.try_iter()
                .any(|event| matches!(event, PdfWatchEvent::Ready { .. }))
        );
        file.write_all(b"\n%%EOF").unwrap();
        drop(file);
        let ready = |rx: &mpsc::Receiver<PdfWatchEvent>| {
            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                match rx
                    .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                    .unwrap()
                {
                    PdfWatchEvent::Ready { bytes, .. } => break bytes,
                    PdfWatchEvent::TimedOut(_) => panic!("unexpected timeout"),
                    _ => {}
                }
            }
        };
        assert_eq!(ready(&rx).len(), 3 * 65536 + 6);
        let replacement = dir.join("replacement.tmp");
        fs::write(&replacement, b"%PDF-renamed\n%%EOF").unwrap();
        fs::remove_file(&path).unwrap();
        fs::rename(&replacement, &path).unwrap();
        assert_eq!(ready(&rx), b"%PDF-renamed\n%%EOF");
        fs::remove_file(&path).unwrap();
        thread::sleep(Duration::from_millis(150));
        fs::write(&path, b"truncated").unwrap();
        thread::sleep(Duration::from_millis(220));
        assert!(
            !rx.try_iter()
                .any(|event| matches!(event, PdfWatchEvent::Ready { .. }))
        );
        fs::write(&path, b"%PDF-recreated\n%%EOF").unwrap();
        assert_eq!(ready(&rx), b"%PDF-recreated\n%%EOF");
        let previous = stamp(&path).unwrap();
        fs::write(&path, b"different length\n%%EOF").unwrap();
        assert!(read_stable(&path, Some(previous)).is_none());
        let mut early_eof = b"%%EOF".to_vec();
        early_eof.extend(vec![b' '; 2048]);
        fs::write(&path, early_eof).unwrap();
        assert!(read_stable(&path, Some(stamp(&path).unwrap())).is_none());
        fs::write(&path, b"tiny\n%%EOF").unwrap();
        assert_eq!(
            read_stable(&path, Some(stamp(&path).unwrap())).unwrap(),
            b"tiny\n%%EOF"
        );
        drop(watcher);
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn quiet_stable_timeout_and_exact_target() {
        let mut state = Stability::default();
        let value = Some(Stamp {
            len: 20,
            modified: SystemTime::UNIX_EPOCH,
        });
        state.changed(Duration::ZERO);
        assert!(!state.ready(
            Duration::from_millis(299),
            super::super::HOT_RELOAD_DEBOUNCE,
            value
        ));
        assert!(!state.ready(
            Duration::from_millis(300),
            super::super::HOT_RELOAD_DEBOUNCE,
            value
        ));
        assert!(!state.ready(
            Duration::from_millis(399),
            super::super::HOT_RELOAD_DEBOUNCE,
            value
        ));
        assert!(state.ready(
            Duration::from_millis(400),
            super::super::HOT_RELOAD_DEBOUNCE,
            value
        ));
        state.changed(Duration::from_millis(401));
        assert!(!state.ready(
            Duration::from_millis(500),
            super::super::HOT_RELOAD_DEBOUNCE,
            value
        ));
        assert!(!state.timed_out(Duration::from_secs(29)));
        assert!(state.timed_out(Duration::from_secs(30)));
        assert!(!state.ready(Duration::from_secs(31), Duration::ZERO, value));
        let target = PathBuf::from("deck.pdf");
        let event = Event::new(EventKind::Modify(notify::event::ModifyKind::Name(
            notify::event::RenameMode::Both,
        )))
        .add_path(PathBuf::from("sibling.tmp"))
        .add_path(target.clone());
        assert!(matches(&event, &target));
        assert!(!matches(&event, Path::new("other.pdf")));
        #[cfg(target_os = "windows")]
        assert!(matches(&event, Path::new("DECK.PDF")));
    }
}
