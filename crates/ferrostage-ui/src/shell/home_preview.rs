//! Idle-only, read-only inspection on the existing PDF worker. Never commits a
//! candidate or changes presentation state, history, watches or audience pixels.
use super::*;
use ferrostage_core::BgraFrame;
use ferrostage_pdf::{DocumentMetadata, DocumentStructure, PixelSize};
use ferrostage_render::{RenderRequest, RenderResult, Reply};

#[derive(Clone, Default)]
pub(crate) struct HomePreview {
    pub path: Option<PathBuf>,
    pub token: u64,
    pub frame: Option<BgraFrame>,
    pub error: Option<&'static str>,
}

pub(super) struct PendingPreview {
    generation: DocumentGeneration,
    phase: Phase,
}
enum Phase {
    Metadata(Reply<DocumentMetadata>, DocumentStructure),
    Raster(Reply<Vec<RenderResult>>),
}
fn receive<T>(reply: &Reply<T>) -> Option<Result<T, String>> {
    match reply.try_recv() {
        Ok(result) => Some(result.map_err(|error| error.to_string())),
        Err(std::sync::mpsc::TryRecvError::Empty) => None,
        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
            Some(Err("PDF worker disconnected".into()))
        }
    }
}

impl Shell {
    pub(super) fn cancel_home_preview(&mut self) {
        drop(self.home_preview_task.take());
        if let Some(pending) = self.home_preview_pending.take()
            && let Ok(worker) = &self.renderer
        {
            let _ = worker.abort_candidate(pending.generation);
        }
        self.home_preview = HomePreview {
            token: self.home_preview.token.wrapping_add(1),
            ..Default::default()
        };
    }

    pub(super) fn select_home_preview(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        if self.state.stage() != SessionStage::Idle
            || self.busy()
            || self.modal.is_some()
            || !self
                .settings
                .settings
                .general
                .recent_files
                .iter()
                .take(3)
                .any(|recent| std::path::Path::new(recent) == path)
        {
            return;
        }
        self.cancel_home_preview();
        self.home_preview.path = Some(path.clone());
        if self.renderer.is_err() {
            self.home_preview.error = Some("import-unavailable");
            cx.notify();
            return;
        }
        let token = self.home_preview.token;
        let read = cx.background_executor().spawn(async move {
            let bytes = std::fs::read(path).map_err(|_| "home-preview-read-error")?;
            let structure =
                ferrostage_pdf::extract_structure(&bytes).map_err(|_| "home-preview-invalid")?;
            Ok::<_, &'static str>((bytes, structure))
        });
        self.home_preview_task =
            Some(cx.spawn(async move |weak, cx| {
                let result = read.await;
                let _ =
                    weak.update(cx, |shell, cx| {
                        if shell.home_preview.token != token
                            || shell.state.stage() != SessionStage::Idle
                            || shell.loading
                        {
                            return;
                        }
                        match result {
                            Ok((bytes, structure)) => {
                                let generation = DocumentGeneration(
                                    shell.next_generation.max(shell.state.generation().0 + 1),
                                );
                                shell.next_generation = generation.0 + 1;
                                match shell.renderer.as_ref().ok().and_then(|worker| {
                                    worker.prepare_candidate(generation, bytes).ok()
                                }) {
                                    Some(reply) => {
                                        shell.home_preview_pending = Some(PendingPreview {
                                            generation,
                                            phase: Phase::Metadata(reply, structure),
                                        })
                                    }
                                    None => shell.home_preview.error = Some("home-preview-invalid"),
                                }
                            }
                            Err(key) => shell.home_preview.error = Some(key),
                        }
                        cx.notify();
                    });
            }));
        cx.notify();
    }

    pub(super) fn poll_home_preview(&mut self, cx: &mut Context<Self>) {
        if self.state.stage() != SessionStage::Idle || self.loading {
            return;
        }
        let recents = &self.settings.settings.general.recent_files;
        let first = recents.first().map(PathBuf::from);
        if self.home_preview.path.as_ref().is_some_and(|path| {
            !recents
                .iter()
                .take(3)
                .any(|recent| std::path::Path::new(recent) == path)
        }) {
            self.cancel_home_preview();
        }
        if self.home_preview.path.is_none() {
            if let Some(path) = first {
                self.select_home_preview(path, cx);
            }
            return;
        }
        let Some(pending) = &mut self.home_preview_pending else {
            return;
        };
        let Ok(worker) = &self.renderer else { return };
        let result: Result<Option<BgraFrame>, String> = (|| match &pending.phase {
            Phase::Metadata(reply, structure) => {
                let Some(metadata) = receive(reply) else {
                    return Ok(None);
                };
                let session =
                    crate::workbench::DocumentSession::build(metadata?, structure.clone())?;
                let page = *session
                    .document
                    .slide_map
                    .steps()
                    .first()
                    .ok_or("no content page")?;
                let request = RenderRequest {
                    generation: pending.generation,
                    page,
                    crop: ferrostage_document::slide_crop(session.document.layout),
                    target_px: PixelSize {
                        width: 1280,
                        height: 960,
                    },
                    priority: ferrostage_render::RenderPriority::Presenter,
                    epoch: 0,
                };
                pending.phase = Phase::Raster(
                    worker
                        .preview_candidate(pending.generation, vec![request])
                        .map_err(|error| error.to_string())?,
                );
                Ok(None)
            }
            Phase::Raster(reply) => {
                let Some(result) = receive(reply) else {
                    return Ok(None);
                };
                result?
                    .into_iter()
                    .next()
                    .map(|result| Some(result.frame))
                    .ok_or("missing preview frame".into())
            }
        })();
        if matches!(result, Ok(None)) {
            return;
        }
        let _ = worker.abort_candidate(pending.generation);
        self.home_preview_pending = None;
        match result {
            Ok(frame) => self.home_preview.frame = frame,
            Err(error) => {
                tracing::warn!(%error, "home preview failed");
                self.home_preview.error = Some("home-preview-invalid");
            }
        }
        cx.notify();
    }
}
