//! Projection input only. No controller, path, settings, worker or document.
use crate::texture_cache::TextureCache;
use ferrostage_core::{AudienceProjectionSnapshot, AudienceStage, BlankMode};
use gpui::{
    App, Context, Entity, EventEmitter, Render, WeakEntity, Window, canvas, div, prelude::*, rgb,
};
use std::{cell::RefCell, rc::Rc};

pub(crate) struct RenderStatus(pub Option<String>);
pub(crate) struct StagingStatus(pub u64, pub Result<(), String>);
#[derive(Clone, Copy, PartialEq)]
enum Surface {
    Page,
    Media(u64),
}
pub(crate) struct Audience {
    pub snapshot: Entity<AudienceProjectionSnapshot>,
    textures: Rc<RefCell<TextureCache<Surface>>>,
    reported_error: Rc<RefCell<Option<String>>>,
}
impl EventEmitter<RenderStatus> for Audience {}
impl EventEmitter<StagingStatus> for Audience {}
impl Audience {
    pub(crate) fn staged(&self, attempt: u64) -> bool {
        self.textures.borrow().staged(attempt)
    }
    pub(crate) fn promote(&mut self, attempt: u64, cx: &mut App) -> bool {
        self.textures.borrow_mut().promote(attempt, cx)
    }
    pub(crate) fn cancel_staging(&mut self, cx: &mut App) {
        self.textures.borrow_mut().cancel_stage(cx);
    }
    #[cfg(all(test, feature = "test-support"))]
    pub(crate) fn seed_budget_image(
        &mut self,
        frame: ferrostage_core::BgraFrame,
        budget: usize,
        window: &mut Window,
        cx: &mut gpui::App,
    ) {
        let mut cache = self.textures.borrow_mut();
        cache.sync(
            &[(Surface::Page, frame)],
            &[Surface::Page],
            budget,
            window,
            cx,
        );
        assert!(cache.paint(Surface::Page, None, window, cx, |_, _, _, _| Ok(())));
    }
    #[cfg(all(test, feature = "test-support"))]
    pub(crate) fn effective_texture_budget(&self) -> usize {
        self.textures.borrow().effective_budget()
    }
    pub(crate) fn accepts_texture_budget(&self, bytes: usize, cx: &App) -> bool {
        self.textures.borrow().accepts_budget(bytes)
            && (self.snapshot.read(cx).visual.is_some() || self.textures.borrow().bytes() == 0)
    }
    /// The existing projection is the only incoming budget authority.
    pub(crate) fn commit_snapshot_budget(&mut self, cx: &App) -> Result<(), String> {
        if let Some(visual) = &self.snapshot.read(cx).visual {
            self.textures
                .borrow_mut()
                .apply_budget(visual.texture_budget_bytes)
        } else if self.textures.borrow().bytes() == 0 {
            Ok(())
        } else {
            Err("Projection budget unavailable while retaining GPU images".into())
        }
    }
    pub fn new(snapshot: Entity<AudienceProjectionSnapshot>, cx: &mut Context<Self>) -> Self {
        cx.on_release(|view, cx| view.textures.borrow_mut().release(cx))
            .detach();
        Self {
            snapshot,
            textures: Default::default(),
            reported_error: Default::default(),
        }
    }
}
fn report(
    error: Option<String>,
    reported: &RefCell<Option<String>>,
    view: WeakEntity<Audience>,
    cx: &mut App,
) {
    if *reported.borrow() == error {
        return;
    }
    *reported.borrow_mut() = error.clone();
    cx.defer(move |cx| {
        let _ = view.update(cx, |_, cx| cx.emit(RenderStatus(error)));
    });
}
impl Render for Audience {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let snapshot = self.snapshot.read(cx).clone();
        if snapshot.staging.is_none() {
            self.textures.borrow_mut().cancel_stage(cx);
        }
        let visible = snapshot.stage == AudienceStage::Live && snapshot.blank_mode.is_none();
        let background = if visible
            || snapshot.blank_mode != Some(BlankMode::White)
            || snapshot.stage == AudienceStage::Curtain
        {
            0x000000
        } else {
            0xffffff
        };
        let frames = snapshot
            .visual
            .as_ref()
            .map(|v| {
                std::iter::once((Surface::Page, v.page.clone()))
                    .chain(
                        v.media
                            .iter()
                            .map(|m| (Surface::Media(m.slot_id()), m.pixels().clone())),
                    )
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let budget = snapshot.visual.as_ref().map_or_else(
            || self.textures.borrow().effective_budget(),
            |v| v.texture_budget_bytes,
        );
        let active = std::iter::once(Surface::Page)
            .chain(
                frames
                    .iter()
                    .filter_map(|(s, _)| matches!(s, Surface::Media(_)).then_some(*s)),
            )
            .collect::<Vec<_>>();
        self.textures
            .borrow_mut()
            .sync(&frames, &active, budget, window, cx);
        report(
            self.textures.borrow().error(),
            &self.reported_error,
            cx.weak_entity(),
            cx,
        );
        let textures = self.textures.clone();
        let reported = self.reported_error.clone();
        let view = cx.weak_entity();
        let staging = snapshot.staging.clone();
        let stage_cache = self.textures.clone();
        let stage_view = cx.weak_entity();
        div()
            .size_full()
            .bg(rgb(background))
            .child(
                canvas(
                    |_, _, _| (),
                    move |_, _, window, cx| {
                        if let Some(staging) = staging {
                            let mut cache = stage_cache.borrow_mut();
                            let budget = if cache.effective_budget() == staging.texture_budget_bytes
                            {
                                Ok(())
                            } else {
                                cache.apply_budget(staging.texture_budget_bytes)
                            };
                            let result = budget.and_then(|()| {
                                cache.stage(
                                    staging.attempt,
                                    &[(Surface::Page, staging.pixels)],
                                    window,
                                    cx,
                                    crate::slide_surface::preupload,
                                )
                            });
                            cx.defer(move |cx| {
                                let _ = stage_view.update(cx, |_, cx| {
                                    cx.emit(StagingStatus(staging.attempt, result))
                                });
                            });
                        }
                    },
                )
                .absolute()
                .size_full(),
            )
            .when(visible, |view_element| {
                view_element.child(
                    canvas(
                        |_, _, _| (),
                        move |bounds, _, window, cx| {
                            crate::slide_surface::composite(
                                &mut textures.borrow_mut(),
                                (Surface::Page, bounds),
                                Some(&snapshot),
                                crate::slide_surface::MediaLayers {
                                    frames: snapshot
                                        .visual
                                        .as_ref()
                                        .map_or(&[], |v| v.media.as_slice()),
                                    placeholders: &[],
                                },
                                Surface::Media,
                                window,
                                cx,
                            );
                            report(textures.borrow().error(), &reported, view, cx);
                        },
                    )
                    .size_full(),
                )
            })
    }
}

/// The latest safety state gates even the committed fallback image.
#[cfg(all(test, feature = "test-support"))]
pub(crate) fn paint_projection(
    textures: &mut TextureCache<()>,
    snapshot: &AudienceProjectionSnapshot,
    window: &mut Window,
    cx: &mut App,
    draw: impl FnMut(
        &ferrostage_core::BgraFrame,
        std::sync::Arc<gpui::RenderImage>,
        Option<&AudienceProjectionSnapshot>,
        &mut Window,
    ) -> Result<(), String>,
) -> bool {
    if snapshot.stage != AudienceStage::Live || snapshot.blank_mode.is_some() {
        return false;
    }
    textures.paint((), Some(snapshot), window, cx, draw)
}
