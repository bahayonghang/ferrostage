use crate::RenderRequest;
use ferrostage_core::{BgraFrame, DocumentGeneration, PhysicalPage};
use ferrostage_pdf::PixelSize;
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CacheKey {
    pub generation: DocumentGeneration,
    pub page: PhysicalPage,
    pub crop_bits: [u32; 4],
    pub target_px: PixelSize,
}
impl From<RenderRequest> for CacheKey {
    fn from(request: RenderRequest) -> Self {
        let crop = request.crop;
        Self {
            generation: request.generation,
            page: request.page,
            crop_bits: [crop.x, crop.y, crop.w, crop.h]
                .map(|value| if value == 0.0 { 0 } else { value.to_bits() }),
            target_px: request.target_px,
        }
    }
}
/// LRU byte ledger. UI must release every returned key, including an oversized
/// insertion's key, before retaining more GPU memory.
pub struct L1Budget {
    budget: usize,
    used: usize,
    entries: VecDeque<(CacheKey, usize)>,
}
impl L1Budget {
    pub fn new(budget: usize) -> Self {
        Self {
            budget,
            used: 0,
            entries: VecDeque::new(),
        }
    }
    pub fn used_bytes(&self) -> usize {
        self.used
    }
    pub fn touch(&mut self, key: CacheKey) -> bool {
        if let Some(index) = self.entries.iter().position(|(k, _)| *k == key) {
            if let Some(entry) = self.entries.remove(index) {
                self.entries.push_back(entry);
            }
            true
        } else {
            false
        }
    }
    pub fn remove(&mut self, key: CacheKey) {
        if let Some(index) = self.entries.iter().position(|(k, _)| *k == key)
            && let Some((_, bytes)) = self.entries.remove(index)
        {
            self.used -= bytes;
        }
    }
    pub fn retain(&mut self, key: CacheKey, bytes: usize) -> Vec<CacheKey> {
        self.remove(key);
        let mut evicted = Vec::new();
        if bytes > self.budget {
            evicted.push(key);
            return evicted;
        }
        while self.used > self.budget - bytes {
            if let Some((key, old_bytes)) = self.entries.pop_front() {
                self.used -= old_bytes;
                evicted.push(key);
            }
        }
        self.entries.push_back((key, bytes));
        self.used += bytes;
        evicted
    }
    pub fn set_budget(&mut self, budget: usize) -> Vec<CacheKey> {
        self.budget = budget;
        let mut evicted = Vec::new();
        while self.used > budget {
            if let Some((key, bytes)) = self.entries.pop_front() {
                self.used -= bytes;
                evicted.push(key);
            }
        }
        evicted
    }
    pub fn retire(&mut self, generation: DocumentGeneration) -> Vec<CacheKey> {
        let keys: Vec<_> = self
            .entries
            .iter()
            .filter_map(|(key, _)| (key.generation == generation).then_some(*key))
            .collect();
        for key in &keys {
            self.remove(*key);
        }
        keys
    }
}
pub(crate) struct FrameCache {
    ledger: L1Budget,
    frames: std::collections::HashMap<CacheKey, BgraFrame>,
}
impl FrameCache {
    pub fn new(bytes: usize) -> Self {
        Self {
            ledger: L1Budget::new(bytes),
            frames: Default::default(),
        }
    }
    pub fn get(&mut self, key: CacheKey) -> Option<BgraFrame> {
        self.ledger.touch(key);
        self.frames.get(&key).cloned()
    }
    pub fn insert(&mut self, key: CacheKey, frame: BgraFrame) {
        let bytes = frame.allocation_bytes();
        self.frames.insert(key, frame);
        for key in self.ledger.retain(key, bytes) {
            self.frames.remove(&key);
        }
    }
    pub fn set_budget(&mut self, bytes: usize) {
        for key in self.ledger.set_budget(bytes) {
            self.frames.remove(&key);
        }
    }
    pub fn used_bytes(&self) -> usize {
        self.ledger.used_bytes()
    }
    pub fn retire(&mut self, generation: DocumentGeneration) {
        for key in self.ledger.retire(generation) {
            self.frames.remove(&key);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn key(page: usize) -> CacheKey {
        CacheKey {
            generation: DocumentGeneration(1),
            page: PhysicalPage(page),
            crop_bits: [0; 4],
            target_px: PixelSize {
                width: 1,
                height: 1,
            },
        }
    }
    #[test]
    fn lru_bytes_oversize_resize_and_retirement() {
        let mut budget = L1Budget::new(8);
        assert!(budget.retain(key(1), 4).is_empty());
        budget.retain(key(2), 4);
        budget.touch(key(1));
        assert_eq!(budget.retain(key(3), 4), vec![key(2)]);
        assert_eq!(budget.retain(key(4), usize::MAX), vec![key(4)]);
        assert_eq!(budget.used_bytes(), 8);
        assert_eq!(budget.set_budget(4), vec![key(1)]);
        assert_eq!(budget.retire(DocumentGeneration(1)), vec![key(3)]);
        assert_eq!(budget.used_bytes(), 0);
    }
    #[test]
    fn cpu_frames_are_released_and_oversized_frame_is_not_cached() {
        let frame = BgraFrame::new(1, 1, 4, vec![0; 4]).unwrap();
        let mut cache = FrameCache::new(4);
        cache.insert(key(1), frame.clone());
        cache.insert(key(2), frame);
        assert!(cache.get(key(1)).is_none());
        assert!(cache.get(key(2)).is_some());
        cache.retire(DocumentGeneration(1));
        assert!(cache.frames.is_empty());
        cache.insert(key(1), BgraFrame::new(2, 1, 8, vec![0; 8]).unwrap());
        assert!(cache.frames.is_empty());
    }

    #[test]
    fn generations_share_one_budget() {
        let mut cache = FrameCache::new(4);
        let active = key(0);
        let candidate = CacheKey {
            generation: DocumentGeneration(2),
            ..active
        };
        cache.insert(active, BgraFrame::new(1, 1, 4, vec![255; 4]).unwrap());
        cache.insert(candidate, BgraFrame::new(1, 1, 4, vec![255; 4]).unwrap());
        assert!(cache.get(active).is_none());
        assert!(cache.get(candidate).is_some());
        assert_eq!(cache.ledger.used_bytes(), 4);
    }

    #[test]
    fn spare_vec_capacity_is_charged_to_the_budget() {
        let mut pixels = Vec::with_capacity(1024);
        pixels.extend_from_slice(&[0, 0, 255, 255]);
        let frame = BgraFrame::new(1, 1, 4, pixels).unwrap();
        let mut cache = FrameCache::new(4);
        cache.insert(key(0), frame);
        assert!(cache.get(key(0)).is_none());
        assert_eq!(cache.ledger.used_bytes(), 0);
    }
}
