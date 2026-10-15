use std::{sync::mpsc::Sender, thread::JoinHandle, time::Duration};
#[cfg(any(target_os = "windows", test))]
use std::{
    sync::mpsc::{self, Receiver},
    thread,
    time::Instant,
};

#[cfg(target_os = "windows")]
use crate::enumerate_monitors;
use crate::{MonitorInfo, PlatformError};

/// Explicit fallback: no hidden window or WM_DISPLAYCHANGE subscription.
pub const TOPOLOGY_POLL_INTERVAL: Duration = Duration::from_millis(200);

#[derive(Clone, Debug, PartialEq)]
pub enum TopologyEvent {
    Changed(Vec<MonitorInfo>),
    /// Last known geometry is diagnostic only. Audience availability must be
    /// false immediately, even when this list still contains its saved id.
    Unavailable {
        error: PlatformError,
        last_known: Vec<MonitorInfo>,
    },
}

/// Owns one polling worker. Dropping or stopping cancels its wait and joins it.
/// A synchronous OS enumeration already in progress cannot be interrupted.
pub struct TopologyWatcher {
    cancel: Sender<()>,
    worker: Option<JoinHandle<()>>,
}

impl TopologyWatcher {
    pub fn stop(mut self) -> Result<(), PlatformError> {
        self.shutdown()
    }

    fn shutdown(&mut self) -> Result<(), PlatformError> {
        if let Some(worker) = self.worker.take() {
            // A disconnected receiver means the worker already stopped.
            if self.cancel.send(()).is_err() {
                tracing::debug!("topology watcher already stopped");
            }
            worker.join().map_err(|_| PlatformError::WatcherPanicked)?;
        }
        Ok(())
    }
}

impl Drop for TopologyWatcher {
    fn drop(&mut self) {
        if let Err(error) = self.shutdown() {
            tracing::error!(%error, "stopping topology watcher");
        }
    }
}

pub fn watch_topology(sink: Sender<TopologyEvent>) -> Result<TopologyWatcher, PlatformError> {
    #[cfg(target_os = "windows")]
    {
        start(sink, enumerate_monitors)
    }
    #[cfg(not(target_os = "windows"))]
    {
        drop(sink);
        Err(PlatformError::UnsupportedPlatform)
    }
}

#[cfg(any(target_os = "windows", test))]
fn start(
    sink: Sender<TopologyEvent>,
    mut query: impl FnMut() -> Result<Vec<MonitorInfo>, PlatformError> + Send + 'static,
) -> Result<TopologyWatcher, PlatformError> {
    let (cancel, cancelled) = mpsc::channel();
    let worker = thread::Builder::new()
        .name("monitor-topology".into())
        .spawn(move || {
            let mut state = Observations::default();
            loop {
                if cancelled.try_recv().is_ok() {
                    break;
                }
                if let Some(event) = state.observe(query(), Instant::now())
                    && sink.send(event).is_err()
                {
                    break;
                }
                if stop_requested(&cancelled) {
                    break;
                }
            }
        })
        .map_err(|error| PlatformError::WatcherStart {
            message: error.to_string(),
        })?;
    Ok(TopologyWatcher {
        cancel,
        worker: Some(worker),
    })
}

#[cfg(any(target_os = "windows", test))]
fn stop_requested(cancelled: &Receiver<()>) -> bool {
    !matches!(
        cancelled.recv_timeout(TOPOLOGY_POLL_INTERVAL),
        Err(mpsc::RecvTimeoutError::Timeout)
    )
}

#[cfg(any(target_os = "windows", test))]
#[derive(Default)]
struct Observations {
    current: Option<Vec<MonitorInfo>>,
    pending: Option<(Vec<MonitorInfo>, Instant)>,
    error: Option<PlatformError>,
}

#[cfg(any(target_os = "windows", test))]
impl Observations {
    fn observe(
        &mut self,
        result: Result<Vec<MonitorInfo>, PlatformError>,
        now: Instant,
    ) -> Option<TopologyEvent> {
        let mut next = match result {
            Err(error) => {
                self.pending = None;
                if self.error.as_ref() == Some(&error) {
                    return None;
                }
                self.error = Some(error.clone());
                return Some(TopologyEvent::Unavailable {
                    error,
                    last_known: self.current.clone().unwrap_or_default(),
                });
            }
            Ok(next) => next,
        };
        next.sort_by(|a, b| {
            a.id.cmp(&b.id)
                .then(a.connection_name.cmp(&b.connection_name))
        });
        if self.error.is_none() && self.current.as_ref() == Some(&next) {
            self.pending = None;
            return None;
        }
        if self.current.is_none() && self.error.is_none() {
            return Some(self.publish(next));
        }
        match &self.pending {
            Some((candidate, since)) if candidate == &next => {
                if now.saturating_duration_since(*since) >= TOPOLOGY_POLL_INTERVAL {
                    Some(self.publish(next))
                } else {
                    None
                }
            }
            _ => {
                self.pending = Some((next, now));
                None
            }
        }
    }

    fn publish(&mut self, next: Vec<MonitorInfo>) -> TopologyEvent {
        self.current = Some(next.clone());
        self.error = None;
        self.pending = None;
        TopologyEvent::Changed(next)
    }
}

#[cfg(test)]
mod tests;
