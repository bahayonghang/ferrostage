use std::time::{Duration, Instant};

/// Pure timer bookkeeping. Every timestamp comes from the app's monotonic clock.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct TimerState {
    accumulated: Duration,
    running_since: Option<Instant>,
}

impl TimerState {
    pub(crate) fn start(&mut self, now: Instant) {
        self.accumulated = Duration::ZERO;
        self.running_since = Some(now);
    }

    pub(crate) fn elapsed_at(&self, now: Instant) -> Duration {
        self.accumulated
            + self
                .running_since
                .map_or(Duration::ZERO, |start| now.saturating_duration_since(start))
    }

    pub(crate) fn is_paused(&self) -> bool {
        self.running_since.is_none()
    }

    pub(crate) fn pause(&mut self, now: Instant) {
        self.accumulated = self.elapsed_at(now);
        self.running_since = None;
    }

    pub(crate) fn resume(&mut self, now: Instant) {
        self.running_since.get_or_insert(now);
    }

    pub(crate) fn reset(&mut self, now: Instant) {
        self.accumulated = Duration::ZERO;
        if self.running_since.is_some() {
            self.running_since = Some(now);
        }
    }
}
