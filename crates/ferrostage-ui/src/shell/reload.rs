//! The UI owns the replacement transaction and every outstanding worker reply.
use super::*;
use crate::workbench::{DocumentSession, ReadyFrame, Surface, Workbench};
use ferrostage_core::{AudienceStagingFrame, BgraFrame, DocumentGeneration};
use ferrostage_pdf::{DocumentMetadata, DocumentStructure};
use ferrostage_render::{CandidateReady, GenerationAck, RenderPriority, Reply, RollbackToken};
use std::sync::mpsc;

#[derive(Clone)]
pub(crate) struct PresenterStaging {
    pub attempt: u64,
    pub frames: Vec<(Surface, BgraFrame)>,
}

#[cfg(all(test, feature = "test-support", feature = "pdfium"))]
mod tests;
#[cfg(all(test, feature = "test-support"))]
mod playback_tests {
    use super::*;

    #[gpui::test]
    fn timed_reload_worker_failure_latch_blocks_retained_pixels_until_recovery(
        cx: &mut gpui::TestAppContext,
    ) {
        let (shell, _, audience) = crate::shell::tests::timed_fixture(cx);
        shell.update(cx, |s, cx| {
            crate::shell::tests::timed_frame(s, cx);
            let old = s.snapshot.read(cx).playback.unwrap().identity;
            s.invalidate_reload(true);
            // A failed commit rollback has finished its transaction, but the
            // worker generation remains unusable until explicit recovery.
            s.reload.failed_worker = true;
            s.reload.attempted = Some(s.reload.attempt);
            s.publish(false, cx);
            assert!(s.reload_blocks_render());
            assert!(s.snapshot.read(cx).visual.is_some());
            assert!(s.snapshot.read(cx).playback.is_none());
            for phase in [
                crate::audience::PlaybackPhase::FirstVisible,
                crate::audience::PlaybackPhase::Settled,
            ] {
                s.playback_event(
                    audience.window_id(),
                    crate::audience::PlaybackEvent {
                        identity: old,
                        phase,
                        at: Instant::now(),
                    },
                    cx,
                );
            }
            s.poll_timed_playback(cx);
            assert_eq!(s.state.current_page(), PhysicalPage(0));
            s.reload.failed_worker = false;
            s.publish(false, cx);
            assert_ne!(s.snapshot.read(cx).playback.unwrap().identity, old);
            s.poll_timed_playback(cx);
            assert_eq!(s.state.current_page(), PhysicalPage(0));
        });
    }

