//! A surface swaps images only after the real atlas insertion succeeds.
use ferrostage_core::{AudienceProjectionSnapshot, BgraFrame};
use gpui::{App, RenderImage, Window};
use std::sync::Arc;

struct Image {
    frame: BgraFrame,
    image: Arc<RenderImage>,
    overlay: Option<AudienceProjectionSnapshot>,
}
struct Slot<K> {
    key: K,
    desired: BgraFrame,
    committed: Option<Image>,
    pending: bool,
    failed: bool,
    error: Option<String>,
}
pub(crate) struct TextureCache<K> {
    slots: Vec<Slot<K>>,
    budget: usize,
    pending_bytes: usize,
    budget_error: bool,
    staged: Option<(u64, Vec<(K, Image)>)>,
    stage_result: Option<(u64, Result<(), String>)>,
    #[cfg(test)]
    peak_bytes: usize,
}
impl<K> Default for TextureCache<K> {
    fn default() -> Self {
        Self {
            slots: Vec::new(),
            budget: 0,
            pending_bytes: 0,
            budget_error: false,
            staged: None,
            stage_result: None,
            #[cfg(test)]
            peak_bytes: 0,
        }
    }
}
fn bytes(frame: &BgraFrame) -> usize {
    frame.width() as usize * frame.height() as usize * 4
}
fn same(a: &BgraFrame, b: &BgraFrame) -> bool {
    a.pixels_bgra().as_ptr() == b.pixels_bgra().as_ptr()
}
impl<K: Copy + PartialEq> TextureCache<K> {
    pub(crate) fn image(&self, key: K) -> Option<(BgraFrame, Arc<RenderImage>)> {
        self.slots
            .iter()
            .find(|s| s.key == key)
            .and_then(|s| s.committed.as_ref())
            .map(|i| (i.frame.clone(), i.image.clone()))
    }
    pub(crate) fn discard(&mut self, key: K, cx: &mut App) {
        if let Some(index) = self.slots.iter().position(|s| s.key == key)
            && let Some(image) = self.slots.remove(index).committed
        {
            cx.drop_image(image.image, None);
        }
    }
    /// Transfer the existing atlas owner; no upload or additional allocation.
    pub(crate) fn rename(&mut self, from: K, to: K, cx: &mut App) {
        self.discard(to, cx);
        if let Some(slot) = self.slots.iter_mut().find(|s| s.key == from) {
            slot.key = to;
        }
    }
    /// Half is settled capacity; half is swap headroom. A rejected budget
    /// keeps its previous effective value and cannot authorize new allocations.
    pub fn sync(
        &mut self,
        frames: &[(K, BgraFrame)],
        active: &[K],
        budget: usize,
        window: &mut Window,
        cx: &mut App,
    ) {
        let mut old = std::mem::take(&mut self.slots);
        for (key, frame) in frames {
            if self.slots.iter().any(|slot| slot.key == *key) {
                continue;
            }
            if let Some(index) = old.iter().position(|slot| slot.key == *key) {
                let mut slot = old.remove(index);
                if !same(&slot.desired, frame) {
                    slot.failed = false;
                    slot.error = None;
                }
                slot.desired = frame.clone();
                slot.pending = false;
                self.slots.push(slot);
            } else {
                self.slots.push(Slot {
                    key: *key,
                    desired: frame.clone(),
                    committed: None,
                    pending: false,
                    failed: false,
                    error: None,
                });
            }
        }
        for mut slot in old {
            if active.contains(&slot.key) {
                slot.pending = true;
                self.slots.push(slot);
            } else if let Some(image) = slot.committed {
                cx.drop_image(image.image, Some(window));
            }
        }
        self.slots
            .sort_by_key(|slot| active.iter().position(|key| *key == slot.key));
        self.budget_error = budget != self.budget && !self.accepts_budget(budget);
        if !self.budget_error {
            if self.budget != budget {
                for slot in &mut self.slots {
                    slot.failed = false;
                    slot.error = None;
                }
            }
            self.budget = budget;
        }
    }
    pub fn frame(&self, key: K) -> Option<BgraFrame> {
        self.slots
            .iter()
            .find(|s| s.key == key)
            .and_then(|s| s.committed.as_ref())
            .map(|i| i.frame.clone())
    }
    pub fn apply_budget(&mut self, bytes: usize) -> Result<(), String> {
        if !self.accepts_budget(bytes) {
            return Err("Retained GPU images require more swap headroom".into());
        }
        if self.budget != bytes {
            for slot in &mut self.slots {
                slot.failed = false;
                slot.error = None;
            }
        }
        self.budget = bytes;
        self.budget_error = false;
        Ok(())
    }
    pub fn effective_budget(&self) -> usize {
        self.budget
    }
    pub fn accepts_budget(&self, budget: usize) -> bool {
        self.bytes() <= budget / 2
    }
    pub fn bytes(&self) -> usize {
        self.pending_bytes
            + self.staged.as_ref().map_or(0, |(_, images)| {
                images
                    .iter()
                    .map(|(_, image)| bytes(&image.frame))
                    .sum::<usize>()
            })
            + self
                .slots
                .iter()
                .filter_map(|s| s.committed.as_ref())
                .map(|i| bytes(&i.frame))
                .sum::<usize>()
    }
    pub fn error(&self) -> Option<String> {
        if self.budget_error {
            Some("GPU budget change rejected; retained image requires swap headroom".into())
        } else {
            self.slots.iter().find_map(|slot| slot.error.clone())
        }
    }
    /// Returns true only if the desired pixels were painted. Failed candidates
    /// are retired immediately and not retried until their pixel identity changes.
    /// `draw` is the actual paint_image boundary, also injectable in regressions.
    pub fn paint(
        &mut self,
        key: K,
        overlay: Option<&AudienceProjectionSnapshot>,
        window: &mut Window,
        cx: &mut App,
        mut draw: impl FnMut(
            &BgraFrame,
            Arc<RenderImage>,
            Option<&AudienceProjectionSnapshot>,
            &mut Window,
        ) -> Result<(), String>,
    ) -> bool {
        let Some(index) = self.slots.iter().position(|s| s.key == key) else {
            return false;
        };
        let unchanged = !self.slots[index].pending
            && self.slots[index]
                .committed
                .as_ref()
                .is_some_and(|i| same(&i.frame, &self.slots[index].desired));
        if !unchanged
            && !self.slots[index].pending
            && !self.slots[index].failed
            && !self.budget_error
        {
            let allocation = bytes(&self.slots[index].desired);
            let old_bytes = self.slots[index]
                .committed
                .as_ref()
                .map_or(0, |i| bytes(&i.frame));
            // Lower-priority caches may be evicted, but never this surface's
            // rollback image. All retained and in-flight bytes remain counted.
            while allocation <= self.budget / 2
                && self.bytes() - old_bytes + allocation > self.budget / 2
                && self.slots.len() > index + 1
            {
                let Some(lower) = ((index + 1)..self.slots.len())
                    .rev()
                    .find(|i| self.slots[*i].committed.is_some())
                else {
                    break;
                };
                if let Some(image) = self.slots[lower].committed.take() {
                    cx.drop_image(image.image, Some(window));
                }
            }
            if self.bytes() - old_bytes + allocation > self.budget / 2 {
                // No upload was attempted. Recheck capacity on later paints,
                // because another surface can release its allocation.
                self.slots[index].error = Some("GPU image exceeds available swap budget".into());
            } else {
                let frame = self.slots[index].desired.clone();
                // GPUI consumes BGRA despite image's Rgba container name.
                if let Some(buffer) = image::RgbaImage::from_raw(
                    frame.width(),
                    frame.height(),
                    frame.pixels_bgra().to_vec(),
                ) {
                    let image = Arc::new(RenderImage::new(vec![image::Frame::new(buffer)]));
                    self.pending_bytes = allocation;
                    debug_assert!(self.bytes() <= self.budget);
                    #[cfg(test)]
                    {
                        self.peak_bytes = self.peak_bytes.max(self.bytes());
                    }
                    let result = draw(&frame, image.clone(), overlay, window);
                    self.pending_bytes = 0;
                    match result {
                        Ok(()) => {
                            let previous = self.slots[index].committed.replace(Image {
                                frame,
                                image,
                                overlay: overlay.cloned(),
                            });
                            if let Some(previous) = previous {
                                cx.drop_image(previous.image, Some(window));
                            }
                            self.slots[index].error = None;
                            return true;
                        }
                        Err(error) => {
                            cx.drop_image(image, Some(window));
                            self.slots[index].failed = true;
                            self.slots[index].error = Some(error);
                        }
                    }
                }
            }
        }
        if let Some(image) = &mut self.slots[index].committed {
            if unchanged {
                image.overlay = overlay.cloned();
            }
            let mut held = image.overlay.clone();
            if !unchanged && let Some(held) = &mut held {
                // Never place a new page's annotations on fallback pixels.
                held.strokes = Default::default();
                held.laser_pos_norm = None;
            }
            if let Err(error) = draw(&image.frame, image.image.clone(), held.as_ref(), window) {
                self.slots[index].error = Some(error);
                return false;
            }
            if unchanged {
                self.slots[index].error = None;
            }
            return unchanged;
        }
        false
    }
    pub fn release(&mut self, cx: &mut App) {
        self.cancel_stage(cx);
        for slot in self.slots.drain(..) {
            if let Some(image) = slot.committed {
                cx.drop_image(image.image, None);
            }
        }
        self.pending_bytes = 0;
    }
    pub fn cancel_stage(&mut self, cx: &mut App) {
        if let Some((_, images)) = self.staged.take() {
            for (_, image) in images {
                cx.drop_image(image.image, None);
            }
        }
        self.stage_result = None;
    }
    pub fn stage(
        &mut self,
        attempt: u64,
        frames: &[(K, BgraFrame)],
        window: &mut Window,
        cx: &mut App,
        mut upload: impl FnMut(&BgraFrame, Arc<RenderImage>, &mut Window) -> Result<(), String>,
    ) -> Result<(), String> {
        if let Some((old, result)) = &self.stage_result
            && *old == attempt
        {
            return result.clone();
        }
        self.cancel_stage(cx);
        let allocation: usize = frames.iter().map(|(_, frame)| bytes(frame)).sum();
        let result = if frames.is_empty()
            || allocation > self.budget / 2
            || self.bytes().saturating_add(allocation) > self.budget
        {
            Err("Reload images exceed available GPU swap budget".into())
        } else {
            let mut images = Vec::new();
            let mut result = Ok(());
            self.pending_bytes = allocation;
            for (key, frame) in frames {
                let Some(buffer) = image::RgbaImage::from_raw(
                    frame.width(),
                    frame.height(),
                    frame.pixels_bgra().to_vec(),
                ) else {
                    result = Err("Invalid reload image".into());
                    break;
                };
                let image = Arc::new(RenderImage::new(vec![image::Frame::new(buffer)]));
                if let Err(error) = upload(frame, image.clone(), window) {
                    cx.drop_image(image, Some(window));
                    result = Err(error);
                    break;
                }
                images.push((
                    *key,
                    Image {
                        frame: frame.clone(),
                        image,
                        overlay: None,
                    },
                ));
            }
            self.pending_bytes = 0;
            if result.is_ok() {
                self.staged = Some((attempt, images));
            } else {
                for (_, image) in images {
                    cx.drop_image(image.image, Some(window));
                }
            }
            result
        };
        self.stage_result = Some((attempt, result.clone()));
        result
    }
    pub fn staged(&self, attempt: u64) -> bool {
        self.staged
            .as_ref()
            .is_some_and(|(token, _)| *token == attempt)
    }
    pub fn promote(&mut self, attempt: u64, cx: &mut App) -> bool {
        if !self.staged(attempt) {
            return false;
        }
        let Some((_, images)) = self.staged.take() else {
            return false;
        };
        for slot in self.slots.drain(..) {
            if let Some(old) = slot.committed {
                cx.drop_image(old.image, None);
            }
        }
        for (key, image) in images {
            self.slots.push(Slot {
                key,
                desired: image.frame.clone(),
                committed: Some(image),
                pending: false,
                failed: false,
                error: None,
            });
        }
        self.stage_result = None;
        true
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use ferrostage_core::{
        AudienceStage, AudienceVisual, BlankMode, DocumentGeneration, PhysicalPage,
        PresentationState,
    };
    fn frame(value: u8) -> BgraFrame {
        BgraFrame::new(2, 2, 8, vec![value; 16]).unwrap()
    }
    #[gpui::test]
    fn staged_upload_keeps_old_pixels_counts_both_and_promotes_the_same_arc(
        cx: &mut gpui::TestAppContext,
    ) {
        let window = cx.add_empty_window();
        let mut cache = TextureCache::default();
        window.update(|window, cx| {
            let old = frame(1);
            cache.sync(&[((), old)], &[()], 32, window, cx);
            assert!(cache.paint((), None, window, cx, |_, _, _, _| Ok(())));
            let mut staged = None;
            cache
                .stage(1, &[((), frame(2))], window, cx, |_, image, _| {
                    staged = Some(image);
                    Ok(())
                })
                .unwrap();
            assert_eq!(cache.bytes(), 32);
            assert_eq!(cache.frame(()).unwrap().pixels_bgra()[0], 1);
            assert!(cache.promote(1, cx));
            assert_eq!(cache.bytes(), 16);
            assert!(Arc::ptr_eq(
                &cache.slots[0].committed.as_ref().unwrap().image,
                &staged.unwrap()
            ));
            assert_eq!(cache.frame(()).unwrap().pixels_bgra()[0], 2);
            assert!(
                cache
                    .stage(2, &[((), frame(3))], window, cx, |_, _, _| Err(
                        "atlas failure".into()
                    ))
                    .is_err()
            );
            assert_eq!(cache.bytes(), 16);
            assert!(!cache.staged(2));
            assert_eq!(cache.frame(()).unwrap().pixels_bgra()[0], 2);
            assert!(
                cache
                    .stage(2, &[((), frame(3))], window, cx, |_, _, _| panic!(
                        "failed identity must not spin"
                    ))
                    .is_err()
            );
            cache
                .stage(3, &[((), frame(4))], window, cx, |_, _, _| Ok(()))
                .unwrap();
            cache.cancel_stage(cx);
            assert_eq!(cache.bytes(), 16);
            let huge = BgraFrame::new(4, 4, 16, vec![0; 64]).unwrap();
            assert!(
                cache
                    .stage(4, &[((), huge)], window, cx, |_, _, _| panic!(
                        "budget must reject before allocation"
                    ))
                    .is_err()
            );
        });
    }
    #[gpui::test]
    fn media_instance_change_retires_prior_pixels_before_failed_new_upload(
        cx: &mut gpui::TestAppContext,
    ) {
        let window = cx.add_empty_window();
        let mut cache = TextureCache::default();
        window.update(|window, cx| {
            cache.sync(&[(101, frame(11))], &[101], 64, window, cx);
            let mut old = None;
            assert!(cache.paint(101, None, window, cx, |_, image, _, _| {
                old = Some(Arc::downgrade(&image));
                Ok(())
            }));
            cache.sync(&[(102, frame(22))], &[102], 64, window, cx);
            assert!(old.unwrap().upgrade().is_none());
            let mut drawn = vec![];
            assert!(!cache.paint(102, None, window, cx, |pixels, _, _, _| {
                drawn.push(pixels.pixels_bgra()[0]);
                Err("injected media atlas failure".into())
            }));
            assert_eq!(drawn, [22]);
            assert_eq!(cache.bytes(), 0);
            assert!(cache.frame(101).is_none());
            cache.sync(&[], &[], 64, window, cx);
            assert_eq!(cache.bytes(), 0);
        });
    }
    fn projection(frame: &BgraFrame, page: usize) -> AudienceProjectionSnapshot {
        let mut snapshot = PresentationState::new(3).unwrap().audience_snapshot();
        snapshot.stage = AudienceStage::Live;
        snapshot.generation = DocumentGeneration(7);
        snapshot.current_page = PhysicalPage(page);
        snapshot.laser_pos_norm = Some((0.5, 0.5));
        snapshot.visual = Some(AudienceVisual {
            page: frame.clone(),
            media: vec![],
            texture_budget_bytes: 32,
        });
        snapshot
    }
    #[gpui::test]
    fn upload_failure_repaints_complete_identity_retires_candidate_and_safety_wins(
        cx: &mut gpui::TestAppContext,
    ) {
        let window = cx.add_empty_window();
        let mut cache = TextureCache::default();
        let old = frame(1);
        let new = frame(2);
        let first = projection(&old, 0);
        let next = projection(&new, 1);
        window.update(|window, cx| {
            cache.sync(&[((), old.clone())], &[()], 32, window, cx);
            let mut old_image = None;
            assert!(crate::audience::paint_projection(
                &mut cache,
                &first,
                window,
                cx,
                |pixels, image, snapshot, _| {
                    assert_eq!(pixels.pixels_bgra(), &[1; 16]);
                    assert_eq!(snapshot.unwrap().current_page, PhysicalPage(0));
                    old_image = Some(Arc::downgrade(&image));
                    Ok(())
                }
            ));
            // A raster gap must retain the only GPU fallback.
            cache.sync(&[], &[()], 32, window, cx);
            assert_eq!(cache.bytes(), 16);
            assert!(!crate::audience::paint_projection(
                &mut cache,
                &next,
                window,
                cx,
                |pixels, _, snapshot, _| {
                    assert_eq!(pixels.pixels_bgra()[0], 1);
                    assert_eq!(snapshot.unwrap().current_page, PhysicalPage(0));
                    assert!(snapshot.unwrap().laser_pos_norm.is_none());
                    Ok(())
                }
            ));
            cache.sync(&[((), new.clone())], &[()], 32, window, cx);
            let mut failed_image = None;
            let mut calls = vec![];
            assert!(!crate::audience::paint_projection(
                &mut cache,
                &next,
                window,
                cx,
                |pixels, image, snapshot, _| {
                    calls.push(pixels.pixels_bgra()[0]);
                    if pixels.pixels_bgra()[0] == 2 {
                        failed_image = Some(Arc::downgrade(&image));
                        return Err("injected atlas upload failure".into());
                    }
                    let held = snapshot.unwrap();
                    assert_eq!(held.current_page, PhysicalPage(0));
                    assert_eq!(held.generation, DocumentGeneration(7));
                    assert_eq!(held.visual.as_ref().unwrap().page.pixels_bgra(), &[1; 16]);
                    assert!(held.strokes.is_empty());
                    assert!(held.laser_pos_norm.is_none());
                    Ok(())
                }
            ));
            assert_eq!(calls, [2, 1]);
            assert!(failed_image.unwrap().upgrade().is_none());
            assert!(old_image.as_ref().unwrap().upgrade().is_some());
            assert_eq!(cache.bytes(), 16);
            assert_eq!(cache.peak_bytes, 32);
            assert_eq!(
                cache.error().as_deref(),
                Some("injected atlas upload failure")
            );
            // A rejected shrink and rollback are not new upload permission.
            cache.sync(&[((), new.clone())], &[()], 16, window, cx);
            assert_eq!(cache.effective_budget(), 32);
            cache.sync(&[((), new.clone())], &[()], 32, window, cx);
            assert_eq!(
                cache.error().as_deref(),
                Some("injected atlas upload failure")
            );
            // Repeated draw of identical failed pixels cannot retry the upload.
            for _ in 0..3 {
                cache.sync(&[((), new.clone())], &[()], 32, window, cx);
                crate::audience::paint_projection(
                    &mut cache,
                    &next,
                    window,
                    cx,
                    |pixels, _, _, _| {
                        assert_eq!(pixels.pixels_bgra()[0], 1);
                        Ok(())
                    },
                );
            }
            for (stage, blank) in [
                (AudienceStage::Curtain, None),
                (AudienceStage::Live, Some(BlankMode::Black)),
                (AudienceStage::Live, Some(BlankMode::White)),
            ] {
                let mut covered = next.clone();
                covered.stage = stage;
                covered.blank_mode = blank;
                assert!(!crate::audience::paint_projection(
                    &mut cache,
                    &covered,
                    window,
                    cx,
                    |_, _, _, _| panic!("safety must suppress candidate and fallback")
                ));
            }
            // A newly rasterized identity enables one new attempt, success
            // atomically replaces complete metadata and retires the old image.
            let retry = frame(3);
            let retry_snapshot = projection(&retry, 2);
            cache.sync(&[((), retry)], &[()], 32, window, cx);
            assert!(crate::audience::paint_projection(
                &mut cache,
                &retry_snapshot,
                window,
                cx,
                |pixels, _, snapshot, _| {
                    assert_eq!(pixels.pixels_bgra()[0], 3);
                    assert_eq!(snapshot.unwrap().current_page, PhysicalPage(2));
                    Ok(())
                }
            ));
            assert!(old_image.unwrap().upgrade().is_none());
            assert!(cache.error().is_none());
            assert_eq!(
                cache.slots[0]
                    .committed
                    .as_ref()
                    .unwrap()
                    .overlay
                    .as_ref()
                    .unwrap()
                    .current_page,
                PhysicalPage(2)
            );
            let last = Arc::downgrade(&cache.slots[0].committed.as_ref().unwrap().image);
            cache.release(cx);
            assert_eq!(cache.bytes(), 0);
            assert!(last.upgrade().is_none());
        });
    }
    #[gpui::test]
    fn repaint_recovers_and_capacity_refusal_retries_only_after_space_is_available(
        cx: &mut gpui::TestAppContext,
    ) {
        let window = cx.add_empty_window();
        let mut cache = TextureCache::default();
        window.update(|window, cx| {
            let first = frame(1);
            let second = frame(2);
            cache.sync(
                &[(0, first.clone()), (1, second.clone())],
                &[0, 1],
                32,
                window,
                cx,
            );
            assert!(cache.paint(0, None, window, cx, |_, _, _, _| Ok(())));
            assert!(!cache.paint(0, None, window, cx, |_, _, _, _| Err(
                "transient repaint failure".into()
            )));
            assert!(cache.error().is_some());
            assert!(cache.paint(0, None, window, cx, |_, _, _, _| Ok(())));
            assert!(cache.error().is_none());
            for _ in 0..3 {
                assert!(!cache.paint(1, None, window, cx, |_, _, _, _| panic!(
                    "capacity refusal must not upload"
                )));
            }
            cache.sync(&[(1, second)], &[1], 32, window, cx);
            assert!(cache.paint(1, None, window, cx, |_, _, _, _| Ok(())));
            assert!(cache.error().is_none());
            // An impossible high-priority candidate must not destroy an
            // otherwise usable lower-priority committed image.
            let oversized = BgraFrame::new(8, 4, 32, vec![4; 128]).unwrap();
            cache.sync(
                &[(0, oversized), (1, cache.frame(1).unwrap())],
                &[0, 1],
                32,
                window,
                cx,
            );
            assert!(!cache.paint(0, None, window, cx, |_, _, _, _| panic!(
                "oversized candidate must not upload"
            )));
            assert_eq!(cache.frame(1).unwrap().pixels_bgra()[0], 2);
            cache.release(cx);
        });
    }

    #[gpui::test]
    fn budget_shrink_rejects_without_losing_fallback_and_priority_accounts_for_swap(
        cx: &mut gpui::TestAppContext,
    ) {
        let window = cx.add_empty_window();
        let mut cache = TextureCache::default();
        window.update(|window, cx| {
            let first = frame(1);
            let second = frame(2);
            cache.sync(
                &[(0, first.clone()), (1, second.clone())],
                &[0, 1],
                64,
                window,
                cx,
            );
            for key in [0, 1] {
                assert!(cache.paint(key, None, window, cx, |_, _, _, _| Ok(())));
            }
            assert_eq!(cache.bytes(), 32);
            assert!(!cache.accepts_budget(32));
            cache.sync(
                &[(0, first.clone()), (1, second.clone())],
                &[0, 1],
                32,
                window,
                cx,
            );
            assert_eq!(cache.effective_budget(), 64);
            assert_eq!(cache.bytes(), 32);
            assert!(cache.error().is_some());
            assert!(cache.paint(0, None, window, cx, |pixels, _, _, _| {
                assert_eq!(pixels.pixels_bgra()[0], 1);
                Ok(())
            }));
            // Returning to the already effective budget converges immediately.
            cache.sync(&[(0, first), (1, second.clone())], &[0, 1], 64, window, cx);
            assert!(cache.error().is_none());
            let larger = BgraFrame::new(4, 2, 16, vec![3; 32]).unwrap();
            cache.sync(&[(0, larger), (1, second)], &[0, 1], 64, window, cx);
            assert!(cache.paint(0, None, window, cx, |_, _, _, _| Ok(())));
            assert_eq!(cache.bytes(), 32);
            assert_eq!(cache.peak_bytes, 48); // old16 + candidate32
            assert!(cache.slots[1].committed.is_none());
            let oversized = BgraFrame::new(8, 4, 32, vec![4; 128]).unwrap();
            cache.sync(&[(0, oversized)], &[0], 64, window, cx);
            assert!(!cache.paint(0, None, window, cx, |pixels, _, _, _| {
                assert_eq!(pixels.pixels_bgra()[0], 3);
                Ok(())
            }));
            assert_eq!(cache.effective_budget(), 64);
            assert_eq!(cache.bytes(), 32);
            assert!(cache.error().is_some());
            cache.sync(&[], &[], 0, window, cx);
            assert_eq!(cache.bytes(), 0);
            assert_eq!(cache.effective_budget(), 0);
        });
    }
}
