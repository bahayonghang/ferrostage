use super::*;
use crate::{RenderPriority, RenderResult};
use ferrostage_core::{BgraFrame, NormalizedRect, PhysicalPage};
use ferrostage_pdf::{PageMetadata, PdfError, PixelSize};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

type Calls = Arc<Mutex<Vec<(DocumentGeneration, PhysicalPage)>>>;
struct Fake {
    calls: Calls,
    entered: Option<Sender<()>>,
    release: Option<Receiver<()>>,
    removed: Arc<Mutex<Vec<DocumentGeneration>>>,
}
fn metadata() -> DocumentMetadata {
    DocumentMetadata {
        pages: vec![PageMetadata {
            page: PhysicalPage(0),
            width_points: 320.0,
            height_points: 180.0,
            rotation_degrees: 0,
            label: Some("one".into()),
        }],
    }
}
impl RasterBackend for Fake {
    fn probe(&mut self, bytes: Vec<u8>) -> Result<DocumentMetadata, PdfError> {
        if bytes.is_empty() {
            Err(PdfError::Unreadable)
        } else {
            Ok(metadata())
        }
    }
    fn load(
        &mut self,
        _: DocumentGeneration,
        bytes: Vec<u8>,
    ) -> Result<DocumentMetadata, PdfError> {
        self.probe(bytes)
    }
    fn render(
        &mut self,
        generation: DocumentGeneration,
        page: PhysicalPage,
        _: NormalizedRect,
        target: PixelSize,
    ) -> Result<BgraFrame, PdfError> {
        self.calls.lock().unwrap().push((generation, page));
        if let Some(entered) = self.entered.take() {
            entered.send(()).unwrap();
            self.release.take().unwrap().recv().unwrap();
        }
        BgraFrame::new(
            target.width,
            target.height,
            target.width as usize * 4,
            vec![255; target.width as usize * target.height as usize * 4],
        )
        .map_err(|_| PdfError::InvalidBitmap)
    }
    fn remove(&mut self, generation: DocumentGeneration) {
        self.removed.lock().unwrap().push(generation);
    }
}
fn reply<T>(reply: Reply<T>) -> Result<T, RenderError> {
    reply.recv_timeout(Duration::from_secs(5)).unwrap()
}
fn req(generation: u64, page: usize, epoch: u64, priority: RenderPriority) -> RenderRequest {
    RenderRequest {
        generation: DocumentGeneration(generation),
        page: PhysicalPage(page),
        crop: NormalizedRect::FULL,
        target_px: PixelSize {
            width: 320,
            height: 180,
        },
        priority,
        epoch,
    }
}
fn event(handle: &RenderHandle) -> RenderEvent {
    handle
        .results()
        .recv_timeout(Duration::from_secs(5))
        .unwrap()
}
fn simple() -> (RenderHandle, Calls, Arc<Mutex<Vec<DocumentGeneration>>>) {
    let calls = Calls::default();
    let removed = Arc::new(Mutex::new(Vec::new()));
    let fake = Fake {
        calls: calls.clone(),
        removed: removed.clone(),
        entered: None,
        release: None,
    };
    (
        RenderHandle::with_backend(
            move || Box::new(fake),
            CacheBudgets {
                l2_bytes: 512 * 1024,
                l3_bytes: 512 * 1024,
            },
        )
        .unwrap(),
        calls,
        removed,
    )
}
fn blocked() -> (RenderHandle, Calls, Receiver<()>, Sender<()>) {
    let calls = Calls::default();
    let (entered_tx, entered) = mpsc::channel();
    let (release, release_rx) = mpsc::channel();
    let fake = Fake {
        calls: calls.clone(),
        removed: Default::default(),
        entered: Some(entered_tx),
        release: Some(release_rx),
    };
    (
        RenderHandle::with_backend(move || Box::new(fake), CacheBudgets::default()).unwrap(),
        calls,
        entered,
        release,
    )
}
#[test]
fn in_flight_is_not_preempted_but_old_epoch_is_never_published() {
    let (handle, calls, entered, release) = blocked();
    reply(handle.load_active(DocumentGeneration(1), vec![1]).unwrap()).unwrap();
    handle
        .submit(req(1, 0, 0, RenderPriority::Thumbnail))
        .unwrap();
    entered.recv_timeout(Duration::from_secs(5)).unwrap();
    for page in 1..11 {
        handle
            .submit(req(1, page, 0, RenderPriority::Thumbnail))
            .unwrap();
    }
    let epoch = handle.advance_epoch(DocumentGeneration(1), 1).unwrap();
    let p0 = handle
        .submit(req(1, 100, 1, RenderPriority::Audience))
        .unwrap();
    release.send(()).unwrap();
    reply(epoch).unwrap();
    let mut success = Vec::new();
    let mut cancelled = 0;
    for _ in 0..12 {
        let result = event(&handle);
        match result.outcome {
            Ok(_) => success.push(result.id),
            Err(RenderError::Cancelled) => cancelled += 1,
            other => panic!("{other:?}"),
        }
    }
    assert_eq!(success, vec![p0]);
    assert_eq!(cancelled, 11);
    assert_eq!(
        *calls.lock().unwrap(),
        vec![
            (DocumentGeneration(1), PhysicalPage(0)),
            (DocumentGeneration(1), PhysicalPage(100))
        ]
    );
}
#[test]
fn old_generation_inflight_cannot_publish_after_load() {
    let (handle, _, entered, release) = blocked();
    reply(handle.load_active(DocumentGeneration(1), vec![1]).unwrap()).unwrap();
    handle
        .submit(req(1, 0, 0, RenderPriority::Audience))
        .unwrap();
    entered.recv_timeout(Duration::from_secs(5)).unwrap();
    let loaded = handle.load_active(DocumentGeneration(2), vec![1]).unwrap();
    release.send(()).unwrap();
    reply(loaded).unwrap();
    assert_eq!(event(&handle).outcome.unwrap_err(), RenderError::Cancelled);
}
#[test]
fn candidate_metadata_prerender_commit_abort_and_retire_are_separate() {
    let (handle, _, removed) = simple();
    reply(handle.load_active(DocumentGeneration(1), vec![1]).unwrap()).unwrap();
    assert_eq!(
        reply(
            handle
                .prepare_candidate(DocumentGeneration(2), vec![])
                .unwrap()
        )
        .unwrap_err(),
        RenderError::Pdf(PdfError::Unreadable)
    );
    assert_eq!(
        reply(
            handle
                .prepare_candidate(DocumentGeneration(3), vec![1])
                .unwrap()
        )
        .unwrap(),
        metadata()
    );
    handle
        .submit(req(1, 0, 0, RenderPriority::Audience))
        .unwrap();
    assert!(event(&handle).outcome.is_ok());
    let token = reply(
        handle
            .pre_render_candidate(
                DocumentGeneration(3),
                vec![req(3, 0, 0, RenderPriority::Audience)],
            )
            .unwrap(),
    )
    .unwrap();
    assert_eq!(token.frames[0].request.generation, DocumentGeneration(3));
    assert_eq!(
        reply(handle.retire_generation(DocumentGeneration(3)).unwrap()).unwrap_err(),
        RenderError::GenerationInUse
    );
    reply(handle.commit_candidate(token.clone()).unwrap()).unwrap();
    assert_eq!(
        reply(handle.commit_candidate(token).unwrap()).unwrap_err(),
        RenderError::NotReady
    );
    reply(handle.retire_generation(DocumentGeneration(1)).unwrap()).unwrap();
    assert!(removed.lock().unwrap().contains(&DocumentGeneration(1)));
    reply(
        handle
            .prepare_candidate(DocumentGeneration(4), vec![1])
            .unwrap(),
    )
    .unwrap();
    reply(handle.abort_candidate(DocumentGeneration(4)).unwrap()).unwrap();
    assert_eq!(
        reply(
            handle
                .prepare_candidate(DocumentGeneration(4), vec![1])
                .unwrap()
        )
        .unwrap_err(),
        RenderError::StaleGeneration
    );
    handle
        .submit(req(3, 0, 0, RenderPriority::Audience))
        .unwrap();
    assert!(event(&handle).outcome.is_ok());
}
#[test]
fn superseded_prerender_cannot_complete_a_new_batch() {
    let (handle, calls, entered, release) = blocked();
    reply(
        handle
            .prepare_candidate(DocumentGeneration(1), vec![1])
            .unwrap(),
    )
    .unwrap();
    let first = handle
        .pre_render_candidate(
            DocumentGeneration(1),
            vec![req(1, 0, 0, RenderPriority::Audience)],
        )
        .unwrap();
    entered.recv_timeout(Duration::from_secs(5)).unwrap();
    let second = handle
        .pre_render_candidate(
            DocumentGeneration(1),
            vec![req(1, 1, 0, RenderPriority::Audience)],
        )
        .unwrap();
    release.send(()).unwrap();
    assert_eq!(reply(first).unwrap_err(), RenderError::Cancelled);
    let token = reply(second).unwrap();
    assert_eq!(token.frames.len(), 1);
    assert_eq!(token.frames[0].request.page, PhysicalPage(1));
    assert_eq!(calls.lock().unwrap().len(), 2);
}
#[test]
fn cache_hit_has_no_render_timing_and_uncached_always_renders() {
    let (handle, calls, _) = simple();
    reply(handle.load_active(DocumentGeneration(1), vec![1]).unwrap()).unwrap();
    let request = req(1, 0, 0, RenderPriority::Audience);
    handle.submit(request).unwrap();
    assert!(event(&handle).outcome.unwrap().render_duration.is_some());
    handle.submit(request).unwrap();
    let RenderResult {
        cache_hit,
        render_duration,
        ..
    } = event(&handle).outcome.unwrap();
    assert!(cache_hit);
    assert!(render_duration.is_none());
    handle.submit_uncached(request).unwrap();
    assert!(!event(&handle).outcome.unwrap().cache_hit);
    assert_eq!(calls.lock().unwrap().len(), 2);
}

