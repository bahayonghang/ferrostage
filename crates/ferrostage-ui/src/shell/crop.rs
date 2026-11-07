//! Presenter-only inspection. A Start transaction is created only after confirmation.
use super::*;
use crate::workbench::{DocumentSession, Surface};
use ferrostage_core::{BgraFrame, NormalizedRect};
use ferrostage_document::{NotesLayout, notes_crop, slide_crop};
use ferrostage_pdf::PixelSize;
use ferrostage_render::{RenderPriority, RenderRequest, RenderResult, Reply};
use std::sync::mpsc::TryRecvError;

pub(crate) struct CropDraft {
    id: u64,
    generation: DocumentGeneration,
    candidate: bool,
    timer: Option<StartTimer>,
    sample_index: usize,
    layout: NotesLayout,
    requests: Vec<RenderRequest>,
    candidate_raster: Option<Reply<Vec<RenderResult>>>,
    candidate_frames: Vec<BgraFrame>,
    error: Option<String>,
    // Keep the inspected pixels and dialog alive while the replacement transaction
    // owns its candidate. Only a successful publication may consume Start.
    retained: Option<CropPreview>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CropPhase {
    Inspect,
    Applying,
    Failed,
    Retrying,
}

#[derive(Clone)]
pub(crate) struct CropPreview {
    pub id: u64,
    pub layout: NotesLayout,
    pub suggestion: Option<NotesLayout>,
    pub sample_index: usize,
    pub sample_count: usize,
    pub from_start: bool,
    pub read_only: bool,
    pub changing_with_annotations: bool,
    pub remapping_annotations: bool,
    pub public_frame: Option<BgraFrame>,
    pub notes_frames: Vec<BgraFrame>,
    pub pending: bool,
    pub error: Option<String>,
    pub can_confirm: bool,
    pub phase: CropPhase,
}

impl Shell {
    pub(crate) fn crop_needs_confirmation(&self) -> bool {
        self.state.stage() == SessionStage::Ready
            && (self.crop_reload_pending_confirmation()
                || (!self.crop_confirmed
                    && self
                        .workbench
                        .document
                        .as_ref()
                        .is_some_and(|s| s.document.suggested_notes_layout().is_some())))
    }

    fn crop_source(&self) -> Option<(DocumentGeneration, &DocumentSession, bool)> {
        if self.state.stage() == SessionStage::Ready
            && let Some((generation, session)) = self.crop_reload_candidate()
        {
            Some((generation, session, true))
        } else {
            Some((
                self.state.generation(),
                self.workbench.document.as_ref()?,
                false,
            ))
        }
    }

    pub(crate) fn crop_preview(&self) -> Option<CropPreview> {
        let draft = self.crop.as_ref()?;
        if let Some(preview) = &draft.retained {
            return Some(preview.clone());
        }
        let (generation, session, candidate) = self.crop_source()?;
        if draft.generation != generation || draft.candidate != candidate {
            return None;
        }
        let mut frames = if candidate {
            draft.candidate_frames.clone()
        } else {
            draft
                .requests
                .iter()
                .enumerate()
                .map_while(|(i, req)| {
                    let f = self.workbench.frame(Surface::CropSample(i))?;
                    (f.request.generation == req.generation
                        && f.request.page == req.page
                        && f.request.crop == req.crop
                        && f.request.target_px == req.target_px)
                        .then(|| f.frame.clone())
                })
                .collect()
        };
        let ready = !draft.requests.is_empty() && draft.requests.len() == frames.len();
        let public_frame = (!frames.is_empty()).then(|| frames.remove(0));
        let error = draft.error.clone().or_else(|| {
            (!candidate && self.workbench.crop_request_failed())
                .then(|| self.locale.text("crop-preview-failed").to_string())
        });
        let read_only = self.state.stage() != SessionStage::Ready;
        let crop_changes = self.state.slide_crop() != slide_crop(draft.layout);
        Some(CropPreview {
            id: draft.id,
            layout: draft.layout,
            suggestion: session.document.suggested_notes_layout(),
            sample_index: draft.sample_index,
            sample_count: session.document.slide_map.steps().len(),
            from_start: draft.timer.is_some(),
            read_only,
            changing_with_annotations: self.state.has_annotations() && crop_changes,
            remapping_annotations: self.state.has_annotations() && candidate && !crop_changes,
            public_frame,
            notes_frames: frames,
            pending: !ready && error.is_none(),
            can_confirm: ready && error.is_none() && !read_only && !self.busy(),
            phase: CropPhase::Inspect,
            error,
        })
    }

    pub(super) fn open_crop(&mut self, timer: Option<StartTimer>) {
        if self.busy() || self.state.stage() == SessionStage::Idle {
            return;
        }
        self.abandon_modal();
        let Some((generation, session, candidate)) = self.crop_source() else {
            return;
        };
        let sample_index = session
            .document
            .slide_map
            .steps()
            .binary_search_by_key(&self.state.current_page().0, |p| p.0)
            .unwrap_or(0);
        self.crop = Some(CropDraft {
            id: 0,
            generation,
            candidate,
            timer,
            sample_index,
            layout: session.document.layout,
            requests: Vec::new(),
            candidate_raster: None,
            candidate_frames: Vec::new(),
            error: None,
            retained: None,
        });
        self.modal = Some(Modal::Crop);
        self.refresh_crop();
    }

