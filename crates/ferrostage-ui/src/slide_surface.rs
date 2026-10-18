//! Shared letterbox transform and unthemed PDF/annotation paint path.
use ferrostage_core::{AudienceProjectionSnapshot, BgraFrame};
use gpui::{Bounds, PathBuilder, Pixels, Point, RenderImage, Window, point, px, rgba, size};
use std::sync::Arc;
/// Called only from a normal canvas paint phase. The image bounds are nonempty
/// so atlas insertion runs, while the empty scene mask prevents visible pixels.
pub(crate) fn preupload(
    frame: &BgraFrame,
    image: Arc<RenderImage>,
    window: &mut Window,
) -> Result<(), String> {
    let bounds = Bounds::new(
        point(px(0.), px(0.)),
        size(px(frame.width() as f32), px(frame.height() as f32)),
    );
    window.with_content_mask(Some(gpui::ContentMask::default()), |window| {
        window
            .paint_image(bounds, bounds, Default::default(), image, 0, false)
            .map_err(|e| e.to_string())
    })
}
pub(crate) fn fit(bounds: Bounds<Pixels>, width: u32, height: u32) -> Bounds<Pixels> {
    let scale = (f32::from(bounds.size.width) / width as f32)
        .min(f32::from(bounds.size.height) / height as f32);
    let fitted = size(px(width as f32 * scale), px(height as f32 * scale));
    Bounds::new(
        point(
            bounds.origin.x + (bounds.size.width - fitted.width) * 0.5,
            bounds.origin.y + (bounds.size.height - fitted.height) * 0.5,
        ),
        fitted,
    )
}
pub(crate) fn normalize(bounds: Bounds<Pixels>, position: Point<Pixels>) -> Option<(f32, f32)> {
    if bounds.size.width <= px(0.) || bounds.size.height <= px(0.) {
        return None;
    }
    let u = (position.x - bounds.origin.x) / bounds.size.width;
    let v = (position.y - bounds.origin.y) / bounds.size.height;
    ((0.0..=1.0).contains(&u) && (0.0..=1.0).contains(&v)).then_some((u, v))
}
fn color(bgra: [u8; 4]) -> gpui::Rgba {
    rgba(u32::from_be_bytes([bgra[2], bgra[1], bgra[0], bgra[3]]))
}
pub(crate) fn paint(
    bounds: Bounds<Pixels>,
    frame: &BgraFrame,
    image: Arc<RenderImage>,
    overlay: Option<&AudienceProjectionSnapshot>,
    window: &mut Window,
) -> Result<(), String> {
    let rect = fit(bounds, frame.width(), frame.height());
    window
        .paint_image(rect, rect, Default::default(), image, 0, false)
        .map_err(|error| error.to_string())?;
    paint_overlay(rect, overlay, window);
    Ok(())
}
pub(crate) fn paint_overlay(
    rect: Bounds<Pixels>,
    overlay: Option<&AudienceProjectionSnapshot>,
    window: &mut Window,
) {
    let Some(overlay) = overlay else {
        return;
    };
    let transform = |(u, v): (f32, f32)| {
        point(
            rect.origin.x + rect.size.width * u,
            rect.origin.y + rect.size.height * v,
        )
    };
    for stroke in overlay.strokes.iter() {
        let Some(first) = stroke.points.first() else {
            continue;
        };
        let mut path = PathBuilder::stroke(rect.size.width * stroke.width_norm);
        path.move_to(transform(*first));
        if stroke.points.len() == 1 {
            let p = transform(*first);
            path.line_to(point(p.x + px(0.1), p.y));
        }
        for point in stroke.points.iter().skip(1) {
            path.line_to(transform(*point));
        }
        if let Ok(path) = path.build() {
            window.paint_path(path, color(stroke.color_bgra));
        }
    }
    if let Some(position) = overlay.laser_pos_norm {
        let p = transform(position);
        let radius = rect.size.width * overlay.laser_style.radius_norm;
        let bounds = Bounds::new(
            point(p.x - radius, p.y - radius),
            size(radius * 2., radius * 2.),
        );
        window.paint_quad(
            gpui::fill(bounds, color(overlay.laser_style.color_bgra)).corner_radii(radius),
        );
    }
}

