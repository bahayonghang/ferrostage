use super::{Audience, PlaybackEvent, PlaybackPhase, Surface, report};
use crate::{
    texture_cache::TextureCache,
    transitions::{self, Layer, Rect, Source},
};
use ferrostage_core::{
    AudienceProjectionSnapshot, AudienceStage, BgraFrame, PageTransition, PlaybackIdentity,
    TransitionStyle,
};
use gpui::{
    AnyElement, App, Bounds, Pixels, WeakEntity, Window, canvas, div, point, prelude::*, size,
};
use std::{cell::RefCell, rc::Rc, time::Instant};
#[cfg(all(test, feature = "test-support"))]
mod tests;

#[derive(Default)]
pub(super) struct Playback {
    identity: Option<PlaybackIdentity>,
    effect: Option<PageTransition>,
    started: Option<Instant>,
    first_visible: bool,
    settled: bool,
    ready: bool,
    difference: Option<BgraFrame>,
    error: Option<String>,
}
impl Playback {
    pub(super) fn error(&self) -> Option<String> {
        self.error.clone()
    }
    pub(super) fn active(&self) -> bool {
        self.effect.is_some() && !self.settled
    }
    pub(super) fn sync(
        &mut self,
        snapshot: &AudienceProjectionSnapshot,
        cache: &mut TextureCache<Surface>,
        frames: &mut Vec<(Surface, BgraFrame)>,
        cx: &mut App,
    ) {
        let playback = snapshot.playback.filter(|p| {
            p.identity.generation == snapshot.generation && p.identity.page == snapshot.current_page
        });
        let identity = playback.map(|p| p.identity);
        let visible = snapshot.stage == AudienceStage::Live && snapshot.blank_mode.is_none();
        let plain = !visible
            || snapshot.staging.is_some()
            || snapshot.laser_pos_norm.is_some()
            || !snapshot.strokes.is_empty()
            || snapshot
                .visual
                .as_ref()
                .is_some_and(|v| !v.media.is_empty());
        if identity != self.identity {
            let previous = self.identity;
            let effect = playback.and_then(|p| p.transition).filter(|t| {
                t.style != TransitionStyle::Replace
                    && t.duration_seconds > 0.
                    && t.duration_seconds.is_finite()
            });
            let can_animate = !plain
                && previous
                    .zip(identity)
                    .is_some_and(|(a, b)| a.generation == b.generation && a.page != b.page)
                && cache.frame(Surface::Page).is_some();
            cache.discard(Surface::Outgoing, cx);
            cache.discard(Surface::Difference, cx);
            *self = Self {
                identity,
                effect: effect.filter(|_| can_animate),
                ..Default::default()
            };
            if self.effect.is_some() {
                cache.rename(Surface::Page, Surface::Outgoing, cx);
            }
        }
        if plain || playback.is_some_and(|p| p.transition.is_none()) {
            self.effect = None;
            self.difference = None;
        }
        if self.active() {
            let old = cache.frame(Surface::Outgoing);
            let new = snapshot.visual.as_ref().map(|v| &v.page);
            if let (Some(old), Some(new), Some(effect)) = (old, new, self.effect) {
                let copies = if effect.style == TransitionStyle::Fly {
                    3
                } else {
                    2
                };
                let required = old
                    .allocation_bytes()
                    .saturating_add(new.allocation_bytes().saturating_mul(copies - 1));
                if required
                    > snapshot
                        .visual
                        .as_ref()
                        .map_or(0, |v| v.texture_budget_bytes / 2)
                {
                    self.error = Some(
                        "Transition images exceed GPU budget; displaying destination immediately"
                            .into(),
                    );
                    self.effect = None;
                } else {
                    if effect.style == TransitionStyle::Fly && self.difference.is_none() {
                        self.difference = transitions::difference(&old, new, effect);
                        if self.difference.is_none() {
                            self.error=Some("Fly requires matching raster geometry; displaying destination immediately".into());
                            self.effect = None;
                        }
                    }
                    if self.effect.is_some() {
                        frames.push((Surface::Outgoing, old));
                        if let Some(diff) = &self.difference {
                            frames.push((Surface::Difference, diff.clone()));
                        }
                    }
                }
            } else {
                self.effect = None;
            }
        }
        if !self.active() {
            if cache.frame(Surface::Page).is_none() {
                cache.rename(Surface::Outgoing, Surface::Page, cx);
            }
            cache.discard(Surface::Outgoing, cx);
            cache.discard(Surface::Difference, cx);
            self.difference = None;
        }
    }
    pub(super) fn painted(
        &mut self,
        snapshot: &AudienceProjectionSnapshot,
        settled: bool,
        view: WeakEntity<Audience>,
        cx: &mut App,
    ) {
        let Some(identity) = self.identity else {
            return;
        };
        if snapshot.playback.map(|p| p.identity) != Some(identity) {
            return;
        }
        let at = Instant::now();
        let mut events = Vec::with_capacity(2);
        if !self.first_visible {
            events.push(PlaybackPhase::FirstVisible);
        }
        if settled && !self.settled {
            events.push(PlaybackPhase::Settled);
        }
        for phase in events {
            let view = view.clone();
            cx.defer(move |cx| {
                let _ = view.update(cx, |audience, cx| {
                    // A deferred result can outlive another snapshot publication.
                    let current = audience.snapshot.read(cx);
                    if current.stage == AudienceStage::Live
                        && current.blank_mode.is_none()
                        && current.playback.map(|p| p.identity) == Some(identity)
                    {
                        // A blank can retire this queued delivery while keeping
                        // the navigation identity. Mark only delivered phases,
                        // so the first unblanked paint can acknowledge again.
                        let mut state = audience.playback.borrow_mut();
                        if state.identity != Some(identity) {
                            return;
                        }
                        let delivered = match phase {
                            PlaybackPhase::FirstVisible => &mut state.first_visible,
                            PlaybackPhase::Settled => &mut state.settled,
                        };
                        if *delivered {
                            return;
                        }
                        *delivered = true;
                        drop(state);
                        cx.emit(PlaybackEvent {
                            identity,
                            phase,
                            at,
                        });
                    }
                });
            });
        }
    }
}

