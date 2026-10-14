//! Render scheduling.
//!
//! One worker thread will own PDFium. Priorities, once implemented, are:
//! P0 current audience page, P1 neighbor, P2 presenter, P3 prefetch, P4 thumbnail.
//! This module currently only publishes the cache budgets from the architecture plan.

use ferrostage_core::DocumentGeneration;

/// GPU atlas budget for the current page and ±2 neighbors.
pub const L1_GPU_BUDGET_MIB: u32 = 256;
/// CPU bitmap budget for the current page and ±6 neighbors.
pub const L2_CPU_BUDGET_MIB: u32 = 512;
/// Thumbnail strip budget.
pub const L3_THUMB_BUDGET_MIB: u32 = 64;

/// A render request that survived generation checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderRequest {
    pub generation: DocumentGeneration,
    pub page_index: usize,
}

/// Drop work that belongs to a document the presenter has already replaced.
#[must_use]
pub fn is_current(request: RenderRequest, current: DocumentGeneration) -> bool {
    request.generation == current
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_generation_is_not_current() {
        let request = RenderRequest {
            generation: DocumentGeneration(1),
            page_index: 0,
        };
        assert!(!is_current(request, DocumentGeneration(2)));
        assert!(is_current(request, DocumentGeneration(1)));
    }
}
