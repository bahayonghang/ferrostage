# Quality Guidelines

UI changes still go through `just ci`. The view crate does not get a weaker clippy profile.

## Window rules

- Audience view input type is `AudienceProjectionSnapshot`, including `stage: AudienceStage`.
- The audience window exists only in `Presenting`. A `Curtain` snapshot draws solid black and carries no visual.
- Presenter view may show `SlideNotes`.
- Both views share one `Arc` frame from `ferrostage-media`. No child `HWND` (Airspace).
- Pixel format stays BGRA8.

## GPUI pin

```rust
pub const ZED_GPUI_TAG: &str = "v1.23.3";
pub const GPUI_KIT_VERSION: &str = "0.7.3";
```

A change of either constant updates `docs/adr/ADR-001-gpui-from-zed-tag.md` in the same commit.

## Forbidden

- Linking GPUI from `ferrostage-core`.
- A web frontend, a localhost control page, or telemetry to "measure frame time".
- Blocking the GPUI thread on PDFium. Raster requests go to the single render worker.
- `unwrap` on a missing monitor. A missing Presenter may fall back to the primary display after the saved Audience id is excluded, and log at `warn` (`PresenterMissingFallbackPrimary`). That fallback does not assign an Audience. With no saved record, the Presenter is the primary display, or the first `MonitorId` in sort order when none is primary. A saved Audience that is absent, or an explicit `None`, must not take another display. Zero monitors resolve to no presenter and no audience.


## Presenter v0.3 implementation-first scheduling

For the existing10-14-presenter-v0.3 tree, the2026-10-15 user decision in `.trellis/tasks/10-14-presenter-v0.3/acceptance-policy.md` defers physical/exhaustive native/size/performance matrices until after feature implementation. Retain automatic regressions, strict lint/typecheck/build/license checks and native-functional smoke where needed. Do not weaken runtime contracts or label deferred checks PASS; the original quality/performance targets remain later qualification work.

## Headless presenter bounds

`just ci` uses default features. Presenter code is behind `gpui`, so that recipe does not compile it. Also run:

```text
cargo clippy -p ferrostage-ui --features test-support --all-targets --locked -- -D warnings
cargo test -p ferrostage-ui --features test-support --lib --locked
```

`VisualTestContext::debug_bounds` only returns elements that set `.debug_selector`. `.id()` alone records nothing. `window.draw` clears the next frame's debug map before paint.

An explicit `track_focus` handle does not copy the div's `.tab_stop`. Call `.tab_index(0).tab_stop(...)` on that `FocusHandle` before `track_focus`. GPUI copies `tab_stop` onto an auto-created handle only when no handle is tracked.

## Debug view stack budget

Windows debug executables reserve a 1 MiB main-thread stack. Large by-value GPUI `Div` builder temporaries can exceed it without recursion, including temporaries belonging to inactive branches in a large settings function. Keep shared display construction in bounded detection/overview/monitor-row builders, and General-only recent-file construction outside `Presenter::settings_panel`. Helpers must return the original content element rather than introduce layout wrappers.

Retain `shell::tests::display_settings_both_hosts_fit_windows_main_thread_stack`: it runs the actual shared-panel interaction test in a subprocess with `RUST_MIN_STACK=1048576`, checks successful exit and confirms one test executed. Both standalone and embedded hosts must render and retain existing draft/details/cancel behavior. A regular Rust test thread has a larger default stack and can miss this failure. For native stack failures, inspect compiled frame reservations and the executable's PE stack reserve; preserve the ordinary stack limit and debug profile. Keep native `just dev` click/exit acceptance separate from this headless regression.