    #[gpui::test]
    fn timed_reload_preflight_rejects_paint_events_until_a_fresh_snapshot(
        cx: &mut gpui::TestAppContext,
    ) {
        let (shell, _, audience) = crate::shell::tests::timed_fixture(cx);
        shell.update(cx, |s, cx| {
            crate::shell::tests::timed_frame(s, cx);
            let old = s.snapshot.read(cx).playback.unwrap().identity;
            s.invalidate_reload(true);
            s.reload.awaiting_bytes = true;
            s.publish(false, cx);
            assert!(s.snapshot.read(cx).playback.is_none());
            for phase in [
                crate::audience::PlaybackPhase::FirstVisible,
                crate::audience::PlaybackPhase::Settled,
            ] {
                s.playback_event(
                    audience.window_id(),
                    crate::audience::PlaybackEvent {
                        identity: old,
                        phase,
                        at: Instant::now(),
                    },
                    cx,
                );
            }
            s.poll_timed_playback(cx);
            assert_eq!(s.state.current_page(), PhysicalPage(0));
            s.reload.awaiting_bytes = false;
            let (_sender, receiver) = mpsc::channel();
            s.reload.parsing = Some((s.reload.attempt, receiver));
            s.publish(false, cx);
            assert!(s.snapshot.read(cx).playback.is_none());
            // A rejected replacement may keep the old document, but must paint
            // a fresh identity before it is eligible to advance again.
            s.reload.parsing = None;
            s.reload.attempted = Some(s.reload.attempt);
            s.publish(false, cx);
            assert_ne!(s.snapshot.read(cx).playback.unwrap().identity, old);
            s.poll_timed_playback(cx);
            assert_eq!(s.state.current_page(), PhysicalPage(0));
        });
    }
}
pub(super) struct Watch {
    _owner: ferrostage_platform::PdfWatcher,
    events: Receiver<ferrostage_platform::PdfWatchEvent>,
}
#[derive(Clone, PartialEq)]
struct Identity {
    generation: DocumentGeneration,
    page: PhysicalPage,
    stage: SessionStage,
    options: ferrostage_document::DocumentOverrides,
    presenter: Option<WindowId>,
    audience: Option<WindowId>,
    targets: std::collections::HashMap<Surface, ferrostage_pdf::PixelSize>,
    presenter_target: Option<ferrostage_pdf::PixelSize>,
    audience_target: Option<ferrostage_pdf::PixelSize>,
    settings: ferrostage_settings::PresenterSettings,
    budget: usize,
}
enum Phase {
    Metadata(Reply<DocumentMetadata>),
    Document(Receiver<Result<DocumentSession, String>>),
    Raster(Reply<CandidateReady>),
    Upload,
    Commit(Reply<RollbackToken>),
    Rollback(Reply<GenerationAck>),
    Abort(Reply<GenerationAck>),
    Retire(Reply<GenerationAck>),
}
struct Pending {
    attempt: u64,
    generation: DocumentGeneration,
    identity: Identity,
    phase: Phase,
    session: Option<DocumentSession>,
    structure: Option<DocumentStructure>,
    ready: Option<CandidateReady>,
    frames: Vec<(Surface, ReadyFrame)>,
    requests: Vec<(Surface, ferrostage_render::RenderRequest)>,
    presenter_ready: bool,
    audience_ready: bool,
    obsolete: bool,
    published: bool,
}
#[derive(Default)]
pub(super) struct Reload {
    watch: Option<Watch>,
    attempt: u64,
    watch_attempt: u64,
    awaiting_bytes: bool,
    bytes: Option<Vec<u8>>,
    structure: Option<DocumentStructure>,
    parsing: Option<(u64, Receiver<Result<DocumentStructure, String>>)>,
    pending: Option<Pending>,
    attempted: Option<u64>,
    last_identity: Option<Identity>,
    pub(super) failed_worker: bool,
    queued_import: Option<PathBuf>,
    orphans: Vec<DocumentGeneration>,
    retirements: Vec<Reply<GenerationAck>>,
}
fn receive<T>(reply: &Reply<T>) -> Option<Result<T, RenderError>> {
    match reply.try_recv() {
        Ok(value) => Some(value),
        Err(mpsc::TryRecvError::Empty) => None,
        Err(mpsc::TryRecvError::Disconnected) => Some(Err(RenderError::WorkerUnavailable)),
    }
}
fn invalid_pdf(error: &RenderError) -> bool {
    matches!(
        error,
        RenderError::Pdf(
            ferrostage_pdf::PdfError::Unreadable
                | ferrostage_pdf::PdfError::EmptyDocument
                | ferrostage_pdf::PdfError::InvalidPageGeometry
        )
    )
}
impl Shell {
    pub(super) fn shutdown_reload(&mut self) {
        self.reload.watch = None;
        // Shutdown is ordered after outstanding Commit/Abort commands and joins
        // the sole PDFium thread. No projection survives application shutdown.
        if let Ok(worker) =
            std::mem::replace(&mut self.renderer, Err(RenderError::WorkerUnavailable))
            && let Err(error) = worker.shutdown()
        {
            tracing::error!(%error, "PDF worker shutdown failed");
        }
        self.reload.pending = None;
    }
    pub(super) fn recover_reload_orphans(&mut self) {
        if let Ok(worker) = &self.renderer {
            for generation in self.reload.orphans.drain(..) {
                if let Ok(reply) = worker.retire_generation(generation) {
                    self.reload.retirements.push(reply);
                }
            }
        }
    }
    fn reload_identity(&self) -> Option<Identity> {
        let session = self.workbench.document.as_ref()?;
        Some(Identity {
            generation: self.state.generation(),
            page: self.state.current_page(),
            stage: self.state.stage(),
            options: session.document.overrides,
            presenter: self.presenter.as_ref().map(|w| w.handle.window_id()),
            audience: self.audience.as_ref().map(|w| w.handle.window_id()),
            targets: self.workbench.surface_targets.clone(),
            presenter_target: self.workbench.presenter_target,
            audience_target: self.workbench.audience_target,
            settings: self.settings.settings.presenter.clone(),
            budget: self.settings.settings.cache.l1_gpu_mib as usize * 1024 * 1024 / 2,
        })
    }
    pub(super) fn replace_pdf_watch(
        &mut self,
        settings: &ferrostage_settings::HotReloadSettings,
    ) -> Result<(), String> {
        let next = self.prepare_pdf_watch(settings)?;
        self.install_pdf_watch(next);
        Ok(())
    }
    pub(super) fn prepare_pdf_watch(
        &self,
        settings: &ferrostage_settings::HotReloadSettings,
    ) -> Result<Option<Watch>, String> {
        let next = if settings.enabled {
            if let Some(path) = &self.path {
                let (sink, events) = mpsc::channel();
                Some(Watch {
                    _owner: ferrostage_platform::watch_pdf(
                        path,
                        Duration::from_millis(settings.debounce_ms),
                        sink,
                    )
                    .map_err(|e| e.to_string())?,
                    events,
                })
            } else {
                None
            }
        } else {
            None
        };
        Ok(next)
    }
    pub(super) fn install_pdf_watch(&mut self, next: Option<Watch>) {
        self.reload.watch = next;
        self.reload.awaiting_bytes = false;
        self.invalidate_reload(true);
    }
    fn invalidate_reload(&mut self, discard_bytes: bool) {
        self.invalidate_playback();
        self.reload.attempt += 1;
        if let Some(pending) = &mut self.reload.pending {
            pending.obsolete = true;
        }
        if discard_bytes {
            self.reload.bytes = None;
            self.reload.structure = None;
        }
    }
    pub(super) fn queue_reload_import(&mut self, path: PathBuf) -> bool {
        if self.reload.pending.is_some() {
            self.invalidate_reload(true);
            self.reload.queued_import = Some(path);
            true
        } else {
            false
        }
    }
    pub(super) fn reload_blocks_render(&self) -> bool {
        self.reload.failed_worker
            || self
                .reload
                .pending
                .as_ref()
                .is_some_and(|p| matches!(p.phase, Phase::Commit(_) | Phase::Rollback(_)))
    }
    pub(super) fn reload_busy(&self) -> bool {
        self.reload.pending.is_some()
    }
    pub(super) fn reload_blocks_playback(&self) -> bool {
        self.reload.failed_worker
            || self.reload.awaiting_bytes
            || self.reload.parsing.is_some()
            || self.reload.pending.is_some()
            || (self.reload.bytes.is_some() && self.reload.attempted != Some(self.reload.attempt))
    }
    pub(crate) fn presenter_staging(&self) -> Option<PresenterStaging> {
        let pending = self.reload.pending.as_ref()?;
        (!pending.obsolete && matches!(pending.phase, Phase::Upload | Phase::Commit(_))).then(
            || PresenterStaging {
                attempt: pending.attempt,
                frames: pending
                    .frames
                    .iter()
                    .filter(|(s, _)| *s != Surface::Audience)
                    .map(|(s, f)| (*s, f.frame.clone()))
                    .collect(),
            },
        )
    }
    pub(super) fn audience_staging(&self) -> Option<AudienceStagingFrame> {
        let pending = self.reload.pending.as_ref()?;
        if pending.obsolete || !matches!(pending.phase, Phase::Upload | Phase::Commit(_)) {
            return None;
        }
        let (_, frame) = pending
            .frames
            .iter()
            .find(|(s, _)| *s == Surface::Audience)?;
        Some(AudienceStagingFrame {
            attempt: pending.attempt,
            generation: pending.generation,
            current_page: frame.request.page,
            slide_crop: frame.request.crop,
            pixels: frame.frame.clone(),
            texture_budget_bytes: pending.identity.budget,
        })
    }
    pub(crate) fn staging_status(
        &mut self,
        attempt: u64,
        audience: bool,
        result: Result<(), String>,
        cx: &mut Context<Self>,
    ) {
        let Some(pending) = &mut self.reload.pending else {
            return;
        };
        if pending.attempt != attempt || pending.obsolete || !matches!(pending.phase, Phase::Upload)
        {
            return;
        }
        if result.is_ok()
            && if audience {
                pending.audience_ready
            } else {
                pending.presenter_ready
            }
        {
            return;
        }
        match result {
            Ok(()) => {
                if audience {
                    pending.audience_ready = true;
                } else {
                    pending.presenter_ready = true;
                }
            }
            Err(error) => {
                pending.obsolete = true;
                self.error = Some(self.locale.error("wb-upload-error", error));
            }
        }
        cx.notify();
    }
    fn cancel_reload_images(&self, cx: &mut Context<Self>) {
        if let Some(record) = &self.presenter {
            let _ = record.handle.update(cx, |view, _, cx| {
                if let Ok(view) = view.downcast::<Presenter>() {
                    view.update(cx, |view, cx| view.cancel_staging(cx));
                }
            });
        }
        if let Some(record) = &self.audience {
            let _ = record.handle.update(cx, |view, _, cx| {
                if let Ok(view) = view.downcast::<Audience>() {
                    view.update(cx, |view, cx| view.cancel_staging(cx));
                }
            });
        }
    }
    fn uploaded(&self, attempt: u64, cx: &App) -> bool {
        self.presenter.as_ref().is_some_and(|w| {
            w.handle
                .read(cx, |view: Entity<Presenter>, cx| {
                    view.read(cx).staged(attempt)
                })
                .unwrap_or(false)
        }) && self.audience.as_ref().is_none_or(|w| {
            w.handle
                .read(cx, |view: Entity<Audience>, cx| {
                    view.read(cx).staged(attempt)
                })
                .unwrap_or(false)
        })
    }
    fn promote_reload_images(&self, attempt: u64, cx: &mut Context<Self>) {
        if let Some(w) = &self.presenter {
            let _ = w.handle.update(cx, |view, _, cx| {
                if let Ok(view) = view.downcast::<Presenter>() {
                    view.update(cx, |view, cx| view.promote(attempt, cx));
                }
            });
        }
        if let Some(w) = &self.audience {
            let _ = w.handle.update(cx, |view, _, cx| {
                if let Ok(view) = view.downcast::<Audience>() {
                    view.update(cx, |view, cx| view.promote(attempt, cx));
                }
            });
        }
    }
    pub(super) fn poll_reload(&mut self, cx: &mut Context<Self>) {
        self.reload.retirements.retain(|reply| {
            if let Some(result) = receive(reply) {
                if let Err(error) = result {
                    tracing::error!(%error, "recovered generation retirement failed");
                }
                false
            } else {
                true
            }
        });
        let events: Vec<_> = self
            .reload
            .watch
            .as_ref()
            .map(|watch| watch.events.try_iter().collect())
            .unwrap_or_default();
        for event in events {
            match event {
                ferrostage_platform::PdfWatchEvent::Invalidated(attempt) => {
                    self.reload.watch_attempt = attempt;
                    self.invalidate_reload(true);
                    self.reload.awaiting_bytes = true;
                }
                ferrostage_platform::PdfWatchEvent::Ready { attempt, bytes }
                    if attempt == self.reload.watch_attempt =>
                {
                    self.reload.awaiting_bytes = false;
                    self.reload.bytes = Some(bytes);
                }
                ferrostage_platform::PdfWatchEvent::TimedOut(attempt)
                    if attempt == self.reload.watch_attempt =>
                {
                    self.reload.awaiting_bytes = false;
                    tracing::debug!(
                        attempt,
                        "external replacement left active document unchanged"
                    );
                }
                _ => {}
            }
        }
        let identity = self.reload_identity();
        if self.reload.pending.is_none()
            && self.reload.bytes.is_some()
            && self.reload.last_identity.is_some()
            && self.reload.last_identity != identity
        {
            self.invalidate_reload(false);
            self.reload.last_identity = identity.clone();
        }
        if self
            .reload
            .pending
            .as_ref()
            .is_some_and(|p| !p.obsolete && !p.published && Some(&p.identity) != identity.as_ref())
        {
            self.invalidate_reload(false);
        }
        if let Some(mut pending) = self.reload.pending.take() {
            let keep = self.advance_reload(&mut pending, cx);
            if keep {
                self.reload.pending = Some(pending);
            } else {
                self.cancel_reload_images(cx);
            }
        }
        if self.reload.pending.is_none()
            && let Some(path) = self.reload.queued_import.take()
        {
            self.reload.failed_worker = false;
            self.begin_import(path, cx);
            return;
        }
        if let Some((attempt, parsing)) = &self.reload.parsing {
            match parsing.try_recv() {
                Ok(result) => {
                    if *attempt == self.reload.attempt {
                        match result {
                            Ok(structure) => self.reload.structure = Some(structure),
                            Err(error) => {
                                tracing::debug!(%error, "external PDF structure not ready; retaining active document");
                                self.reload.attempted = Some(*attempt);
                            }
                        }
                    }
                    self.reload.parsing = None;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.error = Some("Replacement structure worker unavailable".into());
                    self.reload.parsing = None;
                    self.reload.attempted = Some(self.reload.attempt);
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        if self.reload.pending.is_some()
            || self.reload.failed_worker
            || self.reload.attempted == Some(self.reload.attempt)
            || self.loading
            || self.pending.is_some()
            || self.preference_change.is_some()
        {
            return;
        }
        let Some(bytes) = &self.reload.bytes else {
            return;
        };
        let Some(structure) = &self.reload.structure else {
            if self.reload.parsing.is_none() {
                let bytes = bytes.clone();
                let (tx, rx) = mpsc::channel();
                match std::thread::Builder::new()
                    .name("ferrostage-structure".into())
                    .spawn(move || {
                        let result = ferrostage_pdf::structure::extract_structure(&bytes)
                            .map_err(|e| e.to_string());
                        tx.send(result).ok();
                    }) {
                    Ok(_) => self.reload.parsing = Some((self.reload.attempt, rx)),
                    Err(error) => {
                        self.error = Some(error.to_string());
                        self.reload.attempted = Some(self.reload.attempt);
                    }
                }
            }
            return;
        };
        let Some(identity) = identity else { return };
        let Ok(worker) = &self.renderer else { return };
        let generation =
            DocumentGeneration(self.next_generation.max(self.state.generation().0 + 1));
        self.next_generation = generation.0 + 1;
        self.reload.attempted = Some(self.reload.attempt);
        self.reload.last_identity = Some(identity.clone());
        match worker.prepare_candidate(generation, bytes.clone()) {
            Ok(reply) => {
                self.reload.pending = Some(Pending {
                    attempt: self.reload.attempt,
                    generation,
                    identity,
                    phase: Phase::Metadata(reply),
                    session: None,
                    structure: Some(structure.clone()),
                    ready: None,
                    frames: Vec::new(),
                    requests: Vec::new(),
                    presenter_ready: false,
                    audience_ready: self.audience.is_none(),
                    obsolete: false,
                    published: false,
                })
            }
            Err(error) => self.error = Some(error.to_string()),
        }
    }
    fn advance_reload(&mut self, pending: &mut Pending, cx: &mut Context<Self>) -> bool {
        if self.renderer.is_err() {
            return false;
        }
        if pending.obsolete
            && matches!(
                pending.phase,
                Phase::Metadata(_) | Phase::Document(_) | Phase::Raster(_) | Phase::Upload
            )
        {
            match self
                .renderer
                .as_ref()
                .expect("renderer checked")
                .abort_candidate(pending.generation)
            {
                Ok(reply) => pending.phase = Phase::Abort(reply),
                Err(error) => {
                    self.error = Some(error.to_string());
                    return false;
                }
            }
        }
        let mut invalid_input = false;
        let mut committed = false;
        let result: Result<Option<Phase>, String> = (|| {
            match &pending.phase {
                Phase::Metadata(reply) => {
                    let Some(metadata) = receive(reply) else {
                        return Ok(None);
                    };
                    let metadata = metadata.map_err(|e| {
                        invalid_input = invalid_pdf(&e);
                        e.to_string()
                    })?;
                    let structure = pending
                        .structure
                        .take()
                        .ok_or("Missing replacement structure")?;
                    let options = pending.identity.options;
                    let (sender, receiver) = mpsc::channel();
                    std::thread::Builder::new()
                        .name("ferrostage-document".into())
                        .spawn(move || {
                            let document = ferrostage_document::LoadedDocument::build(
                                &metadata, &structure, options,
                            );
                            sender
                                .send(
                                    document
                                        .map(|document| DocumentSession {
                                            metadata,
                                            structure,
                                            document,
                                        })
                                        .map_err(|error| error.to_string()),
                                )
                                .ok();
                        })
                        .map_err(|e| e.to_string())?;
                    Ok(Some(Phase::Document(receiver)))
                }
                Phase::Document(reply) => {
                    let session = match reply.try_recv() {
                        Ok(session) => session.inspect_err(|_| {
                            invalid_input = true;
                        })?,
                        Err(mpsc::TryRecvError::Empty) => return Ok(None),
                        Err(mpsc::TryRecvError::Disconnected) => {
                            return Err("Document builder unavailable".into());
                        }
                    };
                    pending.session = Some(session);
                    let session = pending
                        .session
                        .as_ref()
                        .ok_or("Missing replacement document")?;
                    let mut state = self.state.clone();
                    let old = self.workbench.document.as_ref().ok_or("Document closed")?;
                    let mapping = old
                        .document
                        .reload_mapping(&session.document, state.current_page());
                    state
                        .commit_reload(
                            session
                                .document
                                .descriptor(pending.generation)
                                .map_err(|e| e.to_string())?,
                            mapping.position,
                            &mapping.pages,
                        )
                        .map_err(|e| e.to_string())?;
                    let mut plan = Workbench::default();
                    plan.document = Some(session.clone());
                    plan.surface_targets = pending.identity.targets.clone();
                    plan.presenter_target = pending.identity.presenter_target;
                    plan.audience_target = pending.identity.audience_target;
                    pending.requests = plan
                        .requests(&state, &pending.identity.settings)?
                        .into_iter()
                        .filter(|(_, r)| r.priority <= RenderPriority::Presenter)
                        .collect();
                    let reply = self
                        .renderer
                        .as_ref()
                        .expect("renderer checked")
                        .pre_render_candidate(
                            pending.generation,
                            pending.requests.iter().map(|(_, r)| *r).collect(),
                        )
                        .map_err(|e| e.to_string())?;
                    Ok(Some(Phase::Raster(reply)))
                }
                Phase::Raster(reply) => {
                    let Some(ready) = receive(reply) else {
                        return Ok(None);
                    };
                    let ready = ready.map_err(|e| {
                        invalid_input = invalid_pdf(&e);
                        e.to_string()
                    })?;
                    for (surface, request) in &pending.requests {
                        let frame = ready
                            .frames
                            .iter()
                            .find(|f| f.request == *request)
                            .ok_or("Replacement raster identity mismatch")?;
                        pending.frames.push((
                            *surface,
                            ReadyFrame {
                                request: *request,
                                frame: frame.frame.clone(),
                            },
                        ));
                    }
                    pending.ready = Some(ready);
                    Ok(Some(Phase::Upload))
                }
                Phase::Upload => {
                    if !pending.presenter_ready
                        || !pending.audience_ready
                        || !self.uploaded(pending.attempt, cx)
                    {
                        return Ok(None);
                    }
                    let ready = pending
                        .ready
                        .take()
                        .ok_or("Missing replacement readiness token")?;
                    Ok(Some(Phase::Commit(
                        self.renderer
                            .as_ref()
                            .expect("renderer checked")
                            .commit_reversible(ready)
                            .map_err(|e| e.to_string())?,
                    )))
                }
                Phase::Commit(reply) => {
                    let Some(token) = receive(reply) else {
                        return Ok(None);
                    };
                    let token = token.map_err(|e| e.to_string())?;
                    committed = true;
                    if pending.obsolete
                        || pending.attempt != self.reload.attempt
                        || token.generation() != pending.generation
                        || !self.uploaded(pending.attempt, cx)
                    {
                        return Ok(Some(Phase::Rollback(
                            self.renderer
                                .as_ref()
                                .expect("renderer checked")
                                .rollback_committed(token)
                                .map_err(|e| e.to_string())?,
                        )));
                    }
                    let session = pending
                        .session
                        .as_ref()
                        .ok_or("Missing replacement session")?;
                    let old = self.workbench.document.as_ref().ok_or("Document closed")?;
                    let mapping = old
                        .document
                        .reload_mapping(&session.document, self.state.current_page());
                    let mut state = self.state.clone();
                    let applied = session
                        .document
                        .descriptor(pending.generation)
                        .map_err(|e| e.to_string())
                        .and_then(|d| {
                            state
                                .commit_reload(d, mapping.position, &mapping.pages)
                                .map_err(|e| e.to_string())
                        });
                    let dropped = match applied {
                        Ok(dropped) => dropped,
                        Err(error) => {
                            self.error = Some(error);
                            return Ok(Some(Phase::Rollback(
                                self.renderer
                                    .as_ref()
                                    .expect("renderer checked")
                                    .rollback_committed(token)
                                    .map_err(|e| e.to_string())?,
                            )));
                        }
                    };
                    // Everything fallible has completed. These window updates and
                    // the state publication run in one main-thread dispatch turn.
                    self.promote_reload_images(pending.attempt, cx);
                    self.close_contextual();
                    self.state = state;
                    self.workbench.clear_frames();
                    self.workbench.frames = pending.frames.clone();
                    self.workbench.document = pending.session.take();
                    if dropped > 0 {
                        self.warnings.push(
                            self.locale
                                .format("reload-dropped-ink", &[("count", dropped.to_string())]),
                        );
                    }
                    self.reload.bytes = None;
                    self.reload.structure = None;
                    pending.published = true;
                    self.publish(false, cx);
                    drop(token);
                    Ok(Some(Phase::Retire(
                        self.renderer
                            .as_ref()
                            .expect("renderer checked")
                            .retire_generation(pending.identity.generation)
                            .map_err(|e| e.to_string())?,
                    )))
                }
                Phase::Rollback(reply) => {
                    let Some(ack) = receive(reply) else {
                        return Ok(None);
                    };
                    match ack {
                        Ok(ack) if ack.generation == pending.identity.generation => {}
                        _ => {
                            self.reload.failed_worker = true;
                            self.reload.orphans.push(pending.generation);
                            return Err(self.locale.text("reload-recovery").into());
                        }
                    }
                    Ok(Some(Phase::Retire(
                        self.renderer
                            .as_ref()
                            .expect("renderer checked")
                            .retire_generation(pending.generation)
                            .map_err(|e| e.to_string())?,
                    )))
                }
                Phase::Abort(reply) => {
                    let Some(ack) = receive(reply) else {
                        return Ok(None);
                    };
                    match ack {
                        Ok(ack) if ack.generation == pending.generation => {}
                        Err(RenderError::NotCandidate) => {}
                        Ok(_) => {
                            return Err("Candidate abort acknowledgement identity mismatch".into());
                        }
                        Err(error) => return Err(error.to_string()),
                    }
                    Ok(Some(Phase::Retire(
                        self.renderer
                            .as_ref()
                            .expect("renderer checked")
                            .retire_generation(pending.generation)
                            .map_err(|e| e.to_string())?,
                    )))
                }
                Phase::Retire(_) => Ok(None),
            }
        })();
        if let Phase::Retire(reply) = &pending.phase
            && let Some(result) = receive(reply)
        {
            let expected = if pending.published {
                pending.identity.generation
            } else {
                pending.generation
            };
            match result {
                Ok(ack) if ack.generation == expected => {}
                other => {
                    self.error = Some(format!("Replacement retirement failed: {other:?}"));
                    self.reload.orphans.push(expected);
                }
            }
            return false;
        }
        match result {
            Ok(Some(phase)) => {
                pending.phase = phase;
                cx.notify();
            }
            Ok(None) => {}
            Err(error) => {
                if pending.published {
                    self.error = Some(format!("Replacement retirement failed: {error}"));
                    self.reload.orphans.push(pending.identity.generation);
                    return false;
                }
                if committed {
                    // The worker already switched generations. An unavailable
                    // rollback channel cannot be repaired by Abort(candidate).
                    self.reload.failed_worker = true;
                    self.reload.orphans.push(pending.generation);
                    self.error = Some(format!("{}: {error}", self.locale.text("reload-recovery")));
                    return false;
                }
                if invalid_input {
                    tracing::debug!(%error, "external PDF candidate not ready; retaining active document");
                } else {
                    self.error = Some(error);
                }
                if matches!(
                    pending.phase,
                    Phase::Rollback(_) | Phase::Abort(_) | Phase::Retire(_)
                ) {
                    return false;
                }
                pending.obsolete = true;
                match self
                    .renderer
                    .as_ref()
                    .expect("renderer checked")
                    .abort_candidate(pending.generation)
                {
                    Ok(reply) => pending.phase = Phase::Abort(reply),
                    Err(_) => return false,
                }
            }
        }
        true
    }
}