    fn refresh_crop(&mut self) {
        let Some(old) = self.crop.as_ref() else {
            return;
        };
        let Some((generation, session, candidate)) = self.crop_source() else {
            self.close_crop();
            return;
        };
        if old.generation != generation || old.candidate != candidate {
            self.close_crop();
            return;
        }
        let page = session.document.slide_map.steps()[old.sample_index];
        let independent = session.document.notes_pages_for(page);
        let notes = notes_crop(old.layout);
        let count = 1 + usize::from(notes.is_some()) + independent.len();
        let budget = (self.settings.settings.cache.l1_gpu_mib as usize * 1024 * 1024 / 4) / count;
        let target = crate::workbench::budgeted_target(
            PixelSize {
                width: 720,
                height: 480,
            },
            budget,
        );
        let request = |page, crop| RenderRequest {
            generation,
            page,
            crop,
            target_px: target,
            priority: RenderPriority::Presenter,
            epoch: 0,
        };
        let mut requests = vec![request(page, slide_crop(old.layout))];
        if let Some(rect) = notes {
            requests.push(request(page, rect));
        }
        requests.extend(
            independent
                .iter()
                .map(|p| request(*p, NormalizedRect::FULL)),
        );

        let candidate_raster = if candidate {
            Some(self.crop_reload_preview(generation, requests.clone()))
        } else {
            None
        };
        self.workbench.set_crop_requests(if candidate {
            Vec::new()
        } else {
            requests.clone()
        });
        let draft = self.crop.as_mut().expect("checked crop draft");
        self.contextual_serial += 1;
        draft.id = self.contextual_serial;
        draft.requests = requests;
        draft.error = self
            .renderer
            .is_err()
            .then(|| self.locale.text("crop-preview-unavailable").into());
        draft.candidate_frames.clear();
        draft.candidate_raster = None;
        if let Some(result) = candidate_raster {
            match result {
                Ok(reply) => draft.candidate_raster = Some(reply),
                Err(cause) => {
                    tracing::warn!(%cause,"crop candidate preview unavailable");
                    draft.error = Some(self.locale.text("crop-preview-failed").into());
                }
            }
        }
        if !candidate
            && let Ok(worker) = &self.renderer
            && let Err(cause) =
                self.workbench
                    .schedule(worker, &self.state, &self.settings.settings.presenter)
        {
            tracing::warn!(%cause,"crop preview requests rejected");
            draft.error = Some(self.locale.text("crop-preview-failed").into());
        }
    }

    pub(super) fn poll_crop(&mut self) {
        let Some(draft) = self.crop.as_ref() else {
            return;
        };
        if let Some(retained) = &draft.retained {
            if retained.phase == CropPhase::Retrying {
                if self.crop_reload_candidate().is_some() {
                    self.open_crop(None);
                } else if !self.reload_blocks_playback() || self.reload.failed_worker {
                    self.fail_crop_reload();
                }
            }
            return;
        }
        if !self
            .crop_source()
            .is_some_and(|(generation, _, candidate)| {
                generation == draft.generation && candidate == draft.candidate
            })
        {
            self.close_crop();
            return;
        }
        let draft = self.crop.as_mut().expect("checked crop draft");
        let Some(reply) = &draft.candidate_raster else {
            return;
        };
        let result = match reply.try_recv() {
            Err(TryRecvError::Empty) => return,
            Ok(Ok(results)) => {
                // Completion order is not the presentation order. Match each full key.
                let frames: Option<Vec<_>> = draft
                    .requests
                    .iter()
                    .map(|req| {
                        results
                            .iter()
                            .find(|r| r.request == *req)
                            .map(|r| r.frame.clone())
                    })
                    .collect();
                frames
            }
            _ => None,
        };
        draft.candidate_raster = None;
        if let Some(frames) = result {
            draft.candidate_frames = frames;
        } else {
            draft.error = Some(self.locale.text("crop-preview-failed").into());
        }
    }

    fn confirm_crop(&mut self, id: u64, cx: &mut Context<Self>) {
        let Some(preview) = self.crop_preview().filter(|p| p.id == id && p.can_confirm) else {
            return;
        };
        let draft = self.crop.as_ref().expect("validated draft");
        let (candidate, generation, timer) = (draft.candidate, draft.generation, draft.timer);
        let result = if candidate {
            self.confirm_reload_crop(generation, preview.layout, cx)
        } else {
            self.apply_crop_layout(preview.layout)
        };
        if let Err(cause) = result {
            tracing::warn!(%cause,"crop application rejected");
            if let Some(draft) = &mut self.crop {
                draft.error = Some(self.locale.text("crop-apply-failed").into());
            }
            return;
        }
        if candidate {
            self.contextual_serial += 1;
            let draft = self.crop.as_mut().expect("validated draft");
            draft.id = self.contextual_serial;
            draft.retained = Some(CropPreview {
                id: draft.id,
                phase: CropPhase::Applying,
                can_confirm: false,
                ..preview
            });
        } else {
            self.close_crop();
            self.crop_confirmed = true;
            self.publish(false, cx);
            if let Some(timer) = timer {
                self.start(timer, cx);
            }
        }
    }

