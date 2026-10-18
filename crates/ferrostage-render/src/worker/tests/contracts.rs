use super::*;

#[test]
fn readiness_from_a_stopped_worker_cannot_commit_a_replacement_worker() {
    let (first, _, _) = simple();
    reply(
        first
            .prepare_candidate(DocumentGeneration(1), vec![1])
            .unwrap(),
    )
    .unwrap();
    let old_ready = reply(
        first
            .pre_render_candidate(
                DocumentGeneration(1),
                vec![req(1, 0, 0, RenderPriority::Audience)],
            )
            .unwrap(),
    )
    .unwrap();
    first.shutdown().unwrap();

    let (replacement, _, _) = simple();
    reply(
        replacement
            .prepare_candidate(DocumentGeneration(1), vec![2])
            .unwrap(),
    )
    .unwrap();
    let current_ready = reply(
        replacement
            .pre_render_candidate(
                DocumentGeneration(1),
                vec![req(1, 1, 0, RenderPriority::Audience)],
            )
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        reply(replacement.commit_candidate(old_ready).unwrap()).unwrap_err(),
        RenderError::NotReady
    );
    reply(replacement.commit_candidate(current_ready).unwrap()).unwrap();
}

#[test]
fn issued_readiness_expires_on_epoch_reprepare_and_abort() {
    let (handle, _, _) = simple();
    reply(handle.load_active(DocumentGeneration(1), vec![1]).unwrap()).unwrap();
    reply(
        handle
            .prepare_candidate(DocumentGeneration(2), vec![1])
            .unwrap(),
    )
    .unwrap();
    let ready = |epoch| {
        reply(
            handle
                .pre_render_candidate(
                    DocumentGeneration(2),
                    vec![req(2, 0, epoch, RenderPriority::Audience)],
                )
                .unwrap(),
        )
        .unwrap()
    };
    let epoch_zero = ready(0);
    reply(handle.advance_epoch(DocumentGeneration(2), 1).unwrap()).unwrap();
    assert_eq!(
        reply(handle.commit_candidate(epoch_zero).unwrap()).unwrap_err(),
        RenderError::NotReady
    );
    let superseded = ready(1);
    let current = ready(1);
    assert_eq!(
        reply(handle.commit_candidate(superseded).unwrap()).unwrap_err(),
        RenderError::NotReady
    );
    reply(handle.abort_candidate(DocumentGeneration(2)).unwrap()).unwrap();
    assert_eq!(
        reply(handle.commit_candidate(current).unwrap()).unwrap_err(),
        RenderError::NotReady
    );
    handle
        .submit(req(1, 0, 0, RenderPriority::Audience))
        .unwrap();
    assert!(event(&handle).outcome.is_ok());
}

#[test]
fn failed_candidate_render_releases_only_candidate_resources() {
    let (handle, _, removed) = simple();
    reply(handle.load_active(DocumentGeneration(1), vec![1]).unwrap()).unwrap();
    reply(
        handle
            .prepare_candidate(DocumentGeneration(2), vec![1])
            .unwrap(),
    )
    .unwrap();
    let invalid = RenderRequest {
        crop: NormalizedRect {
            w: 0.0,
            ..NormalizedRect::FULL
        },
        ..req(2, 0, 0, RenderPriority::Audience)
    };
    assert_eq!(
        reply(
            handle
                .pre_render_candidate(DocumentGeneration(2), vec![invalid])
                .unwrap()
        )
        .unwrap_err(),
        RenderError::Pdf(PdfError::InvalidCrop)
    );
    // This reply is queued after the failure, and therefore fences its cleanup.
    assert_eq!(
        reply(
            handle
                .pre_render_candidate(
                    DocumentGeneration(2),
                    vec![req(2, 0, 0, RenderPriority::Audience)]
                )
                .unwrap()
        )
        .unwrap_err(),
        RenderError::NotCandidate
    );
    assert_eq!(*removed.lock().unwrap(), vec![DocumentGeneration(2)]);
    handle
        .submit(req(1, 0, 0, RenderPriority::Audience))
        .unwrap();
    assert!(event(&handle).outcome.is_ok());
}

#[test]
fn cache_identity_includes_pixels_but_replies_echo_new_epoch_and_priority() {
    let (handle, calls, _) = simple();
    reply(handle.load_active(DocumentGeneration(1), vec![1]).unwrap()).unwrap();
    let initial = req(1, 0, 0, RenderPriority::Audience);
    let submit = |request| {
        let id = handle.submit(request).unwrap();
        let event = event(&handle);
        assert_eq!((event.id, event.request), (id, request));
        let result = event.outcome.unwrap();
        assert_eq!((result.id, result.request), (id, request));
        result.cache_hit
    };
    assert!(!submit(initial));
    reply(handle.advance_epoch(DocumentGeneration(1), 1).unwrap()).unwrap();
    let next = RenderRequest {
        epoch: 1,
        priority: RenderPriority::Presenter,
        ..initial
    };
    assert!(submit(next));
    for request in [
        RenderRequest {
            crop: NormalizedRect::LEFT_HALF,
            ..next
        },
        RenderRequest {
            target_px: PixelSize {
                width: 160,
                height: 180,
            },
            ..next
        },
        RenderRequest {
            page: PhysicalPage(1),
            ..next
        },
        RenderRequest {
            priority: RenderPriority::Thumbnail,
            ..next
        },
    ] {
        assert!(!submit(request));
    }
    assert_eq!(calls.lock().unwrap().len(), 5);
}
