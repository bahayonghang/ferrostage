//! Candidate import retains the active worker document until mapping and raster succeed.
use crate::workbench::DocumentSession;
use ferrostage_core::DocumentGeneration;
use ferrostage_pdf::{DocumentMetadata, DocumentStructure};
use ferrostage_render::{CandidateReady, GenerationAck, RenderHandle, Reply};
use std::sync::mpsc::TryRecvError;

pub enum LoadPhase {
    Metadata(Reply<DocumentMetadata>),
    Raster(Reply<CandidateReady>),
    Commit(Reply<GenerationAck>),
}
pub struct PendingLoad {
    pub generation: DocumentGeneration,
    phase: LoadPhase,
    structure: Option<DocumentStructure>,
    session: Option<DocumentSession>,
}
fn receive<T>(reply: &Reply<T>) -> Option<Result<T, String>> {
    match reply.try_recv() {
        Ok(r) => Some(r.map_err(|e| e.to_string())),
        Err(TryRecvError::Empty) => None,
        Err(TryRecvError::Disconnected) => Some(Err("PDF worker disconnected".into())),
    }
}
impl PendingLoad {
    pub fn new(
        worker: &RenderHandle,
        generation: DocumentGeneration,
        bytes: Vec<u8>,
        structure: DocumentStructure,
    ) -> Result<Self, String> {
        Ok(Self {
            generation,
            phase: LoadPhase::Metadata(
                worker
                    .prepare_candidate(generation, bytes)
                    .map_err(|e| e.to_string())?,
            ),
            structure: Some(structure),
            session: None,
        })
    }
    pub fn poll(&mut self, worker: &RenderHandle) -> Option<Result<DocumentSession, String>> {
        let result: Result<Option<DocumentSession>, String> = (|| {
            match &self.phase {
                LoadPhase::Metadata(reply) => {
                    let Some(metadata) = receive(reply) else {
                        return Ok(None);
                    };
                    let session = DocumentSession::build(
                        metadata?,
                        self.structure.take().ok_or("missing candidate structure")?,
                    )?;
                    let page = session.document.slide_map.steps()[0];
                    let request = crate::workbench::request_for(
                        self.generation,
                        page,
                        ferrostage_document::slide_crop(session.document.layout),
                    );
                    self.phase = LoadPhase::Raster(
                        worker
                            .pre_render_candidate(self.generation, vec![request])
                            .map_err(|e| e.to_string())?,
                    );
                    self.session = Some(session);
                }
                LoadPhase::Raster(reply) => {
                    let Some(ready) = receive(reply) else {
                        return Ok(None);
                    };
                    self.phase = LoadPhase::Commit(
                        worker.commit_candidate(ready?).map_err(|e| e.to_string())?,
                    );
                }
                LoadPhase::Commit(reply) => {
                    let Some(ack) = receive(reply) else {
                        return Ok(None);
                    };
                    if ack?.generation != self.generation {
                        return Err("candidate commit generation mismatch".into());
                    }
                    return self
                        .session
                        .take()
                        .map(Some)
                        .ok_or("missing candidate document".into());
                }
            }
            Ok(None)
        })();
        match result {
            Ok(None) => None,
            Ok(Some(s)) => Some(Ok(s)),
            Err(e) => {
                let _ = worker.abort_candidate(self.generation);
                Some(Err(e))
            }
        }
    }
}

#[cfg(all(test, feature = "pdfium"))]
mod native_tests {
    use super::*;
    use crate::workbench::{Surface, Workbench, assemble};
    use ferrostage_core::{PresentationCommand as C, PresentationState};
    use ferrostage_pdf::{PixelSize, RuntimeConfig};
    use ferrostage_render::CacheBudgets;
    use std::time::{Duration, Instant};
    #[test]
    fn real_initial_load_notes_projection_and_failed_candidate_preserve_active() {
        let worker = RenderHandle::spawn(
            RuntimeConfig {
                library_path: std::env::var_os("FERROSTAGE_TEST_PDFIUM")
                    .expect("native UI tests require pinned PDFium")
                    .into(),
                font_directories: vec![],
            },
            CacheBudgets::default(),
        )
        .unwrap();
        let bytes =
            include_bytes!("../../../fixtures/notes-functional/pympress-notes.pdf").to_vec();
        let structure = ferrostage_pdf::extract_structure(&bytes).unwrap();
        let mut load = PendingLoad::new(
            &worker,
            DocumentGeneration(1),
            bytes.clone(),
            structure.clone(),
        )
        .unwrap();
        let deadline = Instant::now() + Duration::from_secs(20);
        let session = loop {
            if let Some(result) = load.poll(&worker) {
                break result.unwrap();
            }
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(5));
        };
        let mut state = PresentationState::idle();
        state
            .apply_at(
                C::LoadMappedDocument(session.document.descriptor(DocumentGeneration(1)).unwrap()),
                Instant::now(),
            )
            .unwrap();
        state
            .apply_at(
                C::StartPresentation {
                    audience_available: true,
                },
                Instant::now(),
            )
            .unwrap();
        let target = PixelSize {
            width: 640,
            height: 360,
        };
        let mut workbench = Workbench::default();
        workbench.document = Some(session);
        workbench.audience_target = Some(target);
        workbench.presenter_target = Some(target);
        workbench.grid = true;
        let settings = ferrostage_settings::PresenterSettings {
            grid_columns: 2,
            grid_visible_pages: 5,
            ..Default::default()
        };
        loop {
            workbench.schedule(&worker, &state, &settings).unwrap();
            assert!(workbench.poll(&worker).is_empty());
            if workbench.frame(Surface::Audience).is_some()
                && workbench.frame(Surface::Notes(0)).is_some()
                && workbench
                    .frames
                    .iter()
                    .filter(|(s, _)| matches!(s, Surface::Thumbnail(_)))
                    .count()
                    == 5
            {
                break;
            }
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(5));
        }
        let slide = workbench.frame(Surface::Audience).unwrap();
        let notes = workbench.frame(Surface::Notes(0)).unwrap();
        assert_ne!(slide.frame.pixels_bgra(), notes.frame.pixels_bgra());
        assert!(
            slide
                .frame
                .pixels_bgra()
                .as_chunks::<4>()
                .0
                .iter()
                .any(|pixel| pixel[..3] != [255; 3])
        );
        let projection = assemble(
            state.audience_snapshot(),
            &state.audience_snapshot(),
            Some(slide),
            Some(target),
            64 * 1024 * 1024,
        );
        assert_eq!(
            projection.visual.as_ref().unwrap().page.pixels_bgra(),
            slide.frame.pixels_bgra()
        );
        assert!(
            assemble(
                state.audience_snapshot(),
                &state.audience_snapshot(),
                Some(notes),
                Some(target),
                64 * 1024 * 1024
            )
            .visual
            .is_none()
        );
        let mut broken = structure;
        broken.pages.pop();
        let mut candidate =
            PendingLoad::new(&worker, DocumentGeneration(2), bytes, broken).unwrap();
        loop {
            if let Some(result) = candidate.poll(&worker) {
                assert!(result.is_err());
                break;
            }
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(5));
        }
        let request = slide.request;
        let id = worker.submit_uncached(request).unwrap();
        loop {
            let event = worker
                .results()
                .recv_timeout(Duration::from_secs(10))
                .unwrap();
            if event.id == id {
                assert!(event.outcome.is_ok());
                break;
            }
        }
    }
}
