//! Local, opt-in paint and navigation timings. Use RUST_LOG=ferrostage_perf=info.
use std::{sync::OnceLock, time::Instant};

pub(crate) fn micros(at: Instant) -> u64 {
    static ORIGIN: OnceLock<Instant> = OnceLock::new();
    at.saturating_duration_since(*ORIGIN.get_or_init(|| at))
        .as_micros() as u64
}

// tracing evaluates the timestamp and fields only when this target is enabled.
macro_rules! mark {
    (at $at:expr, $event:literal, $($fields:tt)*) => {
        tracing::info!(target: "ferrostage_perf", us = $crate::perf::micros($at), $($fields)*, $event)
    };
    ($event:literal, $($fields:tt)*) => {
        $crate::perf::mark!(at std::time::Instant::now(), $event, $($fields)*)
    };
}
pub(crate) use mark;
