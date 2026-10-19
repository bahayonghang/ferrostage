//! A single paint-gated page dwell, independent of the presentation talk timer.
use ferrostage_core::PlaybackIdentity;
use std::time::{Duration, Instant};

#[derive(Default)]
pub(crate) struct Autoplay {
    identity: Option<PlaybackIdentity>,
    remaining: Option<Duration>,
    since: Option<Instant>,
    first_visible: bool,
    settled: bool,
    pub paused: bool,
    suspended: bool,
    turn: u64,
    eligible_turn: u64,
    fired: bool,
}

impl Autoplay {
    pub fn select(&mut self, identity: PlaybackIdentity, dwell: Option<f64>, has_next: bool) {
        self.identity = Some(identity);
        self.remaining = dwell
            .filter(|_| has_next)
            .and_then(|value| Duration::try_from_secs_f64(value).ok());
        self.since = None;
        self.first_visible = false;
        self.settled = false;
        self.fired = false;
    }

    pub fn cancel(&mut self) {
        self.identity = None;
        self.remaining = None;
        self.since = None;
        self.first_visible = false;
        self.settled = false;
    }

    fn freeze(&mut self, now: Instant) {
        if let (Some(since), Some(remaining)) = (self.since.take(), &mut self.remaining) {
            *remaining = remaining.saturating_sub(now.saturating_duration_since(since));
        }
    }

    fn resume(&mut self, now: Instant) {
        if !self.paused
            && !self.suspended
            && self.first_visible
            && self.settled
            && self.remaining.is_some()
            && !self.fired
        {
            self.since.get_or_insert(now);
        }
    }

    pub fn pause(&mut self, paused: bool, now: Instant) {
        if paused != self.paused {
            self.freeze(now);
            self.paused = paused;
            self.resume(now);
            self.eligible_turn = self.turn + 1;
        }
    }

    pub fn suspend(&mut self, suspended: bool, now: Instant) {
        if suspended != self.suspended {
            self.freeze(now);
            self.suspended = suspended;
            self.resume(now);
            self.eligible_turn = self.turn + 1;
        }
    }

    pub fn first_visible(&mut self, identity: PlaybackIdentity, at: Instant) {
        if self.identity == Some(identity) && !self.first_visible {
            self.first_visible = true;
            self.resume(at);
            self.eligible_turn = self.turn + 1;
        }
    }

    pub fn settled(&mut self, identity: PlaybackIdentity, at: Instant) {
        if self.identity == Some(identity) && !self.settled {
            self.settled = true;
            self.resume(at);
            self.eligible_turn = self.turn + 1;
        }
    }

    /// Called once from the shell poller, never recursively from a paint event.
    pub fn next_turn(&mut self, now: Instant) -> Option<PlaybackIdentity> {
        self.turn += 1;
        if self.paused
            || self.suspended
            || !self.first_visible
            || !self.settled
            || self.fired
            || self.turn < self.eligible_turn
        {
            return None;
        }
        let elapsed = now.saturating_duration_since(self.since?);
        if elapsed < self.remaining? {
            return None;
        }
        self.fired = true;
        self.since = None;
        self.identity
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn id(navigation: u64) -> PlaybackIdentity {
        PlaybackIdentity {
            generation: ferrostage_core::DocumentGeneration(1),
            navigation,
            page: ferrostage_core::PhysicalPage(navigation as usize),
        }
    }

    #[test]
    fn dwell_starts_after_transition_settlement() {
        let t = Instant::now();
        let mut clock = Autoplay::default();
        clock.select(id(1), Some(2.), true);
        assert_eq!(clock.next_turn(t + Duration::from_secs(30)), None);
        clock.first_visible(id(1), t + Duration::from_secs(30));
        assert_eq!(clock.next_turn(t + Duration::from_secs(32)), None);
        clock.settled(id(1), t + Duration::from_secs(33));
        assert_eq!(clock.next_turn(t + Duration::from_secs(34)), None);
        assert_eq!(clock.next_turn(t + Duration::from_secs(35)), Some(id(1)));
        assert_eq!(clock.next_turn(t + Duration::from_secs(100)), None);
    }

    #[test]
    fn pause_during_transition_preserves_the_full_post_transition_dwell() {
        let t = Instant::now();
        let mut clock = Autoplay::default();
        clock.select(id(1), Some(2.), true);
        clock.first_visible(id(1), t);
        clock.pause(true, t + Duration::from_secs(1));
        clock.settled(id(1), t + Duration::from_secs(3));
        clock.pause(false, t + Duration::from_secs(10));
        assert_eq!(clock.next_turn(t + Duration::from_secs(11)), None);
        assert_eq!(clock.next_turn(t + Duration::from_secs(12)), Some(id(1)));
    }

    #[test]
    fn zero_waits_for_paint_settle_and_a_later_turn_with_no_backlog() {
        let t = Instant::now();
        let mut clock = Autoplay::default();
        clock.select(id(1), Some(0.), true);
        clock.settled(id(1), t);
        assert_eq!(clock.next_turn(t), None);
        clock.first_visible(id(1), t);
        assert_eq!(clock.next_turn(t), Some(id(1)));
        clock.select(id(2), Some(0.), true);
        clock.first_visible(id(1), t);
        clock.settled(id(1), t);
        assert_eq!(clock.next_turn(t + Duration::from_secs(100)), None);
        clock.first_visible(id(2), t);
        clock.settled(id(2), t);
        assert_eq!(clock.next_turn(t), Some(id(2)));
    }

    #[test]
    fn explicit_pause_and_temporary_suspension_preserve_remaining_dwell() {
        let t = Instant::now();
        let mut clock = Autoplay::default();
        clock.select(id(1), Some(10.), true);
        clock.first_visible(id(1), t);
        clock.settled(id(1), t);
        clock.pause(true, t + Duration::from_secs(3));
        clock.suspend(true, t + Duration::from_secs(20));
        clock.pause(false, t + Duration::from_secs(30));
        assert_eq!(clock.next_turn(t + Duration::from_secs(100)), None);
        clock.suspend(false, t + Duration::from_secs(100));
        assert_eq!(clock.next_turn(t + Duration::from_secs(106)), None);
        assert_eq!(clock.next_turn(t + Duration::from_secs(107)), Some(id(1)));
    }

    #[test]
    fn navigation_preserves_explicit_pause_and_invalidates_old_events() {
        let t = Instant::now();
        let mut clock = Autoplay::default();
        clock.select(id(1), Some(0.), true);
        clock.pause(true, t);
        clock.select(id(2), Some(0.), true);
        clock.first_visible(id(2), t);
        clock.settled(id(2), t);
        assert_eq!(clock.next_turn(t), None);
        clock.pause(false, t);
        assert_eq!(clock.next_turn(t), Some(id(2)));
        clock.cancel();
        clock.first_visible(id(2), t);
        clock.settled(id(2), t);
        assert_eq!(clock.next_turn(t), None);
    }

    #[test]
    fn missing_invalid_and_final_page_durations_never_advance() {
        let t = Instant::now();
        for (duration, has_next) in [
            (None, true),
            (Some(-1.), true),
            (Some(f64::INFINITY), true),
            (Some(f64::MAX), true),
            (Some(0.), false),
        ] {
            let mut clock = Autoplay::default();
            clock.select(id(1), duration, has_next);
            clock.first_visible(id(1), t);
            clock.settled(id(1), t);
            assert_eq!(clock.next_turn(t + Duration::from_secs(100)), None);
        }
    }
}
