use super::*;
use crate::workbench::Surface;
use ferrostage_core::PhysicalPage;
use ferrostage_render::RenderRequest;

pub(super) struct ReturnPosition {
    generation: DocumentGeneration,
    page: PhysicalPage,
    // A private paint capability, renewed on navigation; public playback keeps
    // its existing identity. Retaining it prevents allocation reuse by old callbacks.
    paint: Rc<()>,
}

impl Shell {
    pub(super) fn record_return_position(
        &mut self,
        direct: bool,
        generation: DocumentGeneration,
        page: PhysicalPage,
    ) {
        if self.return_position.is_none() && direct {
            self.return_position = Some(ReturnPosition {
                generation,
                page,
                paint: Rc::new(()),
            });
        } else if let Some(anchor) = &mut self.return_position {
            anchor.paint = Rc::new(());
        }
    }

    pub(super) fn return_to_position(&mut self, cx: &mut Context<Self>) {
        let Some(target) = self.return_target() else {
            return;
        };
        let retry = target == self.state.current_page();
        if retry && let Some(anchor) = &mut self.return_position {
            anchor.paint = Rc::new(());
        }
        self.apply(Command::JumpTo(target), cx);
        if retry {
            self.workbench.retry_failed_current();
            self.update_render();
            self.publish(self.pending.is_some(), cx);
            cx.notify();
        }
    }

    pub(crate) fn return_attempt_token(&self) -> Option<Rc<()>> {
        self.return_position
            .as_ref()
            .map(|anchor| anchor.paint.clone())
    }

    pub(super) fn return_target(&self) -> Option<PhysicalPage> {
        let anchor = self.return_position.as_ref()?;
        (anchor.generation == self.state.generation()
            && self.state.slide_map()?.logical_for(anchor.page).is_some())
        .then_some(anchor.page)
    }

    pub(crate) fn return_label(&self) -> Option<String> {
        let page = self.return_target()?;
        Some(self.locale.format(
            "wb-return",
            &[(
                "page",
                self.projection_page(ferrostage_core::PlaybackIdentity {
                    generation: self.state.generation(),
                    navigation: 0,
                    page,
                }),
            )],
        ))
    }

    pub(crate) fn ready_return_token(&self) -> Option<Rc<()>> {
        (self.state.stage() == SessionStage::Ready
            && self.modal.is_none()
            && self.contextual.is_none()
            && !self.busy()
            && !self.reload_blocks_playback()
            && self.graphics_error.is_none()
            && self.return_target() == Some(self.state.current_page()))
        .then(|| self.return_position.as_ref().unwrap().paint.clone())
    }

    pub(super) fn return_preview_painted(
        &mut self,
        source: WindowId,
        token: &Rc<()>,
        request: RenderRequest,
        cx: &mut Context<Self>,
    ) {
        let Some(current) = self.ready_return_token() else {
            return;
        };
        if !Rc::ptr_eq(&current, token)
            || self
                .presenter
                .as_ref()
                .is_none_or(|p| p.handle.window_id() != source)
            || self
                .workbench
                .frame(Surface::Current)
                .is_none_or(|f| f.request != request)
        {
            return;
        }
        let exact = self
            .workbench
            .requests(&self.state, &self.settings.settings.presenter)
            .is_ok_and(|requests| {
                requests.iter().any(|(surface, expected)| {
                    *surface == Surface::Current
                        && expected.generation == request.generation
                        && expected.page == request.page
                        && expected.crop == request.crop
                        && expected.target_px == request.target_px
                        && expected.priority == request.priority
                })
            });
        if exact {
            self.return_position = None;
            cx.notify();
        }
    }

    pub(super) fn consume_displayed_return(&mut self) -> bool {
        let Some(anchor) = &self.return_position else {
            return false;
        };
        if anchor.generation != self.state.generation() {
            self.return_position = None;
            return true;
        }
        if self.state.stage() != SessionStage::Presenting
            || self.state.current_page() != anchor.page
        {
            return false;
        }
        let shown = if self.state.audience_snapshot().blank_mode.is_some() {
            self.public_frame_ready()
        } else {
            self.selected_playback()
                .is_some_and(|selected| self.settled_playback() == Some(selected))
        };
        if shown {
            self.return_position = None;
        }
        shown
    }
}
