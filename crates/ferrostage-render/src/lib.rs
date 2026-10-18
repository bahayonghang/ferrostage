//! One PDF worker, priority scheduling, and byte-bounded CPU caches.
mod cache;
mod protocol;
mod scheduler;
mod worker;
pub use cache::{CacheKey, L1Budget};
pub use protocol::*;
pub use worker::{RenderHandle, Reply};
/// GPU textures are owned and released by UI, never by this crate.
pub const L1_GPU_BUDGET_MIB: u32 = 256;
pub const L2_CPU_BUDGET_MIB: u32 = 512;
pub const L3_THUMB_BUDGET_MIB: u32 = 64;
pub const THUMBNAIL_HEIGHT: u32 = 180;
