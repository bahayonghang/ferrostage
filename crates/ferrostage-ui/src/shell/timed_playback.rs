use super::*;
use crate::{
    audience::{PlaybackEvent, PlaybackPhase},
    autoplay::Autoplay,
};
use ferrostage_core::{AudiencePlayback, NormalizedRect, PlaybackIdentity};

#[derive(Default)]
pub(super) struct TimedPlayback {
    serial: u64,
    document: Option<ferrostage_core::DocumentGeneration>,
    navigation: Option<Navigation>,
    pub clock: Autoplay,
}

struct Navigation {
    playback: AudiencePlayback,
    crop: NormalizedRect,
    audience: WindowId,
}

impl Shell {
    pub(super) fn invalidate_playback(&mut self) {
        self.timed.navigation = None;
        self.timed.clock.cancel();
    }
    pub(crate) fn autoplay_paused(&self) -> bool {
        self.timed.clock.paused
    }
    pub(crate) fn has_authored_timing(&self) -> bool {
        self.workbench.document.as_ref().is_some_and(|session| {
            session.document.slide_map.steps().iter().any(|page| {
                session
                    .document
                    .page_presentation(*page)
                    .is_some_and(|p| p.dwell_seconds.is_some())
            })
        })
    }

    pub(super) fn select_playback(&mut self, sequential: bool) {
        let Some(audience) = self
            .audience
            .as_ref()
            .filter(|_| self.state.stage() == SessionStage::Presenting)
        else {
            return;
        };
        let metadata = self
            .workbench
            .document
            .as_ref()
            .and_then(|s| s.document.page_presentation(self.state.current_page()))
            .unwrap_or_default();
        self.timed.serial += 1;
        let identity = PlaybackIdentity {
            generation: self.state.generation(),
            navigation: self.timed.serial,
            page: self.state.current_page(),
        };
        self.timed.navigation = Some(Navigation {
            playback: AudiencePlayback {
                identity,
                transition: if sequential {
                    metadata.transition
                } else {
                    None
                },
            },
            crop: self.state.slide_crop(),
            audience: audience.handle.window_id(),
        });
        let has_next = self
            .state
            .slide_map()
            .and_then(|map| map.next_step(self.state.current_page()))
            .is_some_and(|page| page != self.state.current_page());
        self.timed
            .clock
            .select(identity, metadata.dwell_seconds, has_next);
    }

    pub(super) fn sync_playback(&mut self, curtain: bool, cx: &mut Context<Self>) {
        if self.timed.document != Some(self.state.generation()) {
            self.timed.document = Some(self.state.generation());
            self.timed.clock.pause(false, Instant::now());
        }
        if curtain
            || self.state.stage() != SessionStage::Presenting
            || self.audience.is_none()
            || self.reload_blocks_playback()
            || self.pending.is_some()
            || self.loading
        {
            self.timed.navigation = None;
            self.timed.clock.cancel();
            return;
        }
        let matches = self.timed.navigation.as_ref().is_some_and(|n| {
            n.playback.identity.generation == self.state.generation()
                && n.playback.identity.page == self.state.current_page()
                && n.crop == self.state.slide_crop()
                && self
                    .audience
                    .as_ref()
                    .is_some_and(|a| a.handle.window_id() == n.audience)
        });
        if !matches {
            self.select_playback(false);
        }
        self.suspend_playback(cx);
    }

    fn suspend_playback(&mut self, cx: &mut Context<Self>) {
        let presenter_active = self.presenter.as_ref().is_some_and(|p| {
            cx.active_window() == Some(p.handle)
                && p.handle
                    .update(cx, |_, window, _| window.is_window_active())
                    .unwrap_or(false)
        });
        let frame_ready = self
            .timed
            .navigation
            .as_ref()
            .is_some_and(|n| self.playback_frame_matches(n));
        self.timed.clock.suspend(
            !frame_ready
                || !presenter_active
                || self.modal.is_some()
                || self.contextual.is_some()
                || self.focus_intent.is_some()
                || self.preference_change.is_some()
                || self.state.audience_snapshot().blank_mode.is_some()
                || self.graphics_error.is_some(),
            Instant::now(),
        );
    }

    fn playback_frame_matches(&self, navigation: &Navigation) -> bool {
        let identity = navigation.playback.identity;
        self.workbench
            .frame(crate::workbench::Surface::Audience)
            .is_some_and(|frame| {
                frame.request.generation == identity.generation
                    && frame.request.page == identity.page
                    && frame.request.crop == navigation.crop
                    && Some(frame.request.target_px) == self.workbench.audience_target
                    && frame.request.priority == ferrostage_render::RenderPriority::Audience
            })
    }

    pub(super) fn attach_playback(&self, snapshot: &mut AudienceProjectionSnapshot) {
        let Some(navigation) = &self.timed.navigation else {
            snapshot.playback = None;
            return;
        };
        let identity = navigation.playback.identity;
        let exact = self.playback_frame_matches(navigation);
        if snapshot.stage == AudienceStage::Live
            && snapshot.visual.is_some()
            && snapshot.generation == identity.generation
            && snapshot.current_page == identity.page
            && snapshot.slide_crop == navigation.crop
            && exact
        {
            snapshot.playback = Some(navigation.playback);
        }
        // Otherwise assemble's retained pixels retain their previous identity.
        // The new dwell has already been invalidated and rejects old events.
    }

    pub(super) fn playback_event(
        &mut self,
        source: WindowId,
        event: PlaybackEvent,
        cx: &mut Context<Self>,
    ) {
        if self.pending.is_some()
            || self.reload_blocks_playback()
            || self.state.stage() != SessionStage::Presenting
            || self
                .audience
                .as_ref()
                .is_none_or(|a| a.handle.window_id() != source)
            || self
                .timed
                .navigation
                .as_ref()
                .is_none_or(|n| n.audience != source || n.playback.identity != event.identity)
            || self.snapshot.read(cx).playback.map(|p| p.identity) != Some(event.identity)
        {
            return;
        }
        self.suspend_playback(cx);
        match event.phase {
            PlaybackPhase::FirstVisible => self.timed.clock.first_visible(event.identity, event.at),
            PlaybackPhase::Settled => self.timed.clock.settled(event.identity, event.at),
        }
        tracing::debug!(?event.identity, ?event.phase, "audience playback acknowledged");
    }

    pub(super) fn poll_timed_playback(&mut self, cx: &mut Context<Self>) {
        self.sync_playback(self.pending.is_some(), cx);
        if let Some(identity) = self.timed.clock.next_turn(Instant::now())
            && self
                .timed
                .navigation
                .as_ref()
                .is_some_and(|n| n.playback.identity == identity)
            && self.snapshot.read(cx).playback.map(|p| p.identity) == Some(identity)
        {
            self.apply(Command::NextStep, cx);
        }
    }

    pub(super) fn pause_for_annotation(&mut self) {
        self.timed.clock.pause(true, Instant::now());
        if let Some(navigation) = &mut self.timed.navigation {
            navigation.playback.transition = None;
        }
    }
}