    fn apply_crop_layout(&mut self, layout: NotesLayout) -> Result<(), String> {
        let session = self.workbench.document.as_ref().ok_or("Missing document")?;
        if layout == session.document.layout {
            return Ok(());
        }
        let mut options = session.document.overrides;
        options.layout = Some(layout);
        self.apply_layout_options(options)
    }

    pub(super) fn finish_crop_reload(&mut self, cx: &mut Context<Self>) {
        let Some(draft) = self.crop.as_ref().filter(|d| {
            d.retained
                .as_ref()
                .is_some_and(|p| p.phase == CropPhase::Applying)
        }) else {
            return;
        };
        if self.state.generation() != draft.generation {
            self.fail_crop_reload();
            return;
        }
        let timer = draft.timer;
        self.close_crop();
        if let Some(timer) = timer
            && self.state.stage() == SessionStage::Ready
        {
            self.start(timer, cx);
        }
    }

    fn fail_crop_reload(&mut self) {
        if let Some(draft) = self.crop.as_mut()
            && let Some(preview) = &mut draft.retained
        {
            draft.timer = None;
            preview.from_start = false;
            preview.phase = CropPhase::Failed;
            preview.error = Some(
                self.error
                    .clone()
                    .unwrap_or_else(|| self.locale.text("crop-apply-failed").into()),
            );
        }
    }

    pub(super) fn close_crop(&mut self) {
        if let Some(draft) = self.crop.take() {
            if draft
                .retained
                .as_ref()
                .is_some_and(|p| p.phase == CropPhase::Applying)
            {
                self.cancel_crop_reload(draft.generation);
            }
            self.workbench.set_crop_requests(Vec::new());
        }
        if self.modal == Some(Modal::Crop) {
            self.modal = None;
        }
    }

    pub(super) fn reset_crop_session(&mut self) {
        self.close_crop();
        self.crop_confirmed = false;
    }

    pub(super) fn dispatch_crop(&mut self, intent: &Intent, cx: &mut Context<Self>) -> bool {
        if matches!(intent, Intent::OpenCrop) {
            if self.modal.is_none() {
                self.open_crop(None);
            }
            return true;
        }
        let Some(draft) = self.crop.as_ref() else {
            return false;
        };
        let id = draft.id;
        if let Some(preview) = &draft.retained {
            match intent {
                Intent::CancelCrop(ev) if *ev == id => self.close_crop(),
                Intent::Escape => self.close_crop(),
                Intent::RetryCrop(ev) if *ev == id && preview.phase == CropPhase::Failed => {
                    let clear_reported_error = self.error.is_some() && self.error == preview.error;
                    if self.crop_reload_candidate().is_some() {
                        if clear_reported_error {
                            self.error = None;
                        }
                        self.open_crop(None);
                    } else if self.retry_crop_reload(draft.generation) {
                        if clear_reported_error {
                            self.error = None;
                        }
                        let saved = self.crop.as_mut().unwrap().retained.as_mut().unwrap();
                        saved.phase = CropPhase::Retrying;
                        saved.error = None;
                    } else {
                        self.crop.as_mut().unwrap().retained.as_mut().unwrap().error =
                            Some(self.locale.text("crop-retry-unavailable").into());
                    }
                }
                Intent::Stop | Intent::Home => {
                    self.close_crop();
                    return false;
                }
                _ => {}
            }
            return true;
        }
        match intent {
            Intent::CropLayout { id: ev, layout }
                if *ev == id
                    && self.state.stage() == SessionStage::Ready
                    && draft.layout != *layout =>
            {
                self.crop.as_mut().unwrap().layout = *layout;
                self.refresh_crop();
            }
            Intent::CropSample { id: ev, delta } if *ev == id => {
                let Some((_, session, _)) = self.crop_source() else {
                    return true;
                };
                let index = draft
                    .sample_index
                    .saturating_add_signed(*delta)
                    .min(session.document.slide_map.steps().len() - 1);
                if index != draft.sample_index {
                    self.crop.as_mut().unwrap().sample_index = index;
                    self.refresh_crop();
                }
            }
            Intent::RetryCrop(ev) if *ev == id => self.refresh_crop(),
            Intent::ConfirmCrop(ev) if *ev == id => self.confirm_crop(id, cx),
            Intent::CancelCrop(ev) if *ev == id => self.close_crop(),
            Intent::Escape => self.close_crop(),
            Intent::Stop | Intent::Home => return false,
            _ => {}
        }
        true
    }
}