pub(crate) struct MediaLayers<'a> {
    pub frames: &'a [ferrostage_core::AudienceMediaFrame],
    pub placeholders: &'a [(ferrostage_core::NormalizedRect, &'static str)],
}
/// Commit the PDF first; draw media only against matching complete page identity.
/// All image slots share one accounting cache. Ink/laser are always the top layer.
pub(crate) fn composite<K: Copy + PartialEq>(
    cache: &mut crate::texture_cache::TextureCache<K>,
    surface: (K, Bounds<Pixels>),
    snapshot: Option<&AudienceProjectionSnapshot>,
    media: MediaLayers<'_>,
    media_key: impl Fn(u64) -> K,
    window: &mut Window,
    cx: &mut gpui::App,
) -> bool {
    let (key, bounds) = surface;
    let mut drawn = None;
    let current = cache.paint(
        key,
        snapshot,
        window,
        cx,
        |frame, image, overlay, window| {
            paint(bounds, frame, image, None, window)?;
            drawn = Some((fit(bounds, frame.width(), frame.height()), overlay.cloned()));
            Ok(())
        },
    );
    if let Some((rect, overlay)) = drawn {
        let matching = overlay.as_ref().zip(snapshot).is_some_and(|(old, new)| {
            old.generation == new.generation
                && old.current_page == new.current_page
                && old.slide_crop == new.slide_crop
        });
        if matching {
            for frame in media.frames {
                let r = frame.rect_norm();
                let bounds = Bounds::new(
                    point(
                        rect.origin.x + rect.size.width * r.x,
                        rect.origin.y + rect.size.height * r.y,
                    ),
                    size(rect.size.width * r.w, rect.size.height * r.h),
                );
                cache.paint(
                    media_key(frame.slot_id()),
                    None,
                    window,
                    cx,
                    |pixels, image, _, window| paint(bounds, pixels, image, None, window),
                );
            }
            for (r, text) in media.placeholders {
                let bounds = Bounds::new(
                    point(
                        rect.origin.x + rect.size.width * r.x,
                        rect.origin.y + rect.size.height * r.y,
                    ),
                    size(rect.size.width * r.w, rect.size.height * r.h),
                );
                window.with_content_mask(Some(gpui::ContentMask { bounds }), |window| {
                    window.paint_quad(gpui::fill(bounds, rgba(0x202020dd)));
                    let run = gpui::TextRun {
                        len: text.len(),
                        font: gpui::font("Segoe UI"),
                        color: gpui::rgb(0xffffff).into(),
                        ..Default::default()
                    };
                    let line =
                        window
                            .text_system()
                            .shape_line((*text).into(), px(14.), &[run], None);
                    if let Err(error) = line.paint(
                        bounds.origin,
                        px(20.),
                        gpui::TextAlign::Center,
                        Some(bounds.size.width),
                        window,
                        cx,
                    ) {
                        tracing::debug!(%error,"media placeholder text paint");
                    }
                });
            }
        }
        paint_overlay(rect, overlay.as_ref(), window);
    }
    current
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn letterbox_inverse_ignores_bars_and_roundtrips_at_4k() {
        let rect = fit(
            Bounds::new(point(px(0.), px(0.)), size(px(1000.), px(1000.))),
            1920,
            1080,
        );
        assert!(normalize(rect, point(px(500.), px(10.))).is_none());
        assert_eq!(normalize(rect, point(px(500.), px(500.))), Some((0.5, 0.5)));
        let audience = fit(
            Bounds::new(point(px(0.), px(0.)), size(px(3840.), px(2160.))),
            1920,
            1080,
        );
        assert_eq!(
            normalize(audience, point(px(960.), px(1620.))),
            Some((0.25, 0.75))
        );
    }
}

