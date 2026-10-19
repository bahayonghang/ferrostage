use super::*;
use ferrostage_core::{
    AudiencePlayback, AudienceVisual, BlankMode, DocumentGeneration, PhysicalPage,
    PresentationState,
};
use gpui::{AppContext, PlatformAtlas};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

fn snapshot(
    page: usize,
    navigation: u64,
    style: Option<TransitionStyle>,
) -> AudienceProjectionSnapshot {
    let mut s = PresentationState::new(10).unwrap().audience_snapshot();
    s.stage = AudienceStage::Live;
    s.generation = DocumentGeneration(1);
    s.current_page = PhysicalPage(page);
    s.playback = Some(AudiencePlayback {
        identity: PlaybackIdentity {
            generation: s.generation,
            navigation,
            page: s.current_page,
        },
        transition: style.map(|style| PageTransition {
            style,
            ..Default::default()
        }),
    });
    s.visual = Some(AudienceVisual {
        texture_budget_bytes: 4096,
        page: BgraFrame::new(8, 4, 32, vec![(page as u8) * 20; 128]).unwrap(),
        media: vec![],
    });
    s
}

#[gpui::test]
fn blank_before_deferred_delivery_allows_same_identity_to_acknowledge_again(
    cx: &mut gpui::TestAppContext,
) {
    let projection = cx.new(|_| snapshot(0, 1, None));
    let p = projection.clone();
    let audience = cx.new(|cx| Audience::new(p, cx));
    let events = Rc::new(RefCell::new(Vec::new()));
    let recorded = events.clone();
    audience.update(cx, |audience, cx| {
        cx.subscribe_self(move |_, event: &PlaybackEvent, _| recorded.borrow_mut().push(*event))
            .detach();
        let snapshot = audience.snapshot.read(cx).clone();
        audience.playback.borrow_mut().identity = snapshot.playback.map(|p| p.identity);
        audience
            .playback
            .borrow_mut()
            .painted(&snapshot, true, cx.weak_entity(), cx);
        // Publish blank before the queued acknowledgements are delivered.
        audience
            .snapshot
            .update(cx, |s, _| s.blank_mode = Some(BlankMode::Black));
    });
    assert!(events.borrow().is_empty());
    audience.update(cx, |audience, cx| {
        audience.snapshot.update(cx, |s, _| s.blank_mode = None);
        let snapshot = audience.snapshot.read(cx).clone();
        // Two paints queued before delivery must still emit each phase only once.
        for _ in 0..2 {
            audience
                .playback
                .borrow_mut()
                .painted(&snapshot, true, cx.weak_entity(), cx);
        }
    });
    assert_eq!(events.borrow().len(), 2);
    assert_eq!(events.borrow()[0].phase, PlaybackPhase::FirstVisible);
    assert_eq!(events.borrow()[1].phase, PlaybackPhase::Settled);
}

#[gpui::test]
fn identity_cancel_budget_and_repeated_publication_keep_one_effect(cx: &mut gpui::TestAppContext) {
    let window = cx.add_empty_window();
    let mut cache = TextureCache::default();
    let mut playback = Playback::default();
    window.update(|window, cx| {
        let first = snapshot(0, 1, None);
        let frame = first.visual.as_ref().unwrap().page.clone();
        playback.sync(
            &first,
            &mut cache,
            &mut vec![(Surface::Page, frame.clone())],
            cx,
        );
        cache.sync(
            &[(Surface::Page, frame)],
            &[Surface::Page],
            4096,
            window,
            cx,
        );
        assert!(cache.paint(Surface::Page, Some(&first), window, cx, |_, _, _, _| Ok(())));
        let next = snapshot(1, 2, Some(TransitionStyle::Fly));
        let mut frames = vec![(Surface::Page, next.visual.as_ref().unwrap().page.clone())];
        playback.sync(&next, &mut cache, &mut frames, cx);
        assert!(playback.active());
        assert_eq!(frames.len(), 3);
        let diff = playback.difference.as_ref().unwrap().pixels_bgra().as_ptr();
        let started = Instant::now();
        playback.started = Some(started);
        playback.sync(&next, &mut cache, &mut vec![], cx);
        assert_eq!(playback.started, Some(started));
        assert_eq!(
            playback.difference.as_ref().unwrap().pixels_bgra().as_ptr(),
            diff
        );
        let mut blank = next.clone();
        blank.blank_mode = Some(BlankMode::Black);
        playback.sync(&blank, &mut cache, &mut vec![], cx);
        assert!(!playback.active());
        assert!(playback.difference.is_none());
        assert!(cache.frame(Surface::Outgoing).is_none());
        assert!(
            cache.frame(Surface::Page).is_some(),
            "failed incoming upload retains old public pixels"
        );
        let mut limited = snapshot(2, 3, Some(TransitionStyle::Fade));
        limited.visual.as_mut().unwrap().texture_budget_bytes = 256;
        playback.sync(&limited, &mut cache, &mut vec![], cx);
        assert!(!playback.active());
        assert!(playback.error().unwrap().contains("budget"));
        let mut generation = snapshot(3, 4, Some(TransitionStyle::Push));
        generation.generation = DocumentGeneration(2);
        generation.playback.as_mut().unwrap().identity.generation = generation.generation;
        playback.sync(&generation, &mut cache, &mut vec![], cx);
        assert!(!playback.active());
    });
}

