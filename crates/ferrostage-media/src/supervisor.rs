//! Latest-value commands and frames. No UI wait/join or unbounded frame queue.
use crate::{
    Identity, MediaError, SharedFrame, animation,
    protocol::{self, Packet},
};
use ferrostage_core::NormalizedRect;
use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
mod child;
#[derive(Clone, Debug)]
pub struct Runtime {
    pub helper: PathBuf,
    pub directory: PathBuf,
    pub silent: bool,
}
impl Default for Runtime {
    fn default() -> Self {
        let directory = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(PathBuf::from))
            .unwrap_or_default();
        Self {
            helper: directory.join(if cfg!(windows) {
                "ferrostage-media-helper.exe"
            } else {
                "ferrostage-media-helper"
            }),
            directory: directory.join("media-runtime"),
            silent: false,
        }
    }
}
#[derive(Clone, Debug)]
pub enum Source {
    External(PathBuf),
    Embedded { name: String, bytes: Arc<[u8]> },
}
#[derive(Clone, Debug)]
pub struct Start {
    pub identity: Identity,
    pub source: Source,
    pub max_height: u32,
    pub source_crop: NormalizedRect,
    pub aspect_ratio: Option<f32>,
    pub start_ms: u64,
    pub duration_ms: Option<u64>,
    pub plays: Option<u32>,
}
#[derive(Clone, Debug)]
pub struct Update {
    pub identity: Identity,
    pub frame: Option<SharedFrame>,
    pub position_ms: u64,
    pub duration_ms: u64,
    pub paused: bool,
    pub ended: bool,
    pub error: Option<MediaError>,
}
impl Update {
    fn new(identity: Identity) -> Self {
        Self {
            identity,
            frame: None,
            position_ms: 0,
            duration_ms: 0,
            paused: false,
            ended: false,
            error: None,
        }
    }
}
#[derive(Clone, Default)]
struct Desired {
    start: Option<Start>,
    paused: bool,
}
struct Shared {
    desired: Mutex<Desired>,
    latest: Mutex<Option<Update>>,
    alive: AtomicBool,
    exited: AtomicBool,
}
static NEXT_EPOCH: AtomicU64 = AtomicU64::new(1);
pub struct Controller {
    shared: Arc<Shared>,
    identity: Option<Identity>,
}
impl Controller {
    pub fn new(runtime: Runtime) -> Result<Self, MediaError> {
        let shared = Arc::new(Shared {
            desired: Mutex::new(Desired::default()),
            latest: Mutex::new(None),
            alive: AtomicBool::new(true),
            exited: AtomicBool::new(false),
        });
        let worker = shared.clone();
        thread::Builder::new()
            .name("media-supervisor".into())
            .spawn(move || run(worker, runtime))
            .map_err(|_| MediaError::Process)?;
        Ok(Self {
            shared,
            identity: None,
        })
    }
    pub fn start(&mut self, mut start: Start) -> Result<Identity, MediaError> {
        start.identity.epoch = NEXT_EPOCH
            .try_update(Ordering::AcqRel, Ordering::Acquire, |n| n.checked_add(1))
            .map_err(|_| MediaError::Limit)?;
        self.identity = Some(start.identity);
        if let Ok(mut desired) = self.shared.desired.lock() {
            *desired = Desired {
                start: Some(start.clone()),
                paused: false,
            };
        }
        Ok(start.identity)
    }
    pub fn stop(&mut self) {
        self.identity = None;
        if let Ok(mut d) = self.shared.desired.lock() {
            *d = Desired::default();
        }
    }
    pub fn pause(&self, paused: bool) {
        if let Ok(mut d) = self.shared.desired.lock() {
            d.paused = paused;
        }
    }
    pub fn latest(&self) -> Option<Update> {
        self.shared
            .latest
            .lock()
            .ok()?
            .as_ref()
            .filter(|u| Some(u.identity) == self.identity)
            .cloned()
    }
    pub fn identity(&self) -> Option<Identity> {
        self.identity
    }
    pub fn shutdown(&mut self) {
        self.stop();
        self.shared.alive.store(false, Ordering::Release);
    }
    pub fn has_exited(&self) -> bool {
        self.shared.exited.load(Ordering::Acquire)
    }
}
impl Drop for Controller {
    fn drop(&mut self) {
        self.shutdown();
    }
}
struct AnimationPlayback {
    animation: animation::Animation,
    elapsed: Duration,
    last: Instant,
    sequence: u64,
    last_index: Option<usize>,
}
enum Decoder {
    Animation(AnimationPlayback),
    Native(child::Native),
}
struct Active {
    start: Start,
    decoder: Decoder,
    _source: child::PreparedSource,
    update: Update,
    paused: bool,
}
impl Active {
    fn new(
        start: Start,
        runtime: &Runtime,
        cancelled: impl Fn() -> bool,
    ) -> Result<Self, MediaError> {
        if cancelled() {
            return Err(MediaError::Cancelled);
        }
        let source = child::PreparedSource::new(&start.source)?;
        let animation_format = std::fs::File::open(&source.path)
            .ok()
            .and_then(|f| {
                image::ImageReader::new(std::io::BufReader::new(f))
                    .with_guessed_format()
                    .ok()
            })
            .and_then(|r| r.format());
        let decoder = if matches!(
            animation_format,
            Some(image::ImageFormat::Gif | image::ImageFormat::WebP)
        ) {
            Decoder::Animation(AnimationPlayback {
                animation: animation::decode_projected(
                    &source.path,
                    start.max_height,
                    start.source_crop,
                    start.aspect_ratio,
                    cancelled,
                )?,
                elapsed: Duration::ZERO,
                last: Instant::now(),
                sequence: 0,
                last_index: None,
            })
        } else {
            Decoder::Native(child::Native::new(
                runtime,
                protocol::Open {
                    path: source
                        .path
                        .to_str()
                        .ok_or(MediaError::SourceUnavailable)?
                        .into(),
                    max_height: start.max_height,
                    start_ms: start.start_ms,
                    duration_ms: start.duration_ms,
                    plays: start.plays.unwrap_or(1),
                    silent: runtime.silent,
                },
            )?)
        };
        Ok(Self {
            update: Update::new(start.identity),
            start,
            decoder,
            _source: source,
            paused: false,
        })
    }
    fn poll(&mut self, paused: bool) -> Result<(), MediaError> {
        match &mut self.decoder {
            Decoder::Native(native) => {
                if paused != self.paused {
                    native.command(Packet::Pause(paused))?;
                }
                let output = native.poll()?;
                if let Some((sequence, position, duration, pixels)) = output.frame {
                    self.update.frame = Some(SharedFrame {
                        identity: self.start.identity,
                        sequence,
                        pixels: animation::project_frame(
                            &pixels,
                            self.start.source_crop,
                            self.start.aspect_ratio,
                            self.start.max_height,
                        )?,
                    });
                    self.update.position_ms = position;
                    self.update.duration_ms = duration;
                }
                if let Some((position, duration)) = output.progress {
                    self.update.position_ms = position;
                    self.update.duration_ms = duration;
                }
                self.update.ended = output.ended;
            }
            Decoder::Animation(a) => {
                let now = Instant::now();
                if !self.paused && !self.update.ended {
                    a.elapsed += now.duration_since(a.last);
                }
                a.last = now;
                let start = Duration::from_millis(self.start.start_ms);
                let available = a.animation.duration.saturating_sub(start);
                let length = self
                    .start
                    .duration_ms
                    .map(Duration::from_millis)
                    .unwrap_or(available)
                    .min(available);
                if length.is_zero() {
                    return Err(MediaError::Unsupported);
                }
                let total = a.elapsed.as_nanos() / length.as_nanos();
                self.update.ended = self.start.plays.unwrap_or(a.animation.plays) != 0
                    && total >= u128::from(self.start.plays.unwrap_or(a.animation.plays));
                let position = if self.update.ended {
                    length.saturating_sub(Duration::from_nanos(1))
                } else {
                    Duration::from_nanos((a.elapsed.as_nanos() % length.as_nanos()) as u64)
                };
                self.update.position_ms = position.as_millis() as u64;
                self.update.duration_ms = length.as_millis() as u64;
                let mut until = Duration::ZERO;
                let index = a
                    .animation
                    .frames
                    .iter()
                    .position(|(_, delay)| {
                        until += *delay;
                        until > start + position
                    })
                    .unwrap_or(a.animation.frames.len() - 1);
                if a.last_index != Some(index) {
                    a.sequence += 1;
                    a.last_index = Some(index);
                    self.update.frame = Some(SharedFrame {
                        identity: self.start.identity,
                        sequence: a.sequence,
                        pixels: a.animation.frames[index].0.clone(),
                    });
                }
            }
        }
        self.paused = paused;
        self.update.paused = paused;
        Ok(())
    }
}
fn run(shared: Arc<Shared>, runtime: Runtime) {
    let mut active: Option<Active> = None;
    let mut attempted = None;
    while shared.alive.load(Ordering::Acquire) {
        let desired = match shared.desired.lock() {
            Ok(d) => d.clone(),
            Err(_) => break,
        };
        let identity = desired.start.as_ref().map(|s| s.identity);
        if active.as_ref().map(|a| a.start.identity) != identity {
            // Native Drop kills/reaps before another decoder is constructed. This
            // wait is exclusively on the supervisor, never the GUI thread.
            drop(active.take());
            if let Some(start) = desired.start
                && attempted != Some(start.identity)
            {
                attempted = Some(start.identity);
                let id = start.identity;
                let cancelled = || {
                    !shared.alive.load(Ordering::Acquire)
                        || shared
                            .desired
                            .lock()
                            .map_or(true, |d| d.start.as_ref().map(|s| s.identity) != Some(id))
                };
                match Active::new(start, &runtime, cancelled) {
                    Ok(a) => active = Some(a),
                    Err(error) => {
                        let mut u = Update::new(id);
                        u.error = Some(error);
                        if let Ok(mut latest) = shared.latest.lock() {
                            *latest = Some(u);
                        }
                    }
                }
            }
        }
        if let Some(a) = &mut active {
            if let Err(error) = a.poll(desired.paused) {
                a.update.error = Some(error);
            }
            if let Ok(mut latest) = shared.latest.lock() {
                *latest = Some(a.update.clone());
            }
            if a.update.error.is_some() {
                drop(active.take());
            }
        }
        thread::sleep(Duration::from_millis(5));
    }
    drop(active);
    shared.exited.store(true, Ordering::Release);
}