fn bounds(rect: Rect, fitted: Bounds<Pixels>) -> Bounds<Pixels> {
    Bounds::new(
        point(
            fitted.origin.x + fitted.size.width * rect.x,
            fitted.origin.y + fitted.size.height * rect.y,
        ),
        size(fitted.size.width * rect.w, fitted.size.height * rect.h),
    )
}
fn draw(
    layer: Layer,
    cache: &TextureCache<Surface>,
    viewport: Bounds<Pixels>,
    window: &mut Window,
) -> Result<(), String> {
    let key = match layer.source {
        Source::Old => Surface::Outgoing,
        Source::New => Surface::Page,
        Source::Difference => Surface::Difference,
    };
    let Some((frame, image)) = cache.image(key) else {
        return Err("Transition texture unavailable".into());
    };
    let fitted = crate::slide_surface::fit(viewport, frame.width(), frame.height());
    let image_bounds = bounds(layer.image, fitted);
    if ![
        image_bounds.origin.x,
        image_bounds.origin.y,
        image_bounds.size.width,
        image_bounds.size.height,
    ]
    .iter()
    .all(|n| f32::from(*n).is_finite())
    {
        return Err("Transition geometry exceeds drawable bounds".into());
    }
    window.with_content_mask(Some(gpui::ContentMask { bounds: fitted }), |window| {
        window
            .paint_image(
                bounds(layer.clip, fitted),
                image_bounds,
                Default::default(),
                image,
                0,
                false,
            )
            .map_err(|e| e.to_string())
    })
}

pub(super) fn element(
    state: Rc<RefCell<Playback>>,
    cache: Rc<RefCell<TextureCache<Surface>>>,
    snapshot: AudienceProjectionSnapshot,
    view: WeakEntity<Audience>,
    reported: Rc<RefCell<Option<String>>>,
) -> AnyElement {
    let effect = state.borrow().effect.expect("active playback");
    let progress = state.borrow().started.map_or(0., |at| {
        (at.elapsed().as_secs_f64() / effect.duration_seconds).min(1.) as f32
    });
    let planned = transitions::layers(effect, progress);
    let (s, c, snap) = (state.clone(), cache.clone(), snapshot.clone());
    let mut root = div().size_full().relative().child(
        canvas(
            |_, _, _| (),
            move |_, _, window, cx| {
                let mut state = s.borrow_mut();
                if state.ready {
                    return;
                }
                let mut cache = c.borrow_mut();
                let uploaded = cache.paint(
                    Surface::Page,
                    Some(&snap),
                    window,
                    cx,
                    |frame, image, _, window| crate::slide_surface::preupload(frame, image, window),
                );
                let diff = state.difference.is_none()
                    || cache.paint(
                        Surface::Difference,
                        None,
                        window,
                        cx,
                        |frame, image, _, window| {
                            crate::slide_surface::preupload(frame, image, window)
                        },
                    );
                if uploaded && diff {
                    state.ready = true;
                    state.started = Some(Instant::now());
                } else {
                    state.error = cache.error().or_else(|| {
                        Some("Transition upload failed; displaying destination immediately".into())
                    });
                    state.effect = None;
                }
            },
        )
        .absolute()
        .size_full(),
    );
    for layer in planned {
        let (s, c) = (state.clone(), cache.clone());
        root = root.child(
            div().absolute().size_full().opacity(layer.opacity).child(
                canvas(
                    |_, _, _| (),
                    move |viewport, _, window, _| {
                        if !s.borrow().ready {
                            return;
                        }
                        if let Err(error) = draw(layer, &c.borrow(), viewport, window) {
                            s.borrow_mut().error = Some(error);
                            s.borrow_mut().effect = None;
                        }
                    },
                )
                .size_full(),
            ),
        );
    }
    root.child(
        canvas(
            |_, _, _| (),
            move |viewport, _, window, cx| {
                let mut state = state.borrow_mut();
                if state.effect.is_none() {
                    let mut cache = cache.borrow_mut();
                    let fallback = cache.frame(Surface::Page).is_none();
                    if fallback {
                        cache.rename(Surface::Outgoing, Surface::Page, cx);
                    }
                    let painted = if fallback {
                        if let Some((frame, image)) = cache.image(Surface::Page) {
                            let _ =
                                crate::slide_surface::paint(viewport, &frame, image, None, window);
                        }
                        false
                    } else {
                        cache.paint(
                            Surface::Page,
                            Some(&snapshot),
                            window,
                            cx,
                            |frame, image, overlay, window| {
                                crate::slide_surface::paint(viewport, frame, image, overlay, window)
                            },
                        )
                    };
                    if painted {
                        state.painted(&snapshot, true, view.clone(), cx);
                    }
                } else if state.ready {
                    state.painted(&snapshot, progress >= 1., view.clone(), cx);
                }
                report(
                    state.error.clone().or_else(|| cache.borrow().error()),
                    &reported,
                    view.clone(),
                    cx,
                );
                // The final frame requests one cleanup draw; idle playback schedules nothing.
                let identity = state.identity;
                let view = view.clone();
                window.on_next_frame(move |_, cx| {
                    let _ = view.update(cx, |audience, cx| {
                        if audience.playback.borrow().identity == identity {
                            cx.notify();
                        }
                    });
                });
            },
        )
        .absolute()
        .size_full(),
    )
    .into_any_element()
}