#[derive(Default)]
struct SceneEvidence {
    sprites: Vec<(f32, f32, f32, f32, f32)>,
}
#[derive(Default)]
struct Atlas {
    inner: gpui::HeadlessAtlas,
    fail: AtomicBool,
}
impl PlatformAtlas for Atlas {
    fn get_or_insert_with<'a>(
        &self,
        key: gpui::AtlasKey,
        build: &mut dyn FnMut() -> gpui::Result<
            Option<(gpui::Size<gpui::DevicePixels>, std::borrow::Cow<'a, [u8]>)>,
        >,
    ) -> gpui::Result<Option<gpui::AtlasTile>> {
        if self.fail.load(Ordering::SeqCst)
            && matches!(key, gpui::AtlasKey::Image(_))
            && !self.inner.contains(&key)
        {
            return Err(std::io::Error::other("transition injected atlas failure").into());
        }
        self.inner.get_or_insert_with(key, build)
    }
    fn remove(&self, key: &gpui::AtlasKey) {
        self.inner.remove(key);
    }
    fn contains(&self, key: &gpui::AtlasKey) -> bool {
        self.inner.contains(key)
    }
}
struct Renderer {
    atlas: Arc<Atlas>,
    evidence: Arc<Mutex<SceneEvidence>>,
}
impl gpui::PlatformHeadlessRenderer for Renderer {
    fn sprite_atlas(&self) -> Arc<dyn PlatformAtlas> {
        self.atlas.clone()
    }
    fn render_scene(
        &mut self,
        scene: &gpui::Scene,
        _: gpui::Size<gpui::DevicePixels>,
    ) -> gpui::Result<()> {
        self.evidence.lock().unwrap().sprites = scene
            .polychrome_sprites
            .iter()
            .map(|s| {
                (
                    s.bounds.origin.x.0,
                    s.bounds.origin.y.0,
                    s.bounds.size.width.0,
                    s.bounds.size.height.0,
                    s.opacity,
                )
            })
            .collect();
        Ok(())
    }
    fn render_scene_to_image(
        &mut self,
        scene: &gpui::Scene,
        size: gpui::Size<gpui::DevicePixels>,
    ) -> gpui::Result<image::RgbaImage> {
        self.render_scene(scene, size)?;
        Ok(image::RgbaImage::new(1, 1))
    }
}