#[test]
fn candidate_batch_yields_to_active_priority_and_uses_its_own_epoch() {
    let (handle, calls, entered, release) = blocked();
    reply(handle.load_active(DocumentGeneration(1), vec![1]).unwrap()).unwrap();
    reply(
        handle
            .prepare_candidate(DocumentGeneration(2), vec![1])
            .unwrap(),
    )
    .unwrap();
    handle
        .submit(req(1, 0, 0, RenderPriority::Thumbnail))
        .unwrap();
    entered.recv_timeout(Duration::from_secs(5)).unwrap();
    let candidate = handle
        .pre_render_candidate(
            DocumentGeneration(2),
            vec![
                req(2, 1, 0, RenderPriority::Thumbnail),
                req(2, 2, 0, RenderPriority::Thumbnail),
            ],
        )
        .unwrap();
    let epoch = handle.advance_epoch(DocumentGeneration(1), 1).unwrap();
    handle
        .submit(req(1, 100, 1, RenderPriority::Audience))
        .unwrap();
    release.send(()).unwrap();
    reply(epoch).unwrap();
    let ready = reply(candidate).unwrap();
    assert_eq!(ready.frames.len(), 2);
    assert_eq!(
        *calls.lock().unwrap(),
        vec![
            (DocumentGeneration(1), PhysicalPage(0)),
            (DocumentGeneration(1), PhysicalPage(100)),
            (DocumentGeneration(2), PhysicalPage(1)),
            (DocumentGeneration(2), PhysicalPage(2))
        ]
    );
    reply(handle.advance_epoch(DocumentGeneration(2), 0).unwrap()).unwrap();
    reply(handle.commit_candidate(ready).unwrap()).unwrap();
}

mod contracts;
