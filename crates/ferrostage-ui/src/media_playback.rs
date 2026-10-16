//! Presenter-owned media selection. Audience receives only public slot pixels.
use ferrostage_core::{
    AudienceMediaFrame, AudienceStage, DocumentGeneration, PhysicalPage, PresentationState,
    SessionStage,
};
use ferrostage_document::{LoadedDocument, MediaRepeat, MediaSource, MediaTime};
use ferrostage_media::{Controller, Identity, MediaError, Runtime, Source, Start, Update};
use std::path::Path;
#[derive(Clone, Debug)]
pub(crate) struct SlotView {
    pub id: u64,
    pub rect: ferrostage_core::NormalizedRect,
    pub active: bool,
    pub paused: bool,
    pub progress: (u64, u64),
    pub error: Option<MediaError>,
}
#[derive(Default)]
pub(crate) struct Playback {
    controller: Option<Controller>,
    context: Option<(
        DocumentGeneration,
        PhysicalPage,
        SessionStage,
        ferrostage_core::NormalizedRect,
    )>,
    covered: bool,
    pub update: Option<Update>,
    error: Option<(u64, MediaError)>,
}
fn time(value: Option<MediaTime>) -> Result<Option<u64>, MediaError> {
    match value {
        None => Ok(None),
        Some(MediaTime::Seconds(s)) if s.is_finite() && (0.0..=86_400.).contains(&s) => {
            Ok(Some((s * 1000.).round() as u64))
        }
        _ => Err(MediaError::Unsupported),
    }
}
impl Playback {
    pub fn stop(&mut self) {
        if let Some(c) = &mut self.controller {
            c.stop();
        }
        self.update = None;
        self.error = None;
    }
    pub fn shutdown(&mut self) {
        self.stop();
        if let Some(c) = &mut self.controller {
            c.shutdown();
        }
    }
    pub fn sync(
        &mut self,
        state: &PresentationState,
        document: Option<&LoadedDocument>,
        path: Option<&Path>,
        height: u32,
        curtain: bool,
    ) {
        let context = (
            state.generation(),
            state.current_page(),
            state.stage(),
            state.slide_crop(),
        );
        let entered = self.context != Some(context);
        let covered = curtain || state.audience_snapshot().blank_mode.is_some();
        if entered || covered || state.stage() != SessionStage::Presenting {
            self.stop();
        }
        self.context = Some(context);
        self.covered = covered;
        if entered
            && !covered
            && state.stage() == SessionStage::Presenting
            && let Some(slot) = document.and_then(|d| {
                d.media.iter().find(|s| {
                    s.page == state.current_page()
                        && s.slide_rect.is_some()
                        && s.media.autostart == Some(true)
                })
            })
        {
            self.select(slot.annotation as u64, state, document, path, height);
        }
        self.update = self.controller.as_ref().and_then(Controller::latest);
    }
    pub fn select(
        &mut self,
        id: u64,
        state: &PresentationState,
        document: Option<&LoadedDocument>,
        path: Option<&Path>,
        height: u32,
    ) {
        if self.covered
            || state.stage() != SessionStage::Presenting
            || state.audience_snapshot().blank_mode.is_some()
        {
            return;
        }
        if let Some(update) = &self.update
            && update.identity.slot == id
            && update.error.is_none()
            && !update.ended
        {
            if let Some(c) = &self.controller {
                c.pause(!update.paused);
            }
            return;
        }
        self.stop();
        let result = (|| {
            let slot = document
                .and_then(|d| {
                    d.media.iter().find(|s| {
                        s.page == state.current_page()
                            && s.annotation as u64 == id
                            && s.slide_rect.is_some()
                    })
                })
                .ok_or(MediaError::Unsupported)?;
            let source = match &slot.media.source {
                Some(MediaSource::External(value)) => {
                    let value = Path::new(value);
                    Source::External(if value.is_absolute() {
                        value.to_path_buf()
                    } else {
                        path.and_then(Path::parent)
                            .ok_or(MediaError::SourceUnavailable)?
                            .join(value)
                    })
                }
                Some(MediaSource::Embedded { name, bytes }) => Source::Embedded {
                    name: name.clone().unwrap_or_else(|| "media.bin".into()),
                    bytes: bytes.as_slice().into(),
                },
                None => return Err(MediaError::SourceUnavailable),
            };
            let plays = match slot.media.repeat {
                Some(MediaRepeat::Forever) => Some(0),
                Some(MediaRepeat::Count(n)) if n.is_finite() && n >= 1. && n <= u32::MAX as f64 => {
                    Some(n.ceil() as u32)
                }
                Some(_) => return Err(MediaError::Unsupported),
                None => None,
            };
            if self.controller.is_none() {
                self.controller = Some(Controller::new(Runtime::default())?);
            }
            if let Some(controller) = &mut self.controller {
                controller.start(Start {
                    identity: Identity {
                        generation: state.generation(),
                        page: state.current_page(),
                        slot: id,
                        epoch: 0,
                    },
                    source,
                    max_height: height,
                    source_crop: slot.source_crop.ok_or(MediaError::Unsupported)?,
                    aspect_ratio: slot.aspect_ratio,
                    start_ms: time(slot.media.start)?.unwrap_or(0),
                    duration_ms: time(slot.media.duration)?,
                    plays,
                })?;
            }
            Ok(())
        })();
        if let Err(error) = result {
            self.error = Some((id, error));
        }
    }
    pub fn frames(
        &self,
        state: &PresentationState,
        document: Option<&LoadedDocument>,
    ) -> Vec<AudienceMediaFrame> {
        let snapshot = state.audience_snapshot();
        if self.covered || snapshot.stage != AudienceStage::Live || snapshot.blank_mode.is_some() {
            return vec![];
        }
        let Some(update) = &self.update else {
            return vec![];
        };
        if update.identity.generation != state.generation()
            || update.identity.page != state.current_page()
            || update.error.is_some()
        {
            return vec![];
        }
        let Some(frame) = &update.frame else {
            return vec![];
        };
        if frame.identity != update.identity
            || self.controller.as_ref().and_then(Controller::identity) != Some(frame.identity)
        {
            return vec![];
        }
        let Some(rect) = document
            .and_then(|d| {
                d.media.iter().find(|s| {
                    s.page == state.current_page() && s.annotation as u64 == frame.identity.slot
                })
            })
            .and_then(|s| s.slide_rect)
        else {
            return vec![];
        };
        AudienceMediaFrame::new(frame.identity.epoch, rect, frame.pixels.clone())
            .into_iter()
            .collect()
    }
    pub fn slots(
        &self,
        state: &PresentationState,
        document: Option<&LoadedDocument>,
    ) -> Vec<SlotView> {
        document
            .into_iter()
            .flat_map(|d| &d.media)
            .filter(|s| s.page == state.current_page())
            .filter_map(|s| {
                let id = s.annotation as u64;
                let update = self.update.as_ref().filter(|u| u.identity.slot == id);
                Some(SlotView {
                    id,
                    rect: s.slide_rect?,
                    active: update.is_some(),
                    paused: update.is_none_or(|u| u.paused || u.ended),
                    progress: update.map_or((0, 0), |u| (u.position_ms, u.duration_ms)),
                    error: update
                        .and_then(|u| u.error)
                        .or(self.error.filter(|(slot, _)| *slot == id).map(|(_, e)| e)),
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrostage_core::{BlankMode, PresentationCommand as C};
    use ferrostage_document::{DocumentOverrides, NotesLayout};
    use std::time::{Duration, Instant};
    fn fixture(
        name: &str,
        layout: Option<NotesLayout>,
    ) -> (std::path::PathBuf, LoadedDocument, PresentationState) {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/media")
            .join(name);
        let structure =
            ferrostage_pdf::structure::extract_structure(&std::fs::read(&path).unwrap()).unwrap();
        let metadata = ferrostage_pdf::DocumentMetadata {
            pages: structure
                .pages
                .iter()
                .map(|p| {
                    let (w, h) = p.geometry.visible_size();
                    ferrostage_pdf::PageMetadata {
                        page: p.page,
                        width_points: w,
                        height_points: h,
                        rotation_degrees: 0,
                        label: None,
                    }
                })
                .collect(),
        };
        let document = LoadedDocument::build(
            &metadata,
            &structure,
            DocumentOverrides {
                layout,
                ..Default::default()
            },
        )
        .unwrap();
        let mut state = PresentationState::idle();
        state
            .apply_at(
                C::LoadMappedDocument(document.descriptor(DocumentGeneration(1)).unwrap()),
                Instant::now(),
            )
            .unwrap();
        (path, document, state)
    }
    fn wait(
        playback: &mut Playback,
        state: &PresentationState,
        document: &LoadedDocument,
        path: &Path,
    ) -> Vec<AudienceMediaFrame> {
        let deadline = Instant::now() + Duration::from_secs(4);
        loop {
            playback.sync(state, Some(document), Some(path), 1080, false);
            let frames = playback.frames(state, Some(document));
            if !frames.is_empty() {
                return frames;
            }
            assert!(Instant::now() < deadline, "{:?}", playback.update);
            std::thread::sleep(Duration::from_millis(5));
        }
    }
    #[test]
    fn real_public_crop_ready_blank_notes_page_generation_and_close_isolate_media() {
        let (path, document, mut state) = fixture("media-crop-smoke.pdf", Some(NotesLayout::Right));
        let mut playback = Playback {
            controller: Some(
                Controller::new(Runtime {
                    silent: true,
                    ..Default::default()
                })
                .unwrap(),
            ),
            ..Default::default()
        };
        playback.sync(&state, Some(&document), Some(&path), 1080, false);
        assert!(playback.controller.as_ref().unwrap().identity().is_none());
        state
            .apply_at(
                C::StartPresentation {
                    audience_available: true,
                },
                Instant::now(),
            )
            .unwrap();
        let frames = wait(&mut playback, &state, &document, &path);
        assert_eq!(frames.len(), 1);
        let pixels = frames[0].pixels();
        assert_eq!((pixels.width(), pixels.height()), (80, 100));
        assert!(
            pixels
                .pixels_bgra()
                .as_chunks::<4>()
                .0
                .iter()
                .all(|p| p[2] > p[1]),
            "private green/cyan never projected"
        );
        let shared = &playback
            .update
            .as_ref()
            .unwrap()
            .frame
            .as_ref()
            .unwrap()
            .pixels;
        assert_eq!(shared.pixels_bgra().as_ptr(), pixels.pixels_bgra().as_ptr());
        let first = frames[0].slot_id();
        state
            .apply_at(C::ToggleBlank(BlankMode::White), Instant::now())
            .unwrap();
        playback.sync(&state, Some(&document), Some(&path), 1080, false);
        assert!(playback.frames(&state, Some(&document)).is_empty());
        state
            .apply_at(C::ToggleBlank(BlankMode::White), Instant::now())
            .unwrap();
        playback.sync(&state, Some(&document), Some(&path), 1080, false);
        assert!(playback.controller.as_ref().unwrap().identity().is_none());
        playback.select(0, &state, Some(&document), Some(&path), 1080);
        assert!(wait(&mut playback, &state, &document, &path)[0].slot_id() > first);
        state.apply_at(C::NextStep, Instant::now()).unwrap();
        playback.sync(&state, Some(&document), Some(&path), 1080, false);
        assert!(
            playback.controller.as_ref().unwrap().identity().is_none(),
            "notes-only audio must never start"
        );
        state.apply_at(C::PreviousStep, Instant::now()).unwrap();
        let before = wait(&mut playback, &state, &document, &path)[0].slot_id();
        state
            .commit_reload(
                document.descriptor(DocumentGeneration(2)).unwrap(),
                PhysicalPage(0),
                &[],
            )
            .unwrap();
        let after = wait(&mut playback, &state, &document, &path)[0].slot_id();
        assert!(after > before);
        state.apply_at(C::CloseDocument, Instant::now()).unwrap();
        playback.sync(&state, None, None, 1080, false);
        assert!(playback.frames(&state, None).is_empty());
        assert!(playback.controller.as_ref().unwrap().identity().is_none());
        playback.shutdown();
    }
    #[test]
    fn successful_same_generation_notes_reconfigure_invalidates_old_public_pixels() {
        let (path, document, mut state) = fixture("media-crop-smoke.pdf", Some(NotesLayout::None));
        let (_, restricted, _) = fixture("media-crop-smoke.pdf", Some(NotesLayout::Right));
        let mut playback = Playback {
            controller: Some(
                Controller::new(Runtime {
                    silent: true,
                    ..Default::default()
                })
                .unwrap(),
            ),
            ..Default::default()
        };
        state
            .apply_at(
                C::StartPresentation {
                    audience_available: true,
                },
                Instant::now(),
            )
            .unwrap();
        let old = wait(&mut playback, &state, &document, &path)[0].slot_id();
        state
            .apply_at(
                C::ReconfigureDocument(restricted.descriptor(state.generation()).unwrap()),
                Instant::now(),
            )
            .unwrap();
        playback.sync(&state, Some(&restricted), Some(&path), 1080, false);
        assert!(playback.frames(&state, Some(&restricted)).is_empty());
        let public = wait(&mut playback, &state, &restricted, &path);
        assert!(public[0].slot_id() > old);
        assert!(
            public[0]
                .pixels()
                .pixels_bgra()
                .as_chunks::<4>()
                .0
                .iter()
                .all(|p| p[2] > p[1])
        );
        playback.shutdown();
    }
    #[test]
    fn original_beamer_movie_uses_actual_generated_video_and_parameters() {
        let (path, document, _) = fixture("movie.pdf", None);
        let slot = &document.media[0];
        assert_eq!(slot.media.start, Some(MediaTime::Seconds(2.)));
        assert_eq!(slot.media.duration, Some(MediaTime::Seconds(8.)));
        assert_eq!(slot.media.repeat, Some(MediaRepeat::Forever));
        assert_eq!(slot.media.autostart, Some(true));
        let Some(MediaSource::External(relative)) = &slot.media.source else {
            panic!()
        };
        assert!(
            std::fs::metadata(path.parent().unwrap().join(relative))
                .unwrap()
                .len()
                > 500_000
        );
    }
}