#[test]
fn actual_gpui_paint_reuses_atlas_images_and_applies_opacity_clip_motion_and_cleanup() {
    let evidence = Arc::new(Mutex::new(SceneEvidence::default()));
    let e = evidence.clone();
    let atlas = Arc::new(Atlas::default());
    let a = atlas.clone();
    let mut cx = gpui::HeadlessAppContext::with_platform(
        Arc::new(gpui::NoopTextSystem),
        Arc::new(()),
        move || {
            Ok(Some(Box::new(Renderer {
                atlas: a.clone(),
                evidence: e.clone(),
            })))
        },
    );
    let projection = cx.new(|_| snapshot(0, 1, None));
    let p = projection.clone();
    let window = cx
        .open_window(size(gpui::px(800.), gpui::px(400.)), |_, cx| {
            cx.new(|cx| Audience::new(p, cx))
        })
        .unwrap();
    let paint = |cx: &mut gpui::HeadlessAppContext| {
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            window.render_to_image().unwrap();
        })
        .unwrap();
    };
    let events = Rc::new(RefCell::new(Vec::new()));
    let recorded = events.clone();
    window
        .update(&mut cx, |_, _, cx| {
            cx.subscribe_self(move |_, event: &PlaybackEvent, _| recorded.borrow_mut().push(*event))
                .detach()
        })
        .unwrap();
    paint(&mut cx);
    for (index, style) in [
        TransitionStyle::Fade,
        TransitionStyle::Split,
        TransitionStyle::Blinds,
        TransitionStyle::Box,
        TransitionStyle::Wipe,
        TransitionStyle::Dissolve,
        TransitionStyle::Glitter,
        TransitionStyle::Push,
        TransitionStyle::Cover,
        TransitionStyle::Uncover,
        TransitionStyle::Fly,
    ]
    .into_iter()
    .enumerate()
    {
        projection.update(&mut cx, |p, cx| {
            *p = snapshot(index + 1, index as u64 + 2, Some(style));
            cx.notify();
        });
        paint(&mut cx);
        window
            .update(&mut cx, |audience, _, _| {
                let mut p = audience.playback.borrow_mut();
                assert!(p.ready);
                p.started = Some(Instant::now() - std::time::Duration::from_millis(500));
            })
            .unwrap();
        paint(&mut cx);
        let mid = evidence.lock().unwrap().sprites.clone();
        assert!(mid.len() >= 2, "{style:?}: {mid:?}");
        if style == TransitionStyle::Fade {
            assert!((mid[1].4 - 0.5).abs() < 0.1);
        }
        if matches!(style, TransitionStyle::Dissolve | TransitionStyle::Glitter) {
            assert!(mid.len() > 100);
            assert!(mid.iter().all(|s| s.4 == 1.));
        }
        window
            .update(&mut cx, |audience, _, _| {
                audience.playback.borrow_mut().started =
                    Some(Instant::now() - std::time::Duration::from_secs(2));
            })
            .unwrap();
        paint(&mut cx);
        paint(&mut cx);
        let seen: Vec<_> = events
            .borrow()
            .iter()
            .copied()
            .filter(|event| event.identity.navigation == index as u64 + 2)
            .collect();
        assert_eq!(
            seen.len(),
            2,
            "exactly one first-visible and settled per navigation"
        );
        assert_eq!(seen[0].phase, PlaybackPhase::FirstVisible);
        assert_eq!(seen[1].phase, PlaybackPhase::Settled);
        assert!(seen[1].at >= seen[0].at);
        assert_eq!(evidence.lock().unwrap().sprites.len(), 1);
        window
            .update(&mut cx, |audience, _, _| {
                assert!(!audience.playback.borrow().active());
                assert_eq!(audience.textures.borrow().bytes(), 128);
            })
            .unwrap();
    }
    atlas.fail.store(true, Ordering::SeqCst);
    projection.update(&mut cx, |p, cx| {
        *p = snapshot(0, 99, Some(TransitionStyle::Fade));
        cx.notify();
    });
    paint(&mut cx);
    assert_eq!(
        evidence.lock().unwrap().sprites.len(),
        1,
        "failed upload keeps the old public frame visible"
    );
    window
        .update(&mut cx, |audience, _, _| {
            assert!(
                !audience.playback.borrow().first_visible,
                "failed destination never starts its dwell timer"
            );
            assert!(!audience.playback.borrow().active());
            assert!(
                audience
                    .playback
                    .borrow()
                    .error()
                    .unwrap()
                    .contains("atlas failure")
            );
        })
        .unwrap();
    paint(&mut cx);
    assert_eq!(evidence.lock().unwrap().sprites.len(), 1);
    assert!(
        !events
            .borrow()
            .iter()
            .any(|event| event.identity.navigation == 99)
    );
}