#[cfg(all(test, feature = "test-support"))]
mod staging_tests {
    use super::*;
    use gpui::{AppContext, PlatformAtlas, prelude::*};
    use std::sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    };
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
                return Err(std::io::Error::other("injected atlas insertion failure").into());
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
        sprites: Arc<Mutex<usize>>,
        orders: Arc<Mutex<Vec<u32>>>,
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
            *self.sprites.lock().unwrap() = scene.polychrome_sprites.len();
            *self.orders.lock().unwrap() = scene
                .polychrome_sprites
                .iter()
                .map(|s| s.order)
                .chain(scene.quads.iter().map(|q| q.order))
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
    struct View {
        frame: BgraFrame,
        image: Arc<RenderImage>,
        result: Arc<Mutex<Option<Result<(), String>>>>,
    }
    impl gpui::Render for View {
        fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
            let (frame, image, result) =
                (self.frame.clone(), self.image.clone(), self.result.clone());
            gpui::canvas(
                |_, _, _| (),
                move |_, _, window, _| {
                    *result.lock().unwrap() = Some(preupload(&frame, image, window));
                },
            )
            .size_full()
        }
    }
    #[test]
    fn real_paint_uploads_with_empty_mask_culls_scene_and_reports_atlas_failure() {
        let atlas = Arc::new(Atlas::default());
        let sprites = Arc::new(Mutex::new(99));
        let (a, s) = (atlas.clone(), sprites.clone());
        let mut cx = gpui::HeadlessAppContext::with_platform(
            Arc::new(gpui::NoopTextSystem),
            Arc::new(()),
            move || {
                Ok(Some(Box::new(Renderer {
                    atlas: a.clone(),
                    sprites: s.clone(),
                    orders: Arc::default(),
                })))
            },
        );
        let frame = BgraFrame::new(2, 2, 8, vec![255; 16]).unwrap();
        let image = Arc::new(RenderImage::new(vec![image::Frame::new(
            image::RgbaImage::from_raw(2, 2, vec![255; 16]).unwrap(),
        )]));
        let result = Arc::new(Mutex::new(None));
        let (f, i, r) = (frame, image.clone(), result.clone());
        let window = cx
            .open_window(size(px(20.), px(20.)), |_, cx| {
                cx.new(|_| View {
                    frame: f,
                    image: i,
                    result: r,
                })
            })
            .unwrap();
        cx.update_window(window.into(), |_, window, cx| {
            window.draw(cx).clear(cx);
            window.render_to_image().unwrap();
        })
        .unwrap();
        assert!(result.lock().unwrap().as_ref().unwrap().is_ok());
        assert_eq!(
            *sprites.lock().unwrap(),
            0,
            "preuploaded candidate never enters visible scene"
        );
        cx.update_window(window.into(), |_, window, _| {
            assert!(window.has_image_atlas_entry(&image));
            window.drop_image(image.clone()).unwrap();
        })
        .unwrap();
        atlas.fail.store(true, Ordering::SeqCst);
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            window.render_to_image().unwrap();
        })
        .unwrap();
        assert!(
            result
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .as_ref()
                .unwrap_err()
                .contains("atlas insertion failure")
        );
        assert_eq!(*sprites.lock().unwrap(), 0);
    }
    struct CompositeView {
        cache: std::rc::Rc<std::cell::RefCell<crate::texture_cache::TextureCache<u64>>>,
        snapshot: AudienceProjectionSnapshot,
        page: BgraFrame,
        media: ferrostage_core::AudienceMediaFrame,
    }
    impl gpui::Render for CompositeView {
        fn render(
            &mut self,
            window: &mut Window,
            cx: &mut gpui::Context<Self>,
        ) -> impl IntoElement {
            self.cache.borrow_mut().sync(
                &[(0, self.page.clone()), (1, self.media.pixels().clone())],
                &[0, 1],
                1024,
                window,
                cx,
            );
            let (cache, snapshot, media) = (
                self.cache.clone(),
                self.snapshot.clone(),
                self.media.clone(),
            );
            gpui::canvas(
                |_, _, _| (),
                move |bounds, _, window, cx| {
                    composite(
                        &mut cache.borrow_mut(),
                        (0, bounds),
                        Some(&snapshot),
                        MediaLayers {
                            frames: &[media],
                            placeholders: &[],
                        },
                        |_| 1,
                        window,
                        cx,
                    );
                },
            )
            .size_full()
        }
    }
    #[test]
    fn real_composite_paints_pdf_then_media_then_laser_and_blocks_wrong_page_fallback() {
        let atlas = Arc::new(Atlas::default());
        let sprites = Arc::new(Mutex::new(0));
        let orders = Arc::new(Mutex::new(Vec::new()));
        let (a, s, o) = (atlas.clone(), sprites.clone(), orders.clone());
        let mut cx = gpui::HeadlessAppContext::with_platform(
            Arc::new(gpui::NoopTextSystem),
            Arc::new(()),
            move || {
                Ok(Some(Box::new(Renderer {
                    atlas: a.clone(),
                    sprites: s.clone(),
                    orders: o.clone(),
                })))
            },
        );
        let snapshot = AudienceProjectionSnapshot {
            stage: ferrostage_core::AudienceStage::Live,
            generation: ferrostage_core::DocumentGeneration(1),
            current_page: ferrostage_core::PhysicalPage(0),
            slide_crop: ferrostage_core::NormalizedRect::FULL,
            blank_mode: None,
            laser_style: ferrostage_core::LaserStyle {
                radius_norm: 0.1,
                ..Default::default()
            },
            laser_pos_norm: Some((0.5, 0.5)),
            strokes: Arc::from([]),
            visual: None,
            staging: None,
        };
        let window = cx
            .open_window(size(px(20.), px(20.)), |_, cx| {
                cx.new(|_| CompositeView {
                    cache: Default::default(),
                    snapshot,
                    page: BgraFrame::new(2, 2, 8, vec![255; 16]).unwrap(),
                    media: ferrostage_core::AudienceMediaFrame::new(
                        101,
                        ferrostage_core::NormalizedRect::FULL,
                        BgraFrame::new(2, 2, 8, vec![128; 16]).unwrap(),
                    )
                    .unwrap(),
                })
            })
            .unwrap();
        cx.update_window(window.into(), |_, window, cx| {
            window.draw(cx).clear(cx);
            window.render_to_image().unwrap();
        })
        .unwrap();
        assert_eq!(*sprites.lock().unwrap(), 2);
        let draw = orders.lock().unwrap().clone();
        assert_eq!(draw.len(), 3);
        assert!(
            draw[0] < draw[1] && draw[1] < draw[2],
            "PDF/media/laser order: {draw:?}"
        );
        // A new PDF fails actual atlas insertion: preserve complete previous PDF
        // without stale annotations; never composite the requested page's media on that PDF.
        atlas.fail.store(true, Ordering::SeqCst);
        window
            .update(&mut cx, |view, _, cx| {
                view.snapshot.current_page = ferrostage_core::PhysicalPage(1);
                view.page = BgraFrame::new(2, 2, 8, vec![64; 16]).unwrap();
                cx.notify();
            })
            .unwrap();
        cx.update_window(window.into(), |_, window, cx| {
            window.draw(cx).clear(cx);
            window.render_to_image().unwrap();
        })
        .unwrap();
        assert_eq!(*sprites.lock().unwrap(), 1);
    }
}
